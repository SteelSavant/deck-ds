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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct VirtualScreen {
    pub id: ActionId,
}

const SCRIPT: &'static str = "truevideowall";

// TODO::restore kwin script settings

impl ActionImpl for VirtualScreen {
    type State = bool;

    const NAME: &'static str = "VirtualScreen";

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        let enabled = ctx.kwin.get_script_enabled(SCRIPT);
        ctx.set_state::<Self>(matches!(enabled, Ok(true)));

        ctx.kwin.set_script_enabled(SCRIPT, true)?;
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
        let state = ctx.get_state::<Self>();
        ctx.kwin
            .set_script_enabled(SCRIPT, matches!(state, Some(true)))?;

        Ok(())
    }

    fn get_dependencies(&self, _ctx: &mut PipelineContext) -> Vec<Dependency> {
        vec![
            Dependency::KwinScript(SCRIPT.to_string()),
            // Display dependencies
            Dependency::System("xrandr".to_string()),
            Dependency::System("cvt".to_string()),
        ]
    }

    #[inline]
    fn get_id(&self) -> ActionId {
        self.id
    }
}
