use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use xrandr::Relation;

use crate::pipeline::{
    dependency::{emulator_windowing::EmulatorWindowing, DependencyId},
    executor::PipelineContext,
};

use super::{PipelineActionId, PipelineActionImpl};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MultiWindow;

impl PipelineActionImpl for MultiWindow {
    type State = ();

    fn id(&self) -> PipelineActionId {
        PipelineActionId::parse("41354511-6dac-4d2d-b523-1dbcb4642562")
    }

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        ctx.kwin.set_script_enabled("EmulatorWindowing", true)?;
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
        ctx.kwin.set_script_enabled("EmulatorWindowing", false)?;
        Ok(())
    }

    fn get_dependencies(&self) -> Vec<DependencyId> {
        vec![EmulatorWindowing::id()]
    }
}
