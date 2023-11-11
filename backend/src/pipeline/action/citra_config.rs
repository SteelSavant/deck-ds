use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use crate::pipeline::executor::PipelineContext;

use super::PipelineActionImpl;
use anyhow::Result;
use ini::Ini;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema)]
pub enum CitraLayoutOption {
    Default,         // 0
    SingleScreen,    // 1
    LargeScreen,     // 2
    SideBySide,      // 3
    SeparateWindows, // 4
    HybridScreen,    // 5
    Unknown(u8),
}

impl CitraLayoutOption {
    fn from_raw(raw: u8) -> Self {
        match raw {
            0 => CitraLayoutOption::Default,
            1 => CitraLayoutOption::SingleScreen,
            2 => CitraLayoutOption::LargeScreen,
            3 => CitraLayoutOption::SideBySide,
            4 => CitraLayoutOption::SeparateWindows,
            5 => CitraLayoutOption::HybridScreen,
            unknown => CitraLayoutOption::Unknown(unknown),
        }
    }

    fn raw(&self) -> u8 {
        match self {
            CitraLayoutOption::Default => 0,
            CitraLayoutOption::SingleScreen => 1,
            CitraLayoutOption::LargeScreen => 2,
            CitraLayoutOption::SideBySide => 3,
            CitraLayoutOption::SeparateWindows => 4,
            CitraLayoutOption::HybridScreen => 5,
            CitraLayoutOption::Unknown(value) => *value,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum CitraIniSource {
    Flatpak,
    Custom(PathBuf),
    // AppImage (if one exists)
}

impl CitraIniSource {
    pub fn get_path<P: AsRef<Path>>(&self, home_dir: P) -> Result<Cow<PathBuf>> {
        Ok(match self {
            CitraIniSource::Flatpak => Cow::Owned(
                home_dir
                    .as_ref()
                    .join(".var/app/org.citra_emu.citra/config/citra-emu/qt-config.ini"),
            ),
            CitraIniSource::Custom(path) => Cow::Borrowed(path),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CitraConfig {
    pub ini_source: CitraIniSource,
    pub layout_option: CitraLayoutOption,
    // TODO:: pub swap_screens: bool,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CitraState {
    pub layout_option: CitraLayoutOption,
}

impl PipelineActionImpl for CitraConfig {
    type State = CitraState;

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        let ini_path = self.get_ini_path(&ctx.home_dir)?;
        let mut ini = Ini::load_from_file(ini_path.as_path())?;

        let layout_section = ini
            .section_mut(Some("Layout"))
            .expect("Layout section should exist");

        let layout = layout_section
            .get("layout_option")
            .expect("layout_option should exist");

        let raw = layout.parse()?;

        ctx.set_state::<Self>(CitraState {
            layout_option: CitraLayoutOption::from_raw(raw),
        });

        layout_section.insert("layout_option", self.layout_option.raw().to_string());

        Ok(ini.write_to_file(ini_path.as_path())?)
    }

    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()> {
        let state = ctx.get_state::<Self>();

        match state {
            Some(state) => {
                let ini_path = self.get_ini_path(&ctx.home_dir)?;
                let mut ini = Ini::load_from_file(ini_path.as_path())?;

                let layout_section = ini
                    .section_mut(Some("Layout"))
                    .expect("Layout section should exist");

                layout_section.insert("layout_option", state.layout_option.raw().to_string());

                Ok(ini.write_to_file(ini_path.as_path())?)
            }
            None => Ok(()),
        }
    }
}

impl CitraConfig {
    fn get_ini_path<P: AsRef<Path>>(&self, home_dir: P) -> Result<Cow<PathBuf>> {
        let ini_path = self.ini_source.get_path(home_dir)?;
        if ini_path.is_file() {
            Ok(ini_path)
        } else {
            Err(std::io::Error::from(std::io::ErrorKind::NotFound).into())
        }
    }
}
