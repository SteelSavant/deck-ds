use std::path::Path;

use crate::pipeline::executor::PipelineContext;

use self::internal::RawMelonDSState;

use super::{source_file::SourceFile, ActionId, ActionImpl};
use anyhow::{anyhow, Context, Result};
use configparser::ini::Ini;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(test)]
pub use internal::MelonDSLayoutState;

/// melonDS layout options. Because of the "unique" way melonDS handles
/// layouts, these options do not map 1:1.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum MelonDSLayoutOption {
    Natural,    // Puts screens vertical normally, horizonal in book mode.
    Vertical,   // Puts screens vertical always,
    Horizontal, // Puts screens horizonal always,
    Hybrid,     // Puts main screen large, with both screens adjacent. Overrides sizing settings.
    Single,     // Displays only one screen,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum MelonDSSizingOption {
    Even,
    EmphasizeTop,
    EmphasizeBottom,
    Auto,
}

impl MelonDSSizingOption {
    fn raw(&self) -> u64 {
        match self {
            MelonDSSizingOption::Even => 0,
            MelonDSSizingOption::EmphasizeTop => 1,
            MelonDSSizingOption::EmphasizeBottom => 2,
            MelonDSSizingOption::Auto => 3,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MelonDSLayout {
    pub id: ActionId,

    pub layout_option: MelonDSLayoutOption,
    pub sizing_option: MelonDSSizingOption,
    pub book_mode: bool, // if in book mode, set rotation to 270,
    pub swap_screens: bool,
}

mod internal {
    use std::path::PathBuf;

    use serde::{Deserialize, Serialize};

    #[cfg_attr(test, derive(Default))]
    #[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
    pub struct MelonDSLayoutState {
        pub layout: RawMelonDSState,
        pub ini_path: PathBuf,
    }

    #[cfg_attr(test, derive(Default))]
    #[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq)]
    pub struct RawMelonDSState {
        pub layout_option: u64,
        pub sizing_option: u64,
        pub rotation: u64,
        pub swap_screens: u64,
    }
}

impl From<MelonDSLayout> for RawMelonDSState {
    fn from(value: MelonDSLayout) -> Self {
        match (
            value.layout_option,
            value.sizing_option,
            value.book_mode,
            value.swap_screens,
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
                    layout_option: 1,
                    sizing_option: sizing.raw(),
                    rotation: if book_mode { 3 } else { 0 },
                    swap_screens: if swap_screens { 1 } else { 0 },
                }
            }
            (MelonDSLayoutOption::Horizontal, sizing, book_mode, swap_screens) => {
                internal::RawMelonDSState {
                    layout_option: 2,
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
        }
    }
}

impl internal::RawMelonDSState {
    fn read<P: AsRef<Path>>(ini_path: P) -> Result<Self> {
        let mut ini = Ini::new();
        let map = ini.load(&ini_path).map_err(|err| {
            anyhow!(
                "failed to load ini at {}: {err}",
                ini_path.as_ref().display()
            )
        })?;
        println!("map: {:?}", map);
        let section = "default";

        let layout = ini
            .getuint(section, "ScreenLayout")
            .map_err(|err| anyhow!(err))?
            .with_context(|| "key 'ScreenLayout' not found")?;
        let sizing = ini
            .getuint(section, "ScreenSizing")
            .map_err(|err| anyhow!(err))?
            .with_context(|| "key 'ScreenSizing' not found")?;

        let swap = ini
            .getuint(section, "ScreenSwap")
            .map_err(|err| anyhow!(err))?
            .with_context(|| "key 'ScreenSwap' not found")?;
        let rot = ini
            .getuint(section, "ScreenRotation")
            .map_err(|err| anyhow!(err))?
            .with_context(|| "key 'ScreenRotation' not found")?;

        Ok(Self {
            layout_option: layout,
            sizing_option: sizing,
            swap_screens: swap,
            rotation: rot,
        })
    }

    fn write<P: AsRef<Path>>(&self, ini_path: P) -> Result<()> {
        let mut ini = Ini::new_cs();
        ini.load(&ini_path).map_err(|err| {
            anyhow!(
                "failed to load ini at {}: {err}",
                ini_path.as_ref().display()
            )
        })?;

        let section = "default";

        ini.set(
            section,
            "ScreenLayout",
            Some(self.layout_option.to_string()),
        );
        ini.set(
            section,
            "ScreenSizing",
            Some(self.sizing_option.to_string()),
        );
        ini.set(section, "ScreenRotation", Some(self.rotation.to_string()));
        ini.set(section, "ScreenSwap", Some(self.swap_screens.to_string()));

        Ok(ini.write(ini_path)?)
    }
}

impl ActionImpl for MelonDSLayout {
    type State = internal::MelonDSLayoutState;

    const NAME: &'static str = "MelonDSLayout";

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        let ini_path = ctx
            .get_state::<SourceFile>()
            .with_context(|| "No source file set for melonDS settings")?;

        let current = internal::MelonDSLayoutState {
            layout: internal::RawMelonDSState::read(ini_path)?,
            ini_path: ini_path.clone(),
        };

        let raw: RawMelonDSState = (*self).into();

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

    #[inline]
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
    fn test_read_write_melonds_layout() -> Result<()> {
        let source_path = "test/assets/melonds/melonDS.ini";
        let source = std::fs::read_to_string(source_path)?;
        let path = PathBuf::from("test/out/melonds/melonDS.ini");
        create_dir_all(path.parent().unwrap())?;

        std::fs::write(&path, &source)?;

        let initial = internal::RawMelonDSState {
            layout_option: 2,
            sizing_option: 0,
            rotation: 0,
            swap_screens: 0,
        };

        let expected = initial;
        let actual = internal::RawMelonDSState::read(&path)?;

        assert_eq!(expected, actual);

        expected.write(&path)?;
        let actual_str = std::fs::read_to_string(&path)?;
        assert_eq!(source, actual_str);

        let expected: internal::RawMelonDSState = MelonDSLayout {
            id: ActionId::new(), // ignored in conversion to raw state
            layout_option: MelonDSLayoutOption::Hybrid,
            sizing_option: MelonDSSizingOption::Even,
            book_mode: true,
            swap_screens: true,
        }
        .into();
        expected.write(&path)?;

        let actual = internal::RawMelonDSState::read(&path)?;

        assert_eq!(expected, actual);

        std::fs::remove_file(path)?;
        Ok(())
    }
}
