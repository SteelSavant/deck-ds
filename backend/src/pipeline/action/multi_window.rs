use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use xrandr::Relation;

use crate::pipeline::{dependency::Dependency, executor::PipelineContext};

use super::PipelineActionImpl;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MultiWindow;

impl PipelineActionImpl for MultiWindow {
    type State = ();

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        ctx.kwin.set_script_enabled("emulatorwindowing", true)?;
        let external = ctx
            .display
            .get_preferred_external_output()?
            .ok_or(anyhow::anyhow!("Failed to find external display"))?;
        let deck = ctx
            .display
            .get_embedded_output()?
            .ok_or(anyhow::anyhow!("Failed to find embedded display"))?;

        ctx.display
            .set_output_position(&deck, &Relation::Below, &external)
    }

    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()> {
        ctx.kwin.set_script_enabled("emulatorwindowing", false)?;
        Ok(())
    }

    fn get_dependencies(&self) -> Vec<Dependency> {
        vec![Dependency::KwinScript(
            "emulatorwindowing-v1.0.kwinscript".to_string(),
        )]
    }
}
