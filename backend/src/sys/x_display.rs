use std::cmp::Ordering;

use edid::EDID;
use float_cmp::approx_eq;
use xrandr::{Mode, Output, Relation, ScreenResources, XHandle};

use anyhow::Result;

/// Thin wrapper around xrandr for common display operations.
pub struct XDisplay {
    xhandle: XHandle,
}

pub struct ModePreference {
    pub resolution: ModeOption<Resolution>,
    pub aspect_ratio: AspectRatioOption,
    pub refresh: ModeOption<f64>,
}

pub enum ModeOption<T> {
    Exact(T),
    AtLeast(T),
    AtMost(T),
}

pub struct Resolution {
    pub w: u32,
    pub h: u32,
}

pub enum AspectRatioOption {
    Native,
    Exact(f32),
}

impl XDisplay {
    pub fn new() -> Result<Self> {
        Ok(Self {
            xhandle: xrandr::XHandle::open()?,
        })
    }

    pub fn get_embedded_output(&mut self) -> Result<Option<Output>> {
        let outputs: Vec<Output> = self.xhandle.all_outputs()?;

        Ok(outputs.into_iter().filter(|o| o.name == "eDP").last())
    }

    // TODO::allow user to select target output.
    /// Gets the preferred output, ignoring the steam decks embedded display. Chooses primary enabled output if available, otherwise largest.
    pub fn get_preferred_external_output(&mut self) -> Result<Option<Output>> {
        let mut outputs: Vec<Output> = self.xhandle.all_outputs()?;

        outputs.sort_by(|a, b| {
            let is_primary = a.is_primary.cmp(&b.is_primary);
            if is_primary == Ordering::Equal {
                let has_mode = a.current_mode.is_some().cmp(&b.current_mode.is_some());
                if has_mode == Ordering::Equal {
                    (a.mm_height * a.mm_width).cmp(&(b.mm_height * b.mm_width))
                } else {
                    has_mode
                }
            } else {
                is_primary
            }
        });

        Ok(outputs
            .into_iter()
            .filter_map(|o| if o.name == "eDP" { None } else { Some(o) })
            .next())
    }

    pub fn set_output_mode(&mut self, output: &Output, mode: &Mode) -> Result<()> {
        Ok(self.xhandle.set_mode(output, mode)?)
    }

    pub fn set_output_position(
        &mut self,
        output: &Output,
        relation: &Relation,
        relative_output: &Output,
    ) -> Result<()> {
        Ok(self
            .xhandle
            .set_position(output, relation, relative_output)?)
    }

    pub fn set_or_create_preferred_mode(
        &mut self,
        output: &Output,
        pref: &ModePreference,
    ) -> Result<()> {
        let screen = ScreenResources::new(&mut self.xhandle)?;
        let preferred_modes = output.preferred_modes.iter().collect::<Vec<_>>();

        let modes = output
            .modes
            .iter()
            .map(|xid| screen.mode(*xid))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|m| {
                let is_pref = preferred_modes.contains(&&m.xid);
                (m, is_pref)
            })
            .collect::<Vec<_>>();

        let native_ar = {
            let output_edid = output.edid();
            let output_edid: Option<_> = output_edid
                .map(|e| {
                    let edid_data = edid::parse(&e);
                    let res = edid_data.to_result().ok();
                    res.map(|r| r.display)
                })
                .flatten();

            let largest_mode = modes.iter().reduce(|a, b| {
                if a.0.width * a.0.height > b.0.width * b.0.height {
                    a
                } else {
                    b
                }
            });
            output_edid
                .map(|e| e.width as f32 / e.height as f32)
                .unwrap_or(
                    largest_mode
                        .map(|m| m.0.width as f32 / m.0.height as f32)
                        .unwrap_or(16. / 9.),
                )
        };

        let mode = Self::get_preferred_mode(native_ar, modes, pref)?;
        let mode = match mode {
            Some(mode) => mode,
            None => self.create_preferred_mode(output, pref)?,
        };

