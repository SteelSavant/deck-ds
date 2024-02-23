use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    pipeline::{dependency::Dependency, executor::PipelineContext},
    sys::kwin::KWin,
};

use super::{ActionId, ActionImpl, ActionType};
use smart_default::SmartDefault;

const SCRIPT: &str = "emulatorwindowing";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MultiWindow {
    pub id: ActionId,
    pub general: GeneralOptions,
    /// Some(options) if Cemu is configurable, None otherwise
    pub cemu: Option<CemuOptions>,
    /// Some(options) if Citra is configurable, None otherwise
    pub citra: Option<CitraOptions>,
    /// Some(options) if Dolphin is configurable, None otherwise
    pub dolphin: Option<DolphinOptions>,
}

trait OptionsRW {
    fn load(kwin: &KWin) -> Result<Self>
    where
        Self: Sized;
    fn write(&self, kwin: &KWin) -> Result<()>;
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum LimitedMultiWindowLayout {
    #[default]
    ColumnRight,
    ColumnLeft,
    SquareRight,
    SquareLeft,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum MultiWindowLayout {
    ColumnRight,
    ColumnLeft,
    SquareRight,
    SquareLeft,
    #[default]
    Separate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MultiWindowOptions {
    pub enabled: bool,
    pub general: GeneralOptions,
    pub cemu: CemuOptions,
    pub citra: CitraOptions,
    pub dolphin: DolphinOptions,
}

impl OptionsRW for MultiWindowOptions {
    fn load(kwin: &KWin) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            enabled: kwin.get_script_enabled(SCRIPT)?,
            general: GeneralOptions::load(kwin)?,
            cemu: CemuOptions::load(kwin)?,
            citra: CitraOptions::load(kwin)?,
            dolphin: DolphinOptions::load(kwin)?,
        })
    }

    fn write(&self, kwin: &KWin) -> Result<()> {
        self.general.write(kwin)?;
        self.cemu.write(kwin)?;
        self.citra.write(kwin)?;
        self.dolphin.write(kwin)?;

        kwin.set_script_enabled(SCRIPT, self.enabled)?;

        Ok(())
    }
}

#[derive(Debug, Clone, SmartDefault, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct GeneralOptions {
    pub swap_screens: bool,
    #[default(true)]
    pub keep_above: bool,
}

impl OptionsRW for GeneralOptions {
    fn load(kwin: &KWin) -> Result<Self>
    where
        Self: Sized,
    {
        let swap_screens = kwin
            .get_script_bool_setting(SCRIPT, "swapScreens")?
            .unwrap_or(false);
        let keep_above = kwin
            .get_script_bool_setting(SCRIPT, "keepAbove")?
            .unwrap_or(true);

        Ok(Self {
            swap_screens,
            keep_above,
        })
    }

