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

use super::{PipelineActionId, PipelineActionImpl};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VirtualScreen;

impl PipelineActionImpl for VirtualScreen {
    type State = ();

    fn id(&self) -> PipelineActionId {
        PipelineActionId::parse("8bc7b827-1c31-41c2-a807-dc1e99f85922")
    }

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
        let res = if deck_mode.width < deck_mode.height {
            Resolution {
                h: deck_mode.width,
                w: deck_mode.height,
            }
        } else {
            Resolution {
                w: deck_mode.width,
                h: deck_mode.height,
            }
        };

        ctx.display.set_or_create_preferred_mode(
            &external,
            &ModePreference {
                resolution: ModeOption::Exact(res),
                aspect_ratio: AspectRatioOption::Exact(res.w as f32 / res.h as f32),
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
