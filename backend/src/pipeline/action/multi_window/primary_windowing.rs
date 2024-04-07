use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    pipeline::{dependency::Dependency, executor::PipelineContext},
    sys::kwin::KWin,
};

use super::{ActionId, ActionImpl, ActionType, OptionsRW, SCRIPT};
use smart_default::SmartDefault;

mod cemu_options;
mod citra_options;
mod custom_options;
mod dolphin_options;

pub use cemu_options::CemuWindowOptions;
pub use citra_options::CitraWindowOptions;
pub use custom_options::CustomWindowOptions;
pub use dolphin_options::DolphinWindowOptions;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MultiWindow {
    pub id: ActionId,
    pub general: GeneralOptions,
    /// Some(options) if Cemu is configurable, None otherwise
    pub cemu: Option<CemuWindowOptions>,
    /// Some(options) if Citra is configurable, None otherwise
    pub citra: Option<CitraWindowOptions>,
    /// Some(options) if Dolphin is configurable, None otherwise
    pub dolphin: Option<DolphinWindowOptions>,
    // /// Some(options) if Custom is configurable, None otherwise
    pub custom: Option<CustomWindowOptions>,
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
    pub cemu: CemuWindowOptions,
    pub citra: CitraWindowOptions,
    pub dolphin: DolphinWindowOptions,
    pub custom: CustomWindowOptions,
}

impl OptionsRW for MultiWindowOptions {
    fn load(kwin: &KWin) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            enabled: kwin.get_script_enabled(SCRIPT)?,
            general: GeneralOptions::load(kwin)?,
            cemu: CemuWindowOptions::load(kwin)?,
            citra: CitraWindowOptions::load(kwin)?,
            dolphin: DolphinWindowOptions::load(kwin)?,
            custom: CustomWindowOptions::load(kwin)?,
        })
    }

    fn write(&self, kwin: &KWin) -> Result<()> {
        self.general.write(kwin)?;
        self.cemu.write(kwin)?;
        self.citra.write(kwin)?;
        self.dolphin.write(kwin)?;
        self.custom.write(kwin)?;

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

        if let Some(custom) = self.custom.clone() {
            options.custom = custom;
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

    fn get_dependencies(&self, _ctx: &PipelineContext) -> Vec<Dependency> {
        vec![Dependency::KwinScript(SCRIPT.to_string())]
    }

    #[inline]
    fn get_id(&self) -> ActionId {
        self.id
    }
}