    fn write(&self, kwin: &KWin) -> Result<()> {
        kwin.set_script_bool_setting(SCRIPT, "swapScreens", self.swap_screens)?;
        kwin.set_script_bool_setting(SCRIPT, "keepAbove", self.keep_above)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CemuOptions {
    pub single_screen_layout: LimitedMultiWindowLayout,
    pub multi_screen_layout: MultiWindowLayout,
}

impl OptionsRW for CemuOptions {
    fn load(kwin: &KWin) -> Result<Self>
    where
        Self: Sized,
    {
        let single_screen_layout = kwin
            .get_script_string_setting(SCRIPT, "cemuSingleScreenLayout")?
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or(LimitedMultiWindowLayout::ColumnRight);
        let multi_screen_layout = kwin
            .get_script_string_setting(SCRIPT, "cemuMultiScreenSingleSecondaryLayout")?
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or(MultiWindowLayout::Separate);

        Ok(Self {
            single_screen_layout,
            multi_screen_layout,
        })
    }

    fn write(&self, kwin: &KWin) -> Result<()> {
        kwin.set_script_string_setting(
            SCRIPT,
            "cemuSingleScreenLayout",
            &serde_json::to_string(&self.single_screen_layout)?,
        )?;

        kwin.set_script_string_setting(
            SCRIPT,
            "cemuMultiScreenSingleSecondaryLayout",
            &serde_json::to_string(&self.multi_screen_layout)?,
        )?;

        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CitraOptions {
    pub single_screen_layout: LimitedMultiWindowLayout,
    pub multi_screen_layout: MultiWindowLayout,
}

impl OptionsRW for CitraOptions {
    fn load(kwin: &KWin) -> Result<Self>
    where
        Self: Sized,
    {
        let single_screen_layout = kwin
            .get_script_string_setting(SCRIPT, "citraSingleScreenLayout")?
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or(LimitedMultiWindowLayout::ColumnRight);
        let multi_screen_layout = kwin
            .get_script_string_setting(SCRIPT, "citraMultiScreenSingleSecondaryLayout")?
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or(MultiWindowLayout::Separate);

        Ok(Self {
            single_screen_layout,
            multi_screen_layout,
        })
    }

    fn write(&self, kwin: &KWin) -> Result<()> {
        kwin.set_script_string_setting(
            SCRIPT,
            "citraSingleScreenLayout",
            &serde_json::to_string(&self.single_screen_layout)?,
        )?;

        kwin.set_script_string_setting(
            SCRIPT,
            "citraMultiScreenSingleSecondaryLayout",
            &serde_json::to_string(&self.multi_screen_layout)?,
        )?;

        Ok(())
    }
}

#[derive(Debug, Clone, SmartDefault, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DolphinOptions {
    pub single_screen_layout: LimitedMultiWindowLayout,
    pub multi_screen_single_secondary_layout: MultiWindowLayout,
    #[default(MultiWindowLayout::ColumnRight)]
    pub multi_screen_multi_secondary_layout: MultiWindowLayout,
    // GBA ids to blacklist, 1,2,3 or 4
    pub gba_blacklist: Vec<u8>,
}

impl OptionsRW for DolphinOptions {
    fn load(kwin: &KWin) -> Result<Self>
    where
        Self: Sized,
    {
        let single_screen_layout = kwin
            .get_script_string_setting(SCRIPT, "dolphinSingleScreenLayout")?
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or(LimitedMultiWindowLayout::ColumnRight);
        let multi_screen_single_secondary_layout = kwin
            .get_script_string_setting(SCRIPT, "dolphinMultiScreenSingleSecondaryLayout")?
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or(MultiWindowLayout::Separate);
        let multi_screen_multi_secondary_layout = kwin
            .get_script_string_setting(SCRIPT, "dolphinMultiScreenMultiSecondaryLayout")?
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or(MultiWindowLayout::ColumnRight);

        let gba_blacklist = kwin
            .get_script_string_setting(SCRIPT, "dolphinBlacklist")?
            .map(|s| {
                s.split(',')
                    .filter_map(|v| {
                        let trimmed = v.trim().to_ascii_uppercase();
                        if trimmed.starts_with("GBA") && trimmed.len() == 4 {
                            trimmed.chars().last().unwrap().to_string().parse().ok()
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(Self {
            single_screen_layout,
            multi_screen_single_secondary_layout,
            multi_screen_multi_secondary_layout,
            gba_blacklist,
        })
    }

    fn write(&self, kwin: &KWin) -> Result<()> {
        kwin.set_script_string_setting(
            SCRIPT,
            "dolphinSingleScreenLayout",
            &serde_json::to_string(&self.single_screen_layout)?,
        )?;

        kwin.set_script_string_setting(
            SCRIPT,
            "dolphinMultiScreenSingleSecondaryLayout",
            &serde_json::to_string(&self.multi_screen_single_secondary_layout)?,
        )?;

        kwin.set_script_string_setting(
            SCRIPT,
            "dolphinMultiScreenMultiSecondaryLayout",
            &serde_json::to_string(&self.multi_screen_multi_secondary_layout)?,
        )?;

        kwin.set_script_string_setting(
            SCRIPT,
            "dolphinBlacklist",
            &self
                .gba_blacklist
                .iter()
                .map(|v| format!("GBA{v}"))
                .collect::<Vec<_>>()
                .join(","),
        )?;

        Ok(())
    }
}

impl ActionImpl for MultiWindow {
    type State = MultiWindowOptions;

    const TYPE: ActionType = ActionType::MultiWindow;

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        let mut options = MultiWindowOptions::load(&ctx.kwin)?;

        ctx.set_state::<Self>(options.clone());

        options.enabled = true;
        options.general = self.general.clone();

        if let Some(cemu) = self.cemu.clone() {
            options.cemu = cemu;
        }

        if let Some(citra) = self.citra.clone() {
            options.citra = citra;
        }

        if let Some(dolphin) = self.dolphin.clone() {
            options.dolphin = dolphin;
        }

        options.write(&ctx.kwin)
    }

    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()> {
        let state = ctx.get_state::<Self>();
        if let Some(state) = state {
            state.write(&ctx.kwin)
        } else {
            Ok(())
        }
    }

    fn get_dependencies(&self, _ctx: &mut PipelineContext) -> Vec<Dependency> {
        vec![Dependency::KwinScript(SCRIPT.to_string())]
    }

    #[inline]
    fn get_id(&self) -> ActionId {
        self.id
    }
}
