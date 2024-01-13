use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use xrandr::Relation;

use crate::pipeline::{dependency::Dependency, executor::PipelineContext};

use super::{ActionId, ActionImpl};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MultiWindow {
    pub id: ActionId,
}

impl ActionImpl for MultiWindow {
    type State = ();

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        ctx.kwin.set_script_enabled("emulatorwindowing", true)?;
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

        display.set_output_position(&deck, &Relation::Below, &external)
    }

    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()> {
        ctx.kwin.set_script_enabled("emulatorwindowing", false)?;
        Ok(())
    }

    fn get_dependencies(&self, _ctx: &mut PipelineContext) -> Vec<Dependency> {
        vec![Dependency::KwinScript(
            "emulatorwindowing-v1.0.kwinscript".to_string(),
        )]
    }

    fn get_id(&self) -> ActionId {
        self.id
    }
}
