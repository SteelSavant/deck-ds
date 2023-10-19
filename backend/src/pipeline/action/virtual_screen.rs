use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use xrandr::Relation;

use crate::{
    pipeline::{
        dependency::{true_video_wall::TrueVideoWall, DependencyId},
        executor::PipelineContext,
    },
    sys::x_display::{AspectRatioOption, ModeOption, ModePreference, Resolution},
};

use super::PipelineActionExecutor;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VirtualScreen;

impl PipelineActionExecutor for VirtualScreen {
    type State = ();

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        ctx.kwin.set_script_enabled("TrueVideoWall", true)?;
        let external = ctx
            .display
            .get_preferred_external_output()?
            .ok_or(anyhow::anyhow!("Failed to find external display"))?;
        let deck = ctx
            .display
            .get_embedded_output()?
            .ok_or(anyhow::anyhow!("Failed to find embedded display"))?;

        let deck_mode = ctx
            .display
            .get_current_mode(&deck)?
            .expect("Deck screen should have active mode");

        ctx.display.set_or_create_preferred_mode(
            &external,
            &ModePreference {
                resolution: ModeOption::Exact(Resolution {
                    w: deck_mode.width,
                    h: deck_mode.height,
                }),
                aspect_ratio: AspectRatioOption::Native,
                refresh: ModeOption::Exact(deck_mode.rate),
            },
        )?;

        ctx.display
            .set_output_position(&deck, &Relation::Below, &external)
    }

    fn get_dependencies(&self) -> Vec<DependencyId> {
        vec![TrueVideoWall::id()]
    }
}
