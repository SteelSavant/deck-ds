use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pipeline::{dependency::Dependency, executor::PipelineContext};

use super::{ActionId, ActionImpl};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum MultiWindowTarget {
    Cemu,
    Citra,
    Dolphin,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MultiWindow {
    pub id: ActionId,
    /// Which applications are the intended targets;
    /// will eventually be used to choose which UI
    /// to display to configure the KWin script
    pub targets: Vec<MultiWindowTarget>,
}

const SCRIPT: &str = "emulatorwindowing";

impl ActionImpl for MultiWindow {
    type State = bool;

    const NAME: &'static str = "MultiWindow";

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        let enabled = ctx.kwin.get_script_enabled(SCRIPT);
        ctx.set_state::<Self>(matches!(enabled, Ok(true)));

        ctx.kwin.set_script_enabled(SCRIPT, true)
    }

    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()> {
        let state = ctx.get_state::<Self>();
        ctx.kwin
            .set_script_enabled(SCRIPT, matches!(state, Some(true)))
    }

    fn get_dependencies(&self, _ctx: &mut PipelineContext) -> Vec<Dependency> {
        vec![Dependency::KwinScript(SCRIPT.to_string())]
    }

    #[inline]
    fn get_id(&self) -> ActionId {
        self.id
    }
}
