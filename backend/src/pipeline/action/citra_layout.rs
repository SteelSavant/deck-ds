use std::path::Path;

use crate::pipeline::executor::PipelineContext;

use super::{source_file::SourceFile, ActionId, ActionImpl};
use anyhow::{anyhow, Context, Result};
use configparser::ini::{Ini, IniDefault};
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "value")]
pub enum CitraLayoutOption {
    Default,         // 0
    SingleScreen,    // 1
    LargeScreen,     // 2
    SideBySide,      // 3
    SeparateWindows, // 4
    HybridScreen,    // 5
    Unknown(u64),
}

impl CitraLayoutOption {
    fn from_raw(raw: u64) -> Self {
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

    fn raw(&self) -> u64 {
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct CitraLayout {
    pub id: ActionId,
    pub layout: CitraLayoutState,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct CitraLayoutState {
    pub layout_option: CitraLayoutOption,
    pub swap_screens: bool,
    pub fullscreen: bool, // Setting this doesn't work for some reason...
}

impl CitraLayoutState {
    const LAYOUT_SECTION: &'static str = "Layout";
    const UI_SECTION: &'static str = "UI";

    fn read<P: AsRef<Path>>(ini_path: P) -> Result<Self> {
        let mut ini = Ini::new();
        ini.load(&ini_path).map_err(|err| {
            anyhow!(
                "failed to load ini at {}: {err}",
                ini_path.as_ref().display()
            )
        })?;

        let raw_layout = ini
            .getuint(Self::LAYOUT_SECTION, "layout_option")
            .map_err(|err| anyhow!(err))?
            .with_context(|| "key 'layout_option' not found")?;
        let swap_screens = ini
            .getbool(Self::LAYOUT_SECTION, "swap_screen")
            .map_err(|err| anyhow!(err))?
            .with_context(|| "key 'swap_screen' not found")?;

        let ui = Self::UI_SECTION;

        let fullscreen = ini
            .getbool(ui, "fullscreen")
            .map_err(|err| anyhow!(err))?
            .with_context(|| "key 'fullscreen' not found")?;

        Ok(CitraLayoutState {
            layout_option: CitraLayoutOption::from_raw(raw_layout),
            swap_screens,
            fullscreen,
        })
    }

    fn write<P: AsRef<Path>>(&self, ini_path: P) -> Result<()> {
        let mut defaults = IniDefault::default();
        defaults.case_sensitive = true;
        defaults.comment_symbols = vec![];
        defaults.delimiters = vec!['='];

        let mut ini = Ini::new_from_defaults(defaults);

        ini.load(&ini_path).map_err(|err| {
            anyhow!(
                "failed to load ini at {}: {err}",
                ini_path.as_ref().display()
            )
        })?;

        ini.set(
            Self::LAYOUT_SECTION,
            "layout_option",
            Some(self.layout_option.raw().to_string()),
        );
        ini.set(
            Self::LAYOUT_SECTION,
            "swap_screen",
            Some(self.swap_screens.to_string()),
        );

        ini.set(
            Self::UI_SECTION,
            "fullscreen",
            Some(self.fullscreen.to_string()),
        );

        let section_regex = Regex::new(r#"^\[(.*)\]\s*$"#).expect("regex should be valid");

        let fixed = ini
            .writes()
            .split_terminator('\n')
            .map(|line| section_regex.replace(line, "\n[$1]"))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(std::fs::write(ini_path.as_ref(), fixed.trim().as_bytes())?)
    }
}

mod internal {
    use std::path::PathBuf;

    use serde::{Deserialize, Serialize};

    use super::CitraLayoutState;

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct CitraState {
        pub ini_path: PathBuf,
        pub layout: CitraLayoutState,
    }
}

impl ActionImpl for CitraLayout {
    type State = internal::CitraState;

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        let (ini_path, layout) = {
            let ini_path = ctx
                .get_state::<SourceFile>()
                .with_context(|| "No source file set for Citra settings")?;

            (ini_path.clone(), CitraLayoutState::read(ini_path)?)
        };

        self.layout.write(&ini_path).map(|_| {
            ctx.set_state::<Self>(internal::CitraState { ini_path, layout });
        })
    }

    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()> {
        let state = ctx.get_state::<Self>();

        match state {
            Some(state) => state.layout.write(&state.ini_path),
            None => Ok(()),
        }
    }

    fn get_id(&self) -> ActionId {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::util::create_dir_all;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_read_write_citra_layout() -> Result<()> {
        let source_path = "test/assets/citra/qt-config.ini";
        let source = std::fs::read_to_string(source_path)?;
        let path = PathBuf::from("test/out/citra/qt-config.ini");
        create_dir_all(path.parent().unwrap())?;

        std::fs::write(&path, &source)?;

        let expected = CitraLayoutState {
            layout_option: CitraLayoutOption::Default,
            swap_screens: false,
            fullscreen: false,
        };
        let actual = CitraLayoutState::read(&path)?;

        assert_eq!(expected, actual);

        expected.write(&path)?;
        let actual_str = std::fs::read_to_string(&path)?;
        assert_eq!(source, actual_str);

        let expected = CitraLayoutState {
            layout_option: CitraLayoutOption::SeparateWindows,
            swap_screens: true,
            fullscreen: true,
        };

        expected.write(&path)?;

        let actual = CitraLayoutState::read(&path)?;

        assert_eq!(expected, actual);

        std::fs::remove_file(path)?;
        Ok(())
    }
}
