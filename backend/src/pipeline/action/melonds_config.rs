use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use crate::pipeline::executor::PipelineContext;

use super::PipelineActionImpl;
use anyhow::Result;
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
pub enum MelonDSIniSource {
    Flatpak,
    Custom(PathBuf),
    // AppImage (if one exists)
}

impl MelonDSIniSource {
    pub fn get_path<P: AsRef<Path>>(&self, home_dir: P) -> Result<Cow<PathBuf>> {
        Ok(match self {
            MelonDSIniSource::Flatpak => Cow::Owned(
                home_dir
                    .as_ref()
                    .join(".var/app/net.kuribo64.melonDS/config/melonDS/melonDS.ini"),
            ),
            MelonDSIniSource::Custom(path) => Cow::Borrowed(path),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MelonDSConfig {
    pub ini_source: MelonDSIniSource,
    pub layout_option: MelonDSLayoutOption,
    pub sizing_option: MelonDSSizingOption,
    pub book_mode: bool, // if in book mode, set rotation to 270,
    pub swap_screens: bool,
}

mod internal {
    #[derive(Debug, Copy, Clone)]
    pub struct RawMelonDSState {
        pub layout_option: u8,
        pub sizing_option: u8,
        pub rotation: u8,
        pub swap_screens: u8,
    }
}

impl internal::RawMelonDSState {
    fn read(section: &Properties) -> Result<Self> {
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

    fn write<P: AsRef<Path>>(&self, path: P, mut ini: Ini) -> Result<()> {
        let layout_section = ini.general_section_mut();

        layout_section.insert("ScreenLayout", self.layout_option.to_string());
        layout_section.insert("ScreenSizing", self.sizing_option.to_string());
        layout_section.insert("ScreenRotation", self.rotation.to_string());
        layout_section.insert("ScreenSwap", self.swap_screens.to_string());

        Ok(ini.write_to_file(path)?)
    }
}

impl PipelineActionImpl for MelonDSConfig {
    type State = internal::RawMelonDSState;

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        let ini_path = self.get_ini_path(&ctx.home_dir)?;

        let mut ini = Ini::load_from_file(ini_path.as_path())?;
        let layout_section = ini.general_section_mut();
        let current = internal::RawMelonDSState::read(layout_section)?;
        ctx.set_state::<Self>(current);

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

        raw.write(ini_path.as_path(), ini)
    }

    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()> {
        let state = ctx.get_state::<Self>();

        match state {
            Some(state) => {
                let ini_path = self.get_ini_path(&ctx.home_dir)?;
                let ini = Ini::load_from_file(ini_path.as_path())?;

                state.write(ini_path.as_path(), ini)
            }
            None => Ok(()),
        }
    }
}

impl MelonDSConfig {
    fn get_ini_path<P: AsRef<Path>>(&self, home_dir: P) -> Result<Cow<PathBuf>> {
        let ini_path = self.ini_source.get_path(home_dir)?;
        if ini_path.is_file() {
            Ok(ini_path)
        } else {
            Err(std::io::Error::from(std::io::ErrorKind::NotFound).into())
        }
    }
}
