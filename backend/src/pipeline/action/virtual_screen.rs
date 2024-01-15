use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use xrandr::Relation;

use crate::{
    pipeline::{dependency::Dependency, executor::PipelineContext},
    sys::x_display::{AspectRatioOption, ModeOption, ModePreference, Resolution},
};

use super::{
    ui_management::{Pos, Size},
    ActionId, ActionImpl,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VirtualScreen {
    pub id: ActionId,
}

impl ActionImpl for VirtualScreen {
    type State = ();

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        ctx.kwin.set_script_enabled("truevideowall", true)?;
        let display = ctx
            .display
            .as_mut()
            .with_context(|| "VirtualScreen requires x11 to be running")?;

        let external = display
            .get_preferred_external_output()?
            .ok_or(anyhow::anyhow!("Failed to find external display"))?;

        let deck = display
            .get_embedded_output()?
            .ok_or(anyhow::anyhow!("Failed to find embedded display"))?;

        let deck_mode = display
            .get_current_mode(&deck)?
            .expect("Deck screen should have active mode");
        let resolution = if deck_mode.width < deck_mode.height {
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

        display.set_or_create_preferred_mode(
            &external,
            &ModePreference {
                resolution: ModeOption::Exact(resolution),
                aspect_ratio: AspectRatioOption::Exact(resolution.w as f32 / resolution.h as f32),
                refresh: ModeOption::Exact(deck_mode.rate),
            },
        )?;

        let res = display.set_output_position(&deck, &Relation::Below, &external);

        ctx.send_ui_event(super::ui_management::UiEvent::UpdateViewports {
            primary_size: Size(resolution.w, resolution.h),
            secondary_size: Size(resolution.w, resolution.h),
            primary_position: Pos(0, 0),
            secondary_position: Pos(0, resolution.h),
        });

        res
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

    fn get_id(&self) -> ActionId {
        self.id
    }
}
