use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use xrandr::Relation;

use crate::{
    pipeline::{dependency::Dependency, executor::PipelineContext},
    sys::x_display::{AspectRatioOption, ModeOption, ModePreference, Resolution},
};

use super::ActionImpl;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VirtualScreen;

impl ActionImpl for VirtualScreen {
    type State = ();

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        ctx.kwin.set_script_enabled("truevideowall", true)?;
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

    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()> {
        ctx.kwin.set_script_enabled("truevideowall", false)?;
        Ok(())
    }

    fn get_dependencies(&self, _ctx: &mut PipelineContext) -> Vec<Dependency> {
        vec![Dependency::KwinScript(
            "truevideowall-v1.0.kwinscript".to_string(),
        )]
    }
}
