use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use xrandr::{Output, Relation};

use crate::{
    pipeline::{
        action::session_handler::{Pos, Size},
        dependency::Dependency,
        executor::PipelineContext,
    },
    sys::x_display::XDisplay,
};

use super::{session_handler::UiEvent, ActionId, ActionImpl};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MultiWindow {
    pub id: ActionId,
}

const SCRIPT: &str = "emulatorwindowing";

// TODO::restore kwin script settings

impl ActionImpl for MultiWindow {
    type State = bool;

    const NAME: &'static str = "MultiWindow";

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        let enabled = ctx.kwin.get_script_enabled(SCRIPT);
        ctx.set_state::<Self>(matches!(enabled, Ok(true)));

        ctx.kwin.set_script_enabled(SCRIPT, true)?;
        let display = ctx
            .display
            .as_mut()
            .with_context(|| "MultiWindow requires x11 to be running")?;
        let external = display
            .get_preferred_external_output()?
            .ok_or(anyhow::anyhow!("Failed to find external display"))?;
        let deck = display
            .get_embedded_output()?
            .ok_or(anyhow::anyhow!("Failed to find embedded display"))?;

        let res = display.set_output_position(&deck, &Relation::Below, &external);

        res
    }

    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()> {
        ctx.kwin.set_script_enabled(SCRIPT, false)?;
        Ok(())
    }

    fn get_dependencies(&self, _ctx: &mut PipelineContext) -> Vec<Dependency> {
        vec![
            Dependency::KwinScript(SCRIPT.to_string()),
            Dependency::Display,
        ]
    }

    #[inline]
    fn get_id(&self) -> ActionId {
        self.id
    }
}
