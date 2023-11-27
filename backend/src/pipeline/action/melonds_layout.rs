use std::path::Path;

use crate::pipeline::executor::PipelineContext;

use super::{source_file::SourceFile, ActionImpl};
use anyhow::{Context, Result};
use ini::{Ini, Properties};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// melonDS layout options. Because of the "unique" way melonDS handles
/// layouts, these options do not map 1:1.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema)]
pub enum MelonDSLayoutOption {
    Natural,    // Puts screens vertical normally, horizonal in book mode.
    Vertical,   // Puts screens vertical always,
    Horizontal, // Puts screens horizonal always,
    Hybrid,     // Puts main screen large, with both screens adjacent. Overrides sizing settings.
    Single,     // Displays only one screen,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema)]
pub enum MelonDSSizingOption {
    Even,
    EmphasizeTop,
    EmphasizeBottom,
    Auto,
}

impl MelonDSSizingOption {
    fn raw(&self) -> u8 {
        match self {
            MelonDSSizingOption::Even => 0,
            MelonDSSizingOption::EmphasizeTop => 1,
            MelonDSSizingOption::EmphasizeBottom => 2,
            MelonDSSizingOption::Auto => 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MelonDSLayout {
    pub layout_option: MelonDSLayoutOption,
    pub sizing_option: MelonDSSizingOption,
    pub book_mode: bool, // if in book mode, set rotation to 270,
    pub swap_screens: bool,
}

mod internal {
    use std::path::PathBuf;

    pub struct MelonDSLayoutState {
        pub layout: RawMelonDSState,
        pub ini_path: PathBuf,
    }

    #[derive(Debug, Clone)]
    pub struct RawMelonDSState {
        pub layout_option: u8,
        pub sizing_option: u8,
        pub rotation: u8,
        pub swap_screens: u8,
    }
}

impl internal::RawMelonDSState {
    fn read<P: AsRef<Path>>(ini_path: P) -> Result<Self> {
        let mut ini = Ini::load_from_file(ini_path)?;
        let section = ini.general_section_mut();

        let layout = section
            .get("ScreenLayout")
            .expect("ScreenLayout should exist");
        let sizing = section
            .get("ScreenSizing")
            .expect("ScreenSizing should exist");
        let swap = section.get("ScreenSwap").expect("ScreenSwap should exist");
        let rot = section
            .get("ScreenRotation")
            .expect("ScreenRotation should exist");

        Ok(Self {
            layout_option: layout.parse()?,
            sizing_option: sizing.parse()?,
            swap_screens: swap.parse()?,
            rotation: rot.parse()?,
        })
    }

    fn write<P: AsRef<Path>>(&self, ini_path: P) -> Result<()> {
        let mut ini = Ini::load_from_file(&ini_path)?;

        let section = ini.general_section_mut();

        section.insert("ScreenLayout", self.layout_option.to_string());
        section.insert("ScreenSizing", self.sizing_option.to_string());
        section.insert("ScreenRotation", self.rotation.to_string());
        section.insert("ScreenSwap", self.swap_screens.to_string());

        Ok(ini.write_to_file(ini_path)?)
    }
}

impl ActionImpl for MelonDSLayout {
    type State = internal::MelonDSLayoutState;

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        let ini_path = ctx
            .get_state::<SourceFile>()
            .with_context(|| "No source file set for melonDS settings")?;

        let current = internal::MelonDSLayoutState {
            layout: internal::RawMelonDSState::read(ini_path)?,
            ini_path: ini_path.clone(),
        };

        let raw = match (
            self.layout_option,
            self.sizing_option,
            self.book_mode,
            self.swap_screens,
        ) {
            (MelonDSLayoutOption::Natural, sizing, book_mode, swap_screens) => {
                internal::RawMelonDSState {
                    layout_option: 0,
                    sizing_option: sizing.raw(),
                    rotation: if book_mode { 3 } else { 0 },
                    swap_screens: if swap_screens { 1 } else { 0 },
                }
            }
            (MelonDSLayoutOption::Vertical, sizing, book_mode, swap_screens) => {
                internal::RawMelonDSState {
                    layout_option: 0,
                    sizing_option: sizing.raw(),
                    rotation: if book_mode { 3 } else { 0 },
                    swap_screens: if swap_screens { 1 } else { 0 },
                }
            }
            (MelonDSLayoutOption::Horizontal, sizing, book_mode, swap_screens) => {
                internal::RawMelonDSState {
                    layout_option: 0,
                    sizing_option: sizing.raw(),
                    rotation: if book_mode { 3 } else { 0 },
                    swap_screens: if swap_screens { 1 } else { 0 },
                }
            }
            (MelonDSLayoutOption::Hybrid, _, book_mode, swap_screens) => {
                internal::RawMelonDSState {
                    layout_option: 3,
                    sizing_option: 0,
                    rotation: if book_mode { 3 } else { 0 },
                    swap_screens: if swap_screens { 1 } else { 0 },
                }
            }
            (MelonDSLayoutOption::Single, _, book_mode, swap_screens) => {
                internal::RawMelonDSState {
                    layout_option: 0,
                    sizing_option: 4,
                    rotation: if book_mode { 3 } else { 0 },
                    swap_screens: if swap_screens { 1 } else { 0 },
                }
            }
        };

        raw.write(ini_path).map(|_| {
            ctx.set_state::<Self>(current);
        })
    }

    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()> {
        let state = ctx.get_state::<Self>();

        match state {
            Some(state) => state.layout.write(&state.ini_path),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_read_melonds_layout() {
        todo!()
    }

    #[test]
    fn test_write_melonds_layout() {
        todo!()
    }
}
