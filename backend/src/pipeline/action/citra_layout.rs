use std::path::Path;

use crate::pipeline::executor::PipelineContext;

use super::{source_file::SourceFile, ActionImpl};
use anyhow::{Context, Result};
use ini::Ini;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "value")]
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
pub struct CitraLayout {
    pub layout_option: CitraLayoutOption,
    pub swap_screens: bool,
}

impl CitraLayout {
    fn read<P: AsRef<Path>>(ini_path: P) -> Result<Self> {
        let mut ini = Ini::load_from_file(ini_path)?;

        // TODO::the Ini lib fails to load Citra's ini properly. Handle necessary fields with regex instead

        let layout_section = ini
            .section_mut(Some("Layout"))
            .expect("Layout section should exist");

        let layout = layout_section
            .get("layout_option")
            .expect("layout_option should exist");

        let raw = layout.parse()?;

        Ok(CitraLayout {
            layout_option: CitraLayoutOption::from_raw(raw),
            swap_screens: false, // TODO
        })
    }

    fn write<P: AsRef<Path>>(&self, ini_path: P) -> Result<()> {
        let mut ini = Ini::load_from_file(&ini_path)?;

        let layout_section = ini
            .section_mut(Some("Layout"))
            .expect("Layout section should exist");

        layout_section.insert("layout_option", self.layout_option.raw().to_string());

        Ok(ini.write_to_file(ini_path)?)
    }
}

mod internal {
    use std::path::PathBuf;

    use super::CitraLayout;

    pub struct CitraState {
        pub ini_path: PathBuf,
        pub layout: CitraLayout,
    }
}

impl ActionImpl for CitraLayout {
    type State = internal::CitraState;

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        let (ini_path, layout) = {
            let ini_path = ctx
                .get_state::<SourceFile>()
                .with_context(|| "No source file set for Citra settings")?;

            (ini_path.clone(), CitraLayout::read(ini_path)?)
        };

        self.write(&ini_path).map(|_| {
            ctx.set_state::<Self>(internal::CitraState {
                ini_path: ini_path,
                layout,
            });
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
    fn test_read_citra_layout() {
        todo!()
    }

    #[test]
    fn test_write_citra_layout() {
        todo!()
    }
}
