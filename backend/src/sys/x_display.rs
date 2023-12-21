use std::{cmp::Ordering, process::Command, str::FromStr, time::Duration};

use float_cmp::approx_eq;
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::thread::sleep;
use xrandr::{Mode, Output, Relation, ScreenResources, XHandle, XId};

use anyhow::{Ok, Result};

/// Thin wrapper around xrandr for common display operations.
#[derive(Debug)]
pub struct XDisplay {
    xhandle: XHandle,
    timing_fallback: TimingFallbackMethod,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct ModePreference {
    pub resolution: ModeOption<Resolution>,
    pub aspect_ratio: AspectRatioOption,
    pub refresh: ModeOption<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub enum ModeOption<T> {
    Exact(T),
    AtLeast(T),
    AtMost(T),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct Resolution {
    pub w: u32, // TODO::enforce w is multiple of 8 for CVT
    pub h: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub enum AspectRatioOption {
    Any,
    Native,
    Exact(f32),
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub enum TimingFallbackMethod {
    #[default]
    CvtR,
    Cvt,
    // Gtf,
    // Manual
}

// TODO::rework this - its messy and unlikely to play nicely with all configurations; its either doing too much, or too little.
impl XDisplay {
    pub fn new() -> Result<Self> {
        Ok(Self {
            xhandle: xrandr::XHandle::open()?,
            timing_fallback: TimingFallbackMethod::Cvt, // TODO::make this configurable
        })
    }

    pub fn get_embedded_output(&mut self) -> Result<Option<Output>> {
        let outputs: Vec<Output> = self.xhandle.all_outputs()?;

        Ok(outputs.into_iter().filter(|o| o.name == "eDP").last())
    }

    // TODO::allow user to select target output.
    /// Gets the preferred output, ignoring the steam decks embedded display. Chooses primary enabled output if available, otherwise largest.
    pub fn get_preferred_external_output(&mut self) -> Result<Option<Output>> {
        let external = {
            let mut maybe_external = self.get_preferred_external_output_maybe_disconnected()?;

            let mut fail_count = 0;
            const MAX_FAIL_COUNT: u8 = 15;
            while fail_count <= MAX_FAIL_COUNT {
                if let Some(external) = maybe_external {
                    if external.connected {
                        log::debug!("Returning connected external output");

                        return Ok(Some(external));
                    } else if fail_count == MAX_FAIL_COUNT {
                        log::debug!("Returning disconnected external output");

                        return Ok(Some(external));
                    }

                    fail_count += 1;
                    sleep(Duration::from_secs(1));
                    maybe_external = self.get_preferred_external_output_maybe_disconnected()?;
                }
            }

            log::debug!("Returning bad external output");

            maybe_external
        };

        Ok(external)
    }

    fn get_preferred_external_output_maybe_disconnected(&mut self) -> Result<Option<Output>> {
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

    /// Gets the current mode of an output
    pub fn get_current_mode(&mut self, output: &Output) -> Result<Option<Mode>> {
        let resources = ScreenResources::new(&mut self.xhandle)?;
        Ok(output
            .current_mode
            .map(|id| resources.mode(id))
            .transpose()?)
    }

    /// Sets the mode of an output.
    pub fn set_output_mode(&mut self, output: &Output, mode: &Mode) -> Result<()> {
        log::debug!("setting output {} mode to {}", output.name, mode.name);

        let res = Command::new("xrandr")
            .args(["--output", &output.name, "--mode", &mode.name])
            .output()?;
        if res.status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Error setting output mode: {}",
                String::from_utf8_lossy(&res.stderr)
            ))
        }
    }

    /// Sets the position of one output relative to another.
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

    /// Finds a mode matching the preference, or creates one if none matching are found, and sets the output to that mode.
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

        let native_ar = Self::get_native_ar(&modes);

        let mode = Self::get_preferred_mode(native_ar, &modes, pref)?;
        log::debug!("Got preferred mode {mode:?}");
        let mode = match mode {
            Some(mode) => screen.mode(mode)?,
            None => self.create_preferred_mode(output, native_ar, &modes, pref)?,
        };

        self.set_output_mode(output, &mode)
    }

    fn scale_height(mode_ar: f32, native_ar: f32, height: u32) -> u32 {
        let ar_scale = mode_ar / native_ar;

        (height as f32 * ar_scale).round() as u32 // TODO::verify scaling with other resolutions + aspect ratios than 16/9 -> 16/10;
    }

    /// Gets the best mode matching a preference from a list of modes.
    ///
    /// # Parameters
    /// * native_ar - native aspect ratio
    /// * modes - Modes to select from, in the format ([Mode], is_preferred_mode)
    /// * pref - the preferences for the selected mode
    fn get_preferred_mode(
        native_ar: f32,
        modes: &[(Mode, bool)],
        pref: &ModePreference,
    ) -> Result<Option<XId>> {
        struct ModeDiff {
            width: i64,
            rr: f64,
            pref: bool,
        }

        let best_mode = modes
            .iter()
            .fold(None, |acc: Option<(&Mode, ModeDiff)>, e| {
                let rr_diff = match pref.refresh {
                    ModeOption::Exact(rr) => {
                        if approx_eq!(f64, rr, e.0.rate, ulps = 2) {
                            Some(rr - e.0.rate)
                        } else {
                            None
                        }
                    }
                    ModeOption::AtLeast(rr) => {
                        if e.0.rate >= rr {
                            Some(e.0.rate - rr)
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
                    AspectRatioOption::Any => Some(0.),
                };

                let scaled_h = Self::scale_height(mode_ar, native_ar, e.0.height);

                let res_diff = match &pref.resolution {
                    ModeOption::Exact(res) => {
                        if res.w == e.0.width && scaled_h == e.0.height {
                            Some((0, 0))
                        } else {
                            None
                        }
                    }
                    ModeOption::AtLeast(res) => {
                        if e.0.width >= res.w && e.0.height >= scaled_h {
                            Some((
                                e.0.width as i64 - res.w as i64,
                                e.0.height as i64 - scaled_h as i64,
                            ))
                        } else {
                            None
                        }
                    }
                    ModeOption::AtMost(res) => {
                        if res.w <= e.0.width && scaled_h <= e.0.height {
                            Some((
                                res.w as i64 - e.0.width as i64,
                                scaled_h as i64 - e.0.height as i64,
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
                            Ordering::Less => Some((&e.0, diff)),
                            Ordering::Greater => Some(best),
                            Ordering::Equal => match best.1.width.cmp(&res.0) {
                                Ordering::Less => Some((&e.0, diff)),
                                Ordering::Greater => Some(best),
                                Ordering::Equal => match best.1.pref.cmp(&e.1) {
                                    Ordering::Less => Some((&e.0, diff)),
                                    Ordering::Equal | Ordering::Greater => Some(best),
                                },
                            },
                        }
                    } else {
                        Some((&e.0, diff))
                    }
                } else {
                    acc
                }
            });

        Ok(best_mode.map(|m| m.0.xid))
    }

    // TODO::configurable timing methods
    /// Creates a new xrandr output mode based on the preference specification.
    fn create_preferred_mode(
        &mut self,
        output: &Output,
        native_ar: f32,
        modes: &[(Mode, bool)],
        pref: &ModePreference,
    ) -> Result<Mode> {
        log::debug!("Creating preferred mode for preference {pref:?}");

        let nearest_pref = ModePreference {
            aspect_ratio: AspectRatioOption::Any,
            resolution: match pref.resolution {
                ModeOption::Exact(res) | ModeOption::AtMost(res) => ModeOption::AtMost(res),
                ModeOption::AtLeast(_) => ModeOption::AtLeast(Resolution { w: 0, h: 0 }),
            },
            refresh: match pref.refresh {
                ModeOption::Exact(rr) | ModeOption::AtLeast(rr) => ModeOption::AtLeast(rr),
                ModeOption::AtMost(_) => ModeOption::AtLeast(0.),
            },
        };

        let screen = ScreenResources::new(&mut self.xhandle)?;
        let nearest = Self::get_preferred_mode(native_ar, modes, &nearest_pref)?.ok_or(
            anyhow::anyhow!("Unable to find acceptable mode for {nearest_pref:?} from {modes:?}"),
        )?;
        let nearest = screen.mode(nearest)?;

        let (width, height) = {
            let res = match pref.resolution {
                ModeOption::Exact(res) | ModeOption::AtMost(res) => Resolution {
                    w: res.w.min(nearest.width),
                    h: res.h.min(nearest.height),
                },
                ModeOption::AtLeast(_) => Resolution {
                    w: nearest.width,
                    h: nearest.height,
                },
            };
            let ar = res.w as f32 / res.h as f32;
            match pref.aspect_ratio {
                AspectRatioOption::Any => (res.w, res.h),
                AspectRatioOption::Native => {
                    if approx_eq!(f32, ar, native_ar, ulps = 2) {
                        (res.w, res.h)
                    } else {
                        (res.w, Self::scale_height(ar, native_ar, res.h))
                    }
                }
                AspectRatioOption::Exact(ex_ar) => {
                    if approx_eq!(f32, ar, ex_ar, ulps = 2) {
                        (res.w, res.h)
                    } else {
                        (res.w, Self::scale_height(ar, ex_ar, res.h))
                    }
                }
            }
        };

        let rate = match nearest_pref.refresh {
            ModeOption::Exact(_) | ModeOption::AtLeast(_) => nearest.rate,
            ModeOption::AtMost(rr) => rr,
        };

        // let rate = 30.;

        let timings = self.get_timings(width, height, rate)?;
        let mut cmd = Command::new("xrandr");

        let args = [
            vec!["--newmode".to_string(), timings.0.clone()],
            timings.1.clone(),
        ]
        .concat();

        log::debug!("creating new mode");

        let status = cmd.args(&args).status();

        if status.is_err() || !status.unwrap().success() {
            log::debug!("mode exists? removing and retrying");
            let mut del_cmd = Command::new("xrandr");
            del_cmd
                .args(["--delmode", &output.name, &timings.0])
                .status()?;

            let mut rm_cmd = Command::new("xrandr");

            let rm_status = rm_cmd.args(["--rmmode", &timings.0]).status();

            if rm_status.is_ok_and(|status| status.success()) {
                log::debug!("removed old mode; creating new one");

                let mut cmd = Command::new("xrandr");

                let status = cmd.args(&args).status()?;
                if !status.success() {
                    return Err(anyhow::anyhow!(
                        "failed to create new mode from `xrandr {:?}`",
                        args
                    ));
                }
            } else {
                return Err(anyhow::anyhow!("failed to remove old mode {}", &timings.0));
            }
        }

        let mut add_cmd = Command::new("xrandr");
        add_cmd
            .args(["--addmode", &output.name, &timings.0])
            .status()?;

        let resources = ScreenResources::new(&mut self.xhandle)?;
        let mode = resources
            .modes
            .iter()
            .find(|m| timings.0 == m.name)
            .ok_or(anyhow::anyhow!("newly create mode {} not found", timings.0))?;
        Ok(resources.mode(mode.xid)?)
    }

    // Ideally, we'd return the timing values and set them with xlib, but that isn't working,
    // so we return the mode name + modeline args instead
    fn get_timings(&self, width: u32, height: u32, refresh: f64) -> Result<(String, Vec<String>)> {
        let mut args = vec![width.to_string(), height.to_string(), refresh.to_string()];
        match self.timing_fallback {
            TimingFallbackMethod::CvtR => args.insert(0, "-r".to_string()),
            TimingFallbackMethod::Cvt => (),
        }

        let out = Command::new("cvt").args(args).output()?;
        if out.status.success() {
            let regex = Regex::new(r#"(?m)Modeline (?<name>"[\w|\.]+")\s+(?<dotclock>[\d|.]+)\s+(?<width>\d+)\s+(?<hsyncstart>\d+)\s+(?<hsyncend>\d+)\s+(?<htotal>\d+)\s+(?<height>\d+)\s+(?<vsyncstart>\d+)\s+(?<vsyncend>\d+)\s+(?<vtotal>\d+)\s+(?<flags>.*)"#).unwrap();
            let output = String::from_utf8_lossy(&out.stdout);

            let captures = regex.captures(&output).ok_or(anyhow::anyhow!(
                "could not get captures from timing output: `{output}`"
            ))?;

            fn get<T>(captures: &regex::Captures, name: &str) -> Result<T>
            where
                T: FromStr + Clone,
                <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
            {
                Ok(captures
                    .name(name)
                    .unwrap_or_else(|| panic!("expecting timing capture with name {name}"))
                    .as_str()
                    .parse::<T>()?
                    .clone())
            }

            Ok((
                get::<String>(&captures, "name")
                    .map(|name| format!("DeckDS-{}", &name[1..name.len() - 1].to_string()))?,
                captures
                    .get(0)
                    .expect("should have capture from cvt")
                    .as_str()
                    .strip_prefix("Modeline ")
                    .expect("cvt out should have modeline prefix")
                    .split_ascii_whitespace()
                    .map(|v| v.to_string())
                    .skip(1)
                    .collect(),
            ))

        //     let mut name: String = get(&captures, "name")?;
        //     let name_b = unsafe { name.as_bytes_mut() };
        //     let name_ptr = name_b.as_mut_ptr() as *mut i8;

        //     let flags = {
        //         let captured_flags: String = get(&captures, "flags")?;
        //         let ascii_flags = &captured_flags.to_ascii_lowercase();
        //         let flags: IndexMap<&str, i32, RandomState> = IndexMap::from_iter([
        //             ("+hsync", x11::xrandr::RR_HSyncPositive),
        //             ("-hsync", x11::xrandr::RR_HSyncNegative),
        //             ("+vsync", x11::xrandr::RR_VSyncPositive),
        //             ("-vsync", x11::xrandr::RR_VSyncNegative),
        //             ("interlace", x11::xrandr::RR_Interlace),
        //             ("doublescan", x11::xrandr::RR_DoubleScan),
        //             ("csync", x11::xrandr::RR_CSync),
        //             ("+csync", x11::xrandr::RR_CSyncPositive),
        //             ("-csync", x11::xrandr::RR_CSyncNegative),
        //         ]);

        //         flags.into_iter().fold(0u64, |acc, (k, v)| {
        //             if ascii_flags.contains(k) {
        //                 acc | v as u64
        //             } else {
        //                 acc
        //             }
        //         })
        //     };

        //     let clock:f64 = get(&captures, "dotclock")?;
        //     Ok(XRRModeInfo {
        //         id: 0,
        //         width,
        //         height,
        //         dotClock: (clock * 1e6 as f64).round() as u64,
        //         hSyncStart: get(&captures, "hsyncstart")?,
        //         hSyncEnd: get(&captures, "hsyncend")?,
        //         hTotal: get(&captures, "htotal")?,
        //         hSkew: 0, // xrandr doesn't seem to set this, so I'm assuming it defaults to 0
        //         vSyncStart: get(&captures, "vsyncstart")?,
        //         vSyncEnd: get(&captures, "vsyncend")?,
        //         vTotal: get(&captures, "vtotal")?,
        //         nameLength: name_b.len() as u32,
        //         name: name_ptr,
        //         modeFlags: flags,
        //     })
        } else {
            let stderr = out.stderr;
            Err(anyhow::anyhow!("{:?}", String::from_utf8_lossy(&stderr)))
        }
    }

    fn get_native_ar(
        // output: &Output,
        modes: &[(Mode, bool)],
    ) -> f32 {
        // let output_edid = output.edid();
        // let output_edid: Option<_> = output_edid
        //     .map(|e| {
        //         let edid_data = edid::parse(&e);
        //         let res = edid_data.to_result().ok();
        //         res.map(|r| r.display)
        //     })
        //     .flatten();

        let largest_mode = modes.iter().reduce(|a, b| {
            if a.0.width * a.0.height > b.0.width * b.0.height {
                a
            } else {
                b
            }
        });
        // output_edid
        //     .map(|e| e.width as f32 / e.height as f32)
        //     .unwrap_or(
        largest_mode
            .map(|m| m.0.width as f32 / m.0.height as f32)
            .unwrap_or(16. / 9.)
        // )
    }

    pub(crate) fn get_mode(&mut self, mode: u64) -> Result<Mode> {
        Ok(ScreenResources::new(&mut self.xhandle)?.mode(mode)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_mode(xid: u64, width: u32, height: u32, rate: f64, pref: bool) -> (Mode, bool) {
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

    const S9: f32 = 16. / 9.;
    const S10: f32 = 16. / 10.;

    #[test]
    fn test_get_preferred_mode_prf() -> Result<()> {
        let modes = vec![
            create_test_mode(1, 1280, 720, 60., true),
            create_test_mode(2, 1280, 720, 60., false),
        ];

        let mode = XDisplay::get_preferred_mode(
            S9,
            &modes,
            &ModePreference {
                resolution: ModeOption::Exact(Resolution { w: 1280, h: 720 }),
                aspect_ratio: AspectRatioOption::Exact(S9),
                refresh: ModeOption::Exact(60.),
            },
        )?;

        assert_eq!(Some(1), mode);
        Ok(())
    }

    #[test]
    fn test_get_preferred_mode_exact_rate() -> Result<()> {
        let modes = vec![
            create_test_mode(1, 1280, 720, 60., false),
            create_test_mode(2, 1280, 720, 61., false),
            create_test_mode(3, 1280, 720, 59., false),
        ];

        let mode = XDisplay::get_preferred_mode(
            S9,
            &modes,
            &ModePreference {
                resolution: ModeOption::Exact(Resolution { w: 1280, h: 720 }),
                aspect_ratio: AspectRatioOption::Exact(S9),
                refresh: ModeOption::Exact(60.),
            },
        )?;

        assert_eq!(Some(1), mode);
        Ok(())
    }

    #[test]
    fn test_get_preferred_mode_min_rate() -> Result<()> {
        let modes = vec![
            create_test_mode(1, 1280, 720, 15., false),
            create_test_mode(2, 1280, 720, 60., false),
            create_test_mode(3, 1280, 720, 30., false),
        ];

        let mode = XDisplay::get_preferred_mode(
            S9,
            &modes,
            &ModePreference {
                resolution: ModeOption::Exact(Resolution { w: 1280, h: 720 }),
                aspect_ratio: AspectRatioOption::Exact(S9),
                refresh: ModeOption::AtLeast(30.),
            },
        )?;
        assert_eq!(Some(2), mode);
        Ok(())
    }

    #[test]
    fn test_get_preferred_mode_max_rate() -> Result<()> {
        let modes = vec![
            create_test_mode(1, 1280, 720, 15., false),
            create_test_mode(2, 1280, 720, 60., false),
            create_test_mode(3, 1280, 720, 30., false),
        ];

        let mode = XDisplay::get_preferred_mode(
            S9,
            &modes,
            &ModePreference {
                resolution: ModeOption::Exact(Resolution { w: 1280, h: 720 }),
                aspect_ratio: AspectRatioOption::Exact(S9),
                refresh: ModeOption::AtMost(30.),
            },
        )?;
        assert_eq!(Some(3), mode);
        Ok(())
    }

    #[test]
    fn test_get_preferred_mode_exact_res_exact_ar() -> Result<()> {
        let modes = vec![
            create_test_mode(1, 1920, 1080, 60., false),
            create_test_mode(2, 1280, 720, 60., false),
            create_test_mode(3, 2560, 1440, 60., false),
        ];

        let mode = XDisplay::get_preferred_mode(
            S9,
            &modes,
            &ModePreference {
                resolution: ModeOption::Exact(Resolution { w: 1280, h: 720 }),
                aspect_ratio: AspectRatioOption::Exact(S9),
                refresh: ModeOption::Exact(60.),
            },
        )?;
        assert_eq!(Some(2), mode);
        Ok(())
    }

    #[test]
    fn test_get_preferred_mode_min_res_exact_ar() -> Result<()> {
        let modes = vec![
            create_test_mode(1, 1920, 1080, 60., false),
            create_test_mode(2, 1280, 720, 60., false),
            create_test_mode(3, 2560, 1440, 60., false),
        ];

        let mode = XDisplay::get_preferred_mode(
            S9,
            &modes,
            &ModePreference {
                resolution: ModeOption::AtLeast(Resolution { w: 1920, h: 1080 }),
                aspect_ratio: AspectRatioOption::Exact(S9),
                refresh: ModeOption::Exact(60.),
            },
        )?;
        assert_eq!(Some(3), mode);
        Ok(())
    }

    #[test]
    fn test_get_preferred_mode_max_res_exact_ar() -> Result<()> {
        let modes = vec![
            create_test_mode(1, 1920, 1080, 60., false),
            create_test_mode(2, 1280, 720, 60., false),
            create_test_mode(3, 2560, 1440, 60., false),
        ];

        let mode = XDisplay::get_preferred_mode(
            S9,
            &modes,
            &ModePreference {
                resolution: ModeOption::AtMost(Resolution { w: 1920, h: 1080 }),
                aspect_ratio: AspectRatioOption::Exact(S9),
                refresh: ModeOption::Exact(60.),
            },
        )?;
        assert_eq!(Some(1), mode);
        Ok(())
    }

    #[test]
    fn test_get_preferred_mode_exact_res_native_ar() -> Result<()> {
        let modes = vec![
            create_test_mode(1, 1920, 1080, 60., false),
            create_test_mode(2, 1280, 720, 60., false),
            create_test_mode(3, 1280, 800, 60., false),
            create_test_mode(4, 2560, 1440, 60., false),
        ];

        let mode = XDisplay::get_preferred_mode(
            S10,
            &modes,
            &ModePreference {
                resolution: ModeOption::Exact(Resolution { w: 1280, h: 720 }),
                aspect_ratio: AspectRatioOption::Native,
                refresh: ModeOption::Exact(60.),
            },
        )?;
        assert_eq!(Some(3), mode);
        Ok(())
    }

    #[test]
    fn test_get_preferred_mode_min_res_native_ar() -> Result<()> {
        let modes = vec![
            create_test_mode(1, 1920, 1200, 60., false),
            create_test_mode(2, 1280, 720, 60., false),
            create_test_mode(3, 1280, 800, 60., false),
            create_test_mode(4, 2560, 1440, 60., false),
        ];

        let mode = XDisplay::get_preferred_mode(
            S10,
            &modes,
            &ModePreference {
                resolution: ModeOption::AtLeast(Resolution { w: 1280, h: 720 }),
                aspect_ratio: AspectRatioOption::Native,
                refresh: ModeOption::Exact(60.),
            },
        )?;
        assert_eq!(Some(1), mode);
        Ok(())
    }

    #[test]
    fn test_get_preferred_mode_max_res_native_ar() -> Result<()> {
        let modes = vec![
            create_test_mode(1, 1920, 1200, 60., false),
            create_test_mode(2, 1280, 720, 60., false),
            create_test_mode(3, 1280, 800, 60., false),
            create_test_mode(4, 2560, 1440, 60., false),
        ];

        let mode = XDisplay::get_preferred_mode(
            S10,
            &modes,
            &ModePreference {
                resolution: ModeOption::AtMost(Resolution { w: 1280, h: 720 }),
                aspect_ratio: AspectRatioOption::Native,
                refresh: ModeOption::Exact(60.),
            },
        )?;
        assert_eq!(Some(3), mode);
        Ok(())
    }

    #[test]
    fn test_get_native_ar() {
        let modes = vec![
            create_test_mode(1, 1920, 1200, 60., false),
            create_test_mode(2, 1280, 720, 60., false),
            create_test_mode(3, 1280, 800, 60., false),
            create_test_mode(4, 2560, 1440, 60., false),
        ];

        let actual = XDisplay::get_native_ar(&modes);
        assert_eq!(S9, actual)
    }

    #[test]
    fn test_scale_height_s9_s10() {
        let actual = XDisplay::scale_height(S9, S10, 720);
        assert_eq!(800, actual)
    }

    #[test]
    fn test_scale_height_s10_s9() {
        let actual = XDisplay::scale_height(S10, S9, 800);
        assert_eq!(720, actual)
    }
}