        self.set_output_mode(output, &mode)
    }

    /// Gets the best mode matching a preference from a list of modes.
    ///
    /// # Parameters
    /// * native_ar - native aspect ratio
    /// * modes - Modes to select from, in the format ([Mode], is_preferred_mode)
    /// * pref - the preferences for the selected mode
    fn get_preferred_mode(
        native_ar: f32,
        modes: Vec<(Mode, bool)>,
        pref: &ModePreference,
    ) -> Result<Option<Mode>> {
        struct ModeDiff {
            width: i64,
            rr: f64,
            pref: bool,
        }

        let best_mode = modes
            .into_iter()
            .fold(None, |acc: Option<(Mode, ModeDiff)>, e| {
                let rr_diff = match pref.refresh {
                    ModeOption::Exact(rr) => {
                        if approx_eq!(f64, rr, e.0.rate, ulps = 2) {
                            Some(rr - e.0.rate)
                        } else {
                            None
                        }
                    }
                    ModeOption::AtLeast(rr) => {
                        if  e.0.rate >= rr {
                            Some( e.0.rate - rr)
                        } else {
                            None
                        }
                    }
                    ModeOption::AtMost(rr) => {
                        if rr <= e.0.rate {
                            Some(rr - e.0.rate)
                        } else {
                            None
                        }
                    }
                };

                let mode_ar = e.0.width as f32 / e.0.height as f32;
                let ar_diff = match pref.aspect_ratio {
                    AspectRatioOption::Native => {
                        if approx_eq!(f32, native_ar, mode_ar, ulps = 2) {
                            Some(native_ar - mode_ar)
                        } else {
                            None
                        }
                    }
                    AspectRatioOption::Exact(ar) => {
                        if approx_eq!(f32, ar, mode_ar, ulps = 2) {
                            Some(ar - mode_ar)
                        } else {
                            None
                        }
                    }
                };

                let ar_scale = mode_ar / native_ar;

                let scaled_h = (e.0.height as f32 * ar_scale).round() as u32; // TODO::verify scaling with other resolutions + aspect ratios than 16/9 -> 16/10;
                let res_diff = match &pref.resolution {
                    ModeOption::Exact(res) => {
                        if res.w == e.0.width && res.h == scaled_h {
                            Some((0, 0))
                        } else {
                            None
                        }
                    }
                    ModeOption::AtLeast(res) => {
                        if res.w >= e.0.width && res.h >= scaled_h {
                            Some((
                                res.w as i64 - e.0.width as i64,
                                res.h as i64 - scaled_h as i64,
                            ))
                        } else {
                            None
                        }
                    }
                    ModeOption::AtMost(res) => {
                        if res.w <= e.0.width && res.h <= scaled_h {
                            Some((
                                res.w as i64 - e.0.width as i64,
                                res.h as i64 - scaled_h as i64,
                            ))
                        } else {
                            None
                        }
                    }
                };

                if let (Some(res), Some(_), Some(rr)) = (res_diff, ar_diff, rr_diff) {
                    let diff = ModeDiff {
                        width: res.0,
                        rr,
                        pref: e.1,
                    };

                    if let Some(best) = acc {
                        match best
                            .1
                            .rr
                            .partial_cmp(&rr)
                            .expect("refresh rates should be real numbers")
                        {
                            Ordering::Less => Some((e.0, diff)),
                            Ordering::Greater => Some(best),
                            Ordering::Equal => match best.1.width.cmp(&res.0) {
                                Ordering::Less => Some((e.0, diff)),
                                Ordering::Greater => Some(best),
                                Ordering::Equal => match best.1.pref.cmp(&e.1) {
                                    Ordering::Less => Some((e.0, diff)),
                                    Ordering::Equal | Ordering::Greater => Some(best),
                                },
                            },
                        }
                    } else {
                        Some((e.0, diff))
                    }
                } else {
                    acc
                }
            });

        Ok(best_mode.map(|m| m.0))
    }

    fn create_preferred_mode(&mut self, output: &Output, pref: &ModePreference) -> Result<Mode> {
        // TODO::configurable timing methods
        todo!("TODO::create mode from preference");
    }

    pub fn get_current_mode(&mut self, output: &Output) -> Result<Option<Mode>> {
        let resources = ScreenResources::new(&mut self.xhandle)?;
        Ok(output
            .current_mode
            .map(|id| resources.mode(id))
            .transpose()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_mode(xid: u64, width: u32, height: u32, rate: f64, pref: bool) -> (Mode, bool) {
        (
            Mode {
                xid,
                width,
                height,
                dot_clock: 0,
                hsync_tart: 0,
                hsync_end: 0,
                htotal: 0,
                hskew: 0,
                vsync_start: 0,
                vsync_end: 0,
                vtotal: 0,
                name: "".to_string(),
                flags: 0,
                rate,
            },
            pref,
        )
    }

    const s9: f32 = 16. / 9.;
    const s10: f32 = 16. / 10.;

    #[test]
    fn test_get_preferred_mode_prf() -> Result<()> {
        let modes = vec![
            create_mode(1, 1280, 720, 60., true),
            create_mode(2, 1280, 720, 60., false),
        ];

        let mode = XDisplay::get_preferred_mode(
            s9,
            modes,
            &ModePreference {
                resolution: ModeOption::Exact(Resolution { w: 1280, h: 720 }),
                aspect_ratio: AspectRatioOption::Exact(s9),
                refresh: ModeOption::Exact(60.),
            },
        )?
        .map(|m| m.xid);

        assert_eq!(Some(1), mode);
        Ok(())
    }

    #[test]
    fn test_get_preferred_mode_exact_rate() -> Result<()> {
        let modes = vec![
            create_mode(1, 1280, 720, 60., false),
            create_mode(2, 1280, 720, 61., false),
            create_mode(3, 1280, 720, 59., false),
        ];

        let mode = XDisplay::get_preferred_mode(
            s9,
            modes,
            &ModePreference {
                resolution: ModeOption::Exact(Resolution { w: 1280, h: 720 }),
                aspect_ratio: AspectRatioOption::Exact(s9),
                refresh: ModeOption::Exact(60.),
            },
        )?
        .map(|m| m.xid);

        assert_eq!(Some(1), mode);
        Ok(())
    }

    #[test]
    fn test_get_preferred_mode_min_rate() -> Result<()> {
        let modes = vec![
            create_mode(1, 1280, 720, 15., false),
            create_mode(2, 1280, 720, 60., false),
            create_mode(3, 1280, 720, 30., false),
        ];

        let mode = XDisplay::get_preferred_mode(
            s9,
            modes,
            &ModePreference {
                resolution: ModeOption::Exact(Resolution { w: 1280, h: 720 }),
                aspect_ratio: AspectRatioOption::Exact(s9),
                refresh: ModeOption::AtLeast(30.),
            },
        )?
        .map(|m| m.xid);

        assert_eq!(Some(2), mode);
        Ok(())
    }

    #[test]
    fn test_get_preferred_mode_max_rate() -> Result<()> {
        let modes = vec![
            create_mode(1, 1280, 720, 15., false),
            create_mode(2, 1280, 720, 60., false),
            create_mode(3, 1280, 720, 30., false),
        ];

        let mode = XDisplay::get_preferred_mode(
            s9,
            modes,
            &ModePreference {
                resolution: ModeOption::Exact(Resolution { w: 1280, h: 720 }),
                aspect_ratio: AspectRatioOption::Exact(s9),
                refresh: ModeOption::AtMost(30.),
            },
        )?
        .map(|m| m.xid);

        assert_eq!(Some(3), mode);
        Ok(())
    }

    // #[test]
    // fn test_get_preferred_mode_exact_res_native() {
    //     todo!()
    // }

    // #[test]
    // fn test_get_preferred_mode_min_res_native() {
    //     todo!()
    // }

    // #[test]
    // fn test_get_preferred_mode_max_res_native() {
    //     todo!()
    // }
}
