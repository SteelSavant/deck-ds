use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    pipeline::{dependency::Dependency, executor::PipelineContext},
    settings_db::SystemDisplay,
    sys::{
        display_info::get_display_info,
        x_display::{AspectRatioOption, ModeOption, ModePreference, Resolution},
    },
};

use super::{ActionId, ActionImpl, ActionType};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct VirtualScreen {
    pub id: ActionId,
}

const SCRIPT: &str = "truevideowall";

// TODO::ideally, this would listen for changes to connected monitors and re-run accordingly
impl ActionImpl for VirtualScreen {
    type State = bool;

    const TYPE: ActionType = ActionType::VirtualScreen;

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        let enabled = ctx.kwin.get_script_enabled(SCRIPT);
        ctx.set_state::<Self>(matches!(enabled, Ok(true)));

        ctx.kwin.set_script_enabled(SCRIPT, true)?;
        let display = ctx
            .display
            .as_mut()
            .with_context(|| "VirtualScreen requires x11 to be running")?;

        let display_info = get_display_info()?;

        let display_settings = ctx
            .settings_db
            .get_monitor_display_settings(&display_info)?;

        let external = display.get_preferred_external_output(&display_settings)?;

        let mut deck = display
            .get_embedded_output()?
            .ok_or(anyhow::anyhow!("Failed to find embedded display"))?;

        let deck_mode = display
            .get_current_mode(&deck)?
            .expect("Embedded display should have active mode");

        if let Some((external, monitor_settings)) = external {
            let external_mode = display
                .get_current_mode(&external)?
                .expect("External display should have active mode");

            let smallest_mode = if deck_mode.height * deck_mode.width
                > external_mode.height * external_mode.width
            {
                &external_mode
            } else {
                &deck_mode
            };

            let resolution = if smallest_mode.width < smallest_mode.height {
                Resolution {
                    h: smallest_mode.width,
                    w: smallest_mode.height,
                }
            } else {
                Resolution {
                    w: smallest_mode.width,
                    h: smallest_mode.height,
                }
            };

            display.set_or_create_preferred_mode(
                &external,
                &ModePreference {
                    resolution: ModeOption::Exact(resolution),
                    aspect_ratio: AspectRatioOption::Exact(
                        resolution.w as f32 / resolution.h as f32,
                    ),
                    refresh: ModeOption::AtMost(deck_mode.rate),
                },
            )?;

            display.reconfigure_embedded(
                &mut deck,
                &monitor_settings.deck_location.into(),
                Some(&external),
                monitor_settings.system_display == SystemDisplay::Embedded,
            )?;
        } else {
            display.reconfigure_embedded(&mut deck, &xrandr::Relation::Below, None, true)?;
        }

        Ok(())
    }

    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()> {
        // Display teardown handled by session handler; we just need to disable the kwinscript

        let state = ctx.get_state::<Self>();
        ctx.kwin
            .set_script_enabled(SCRIPT, matches!(state, Some(true)))
    }

    fn get_dependencies(&self, _ctx: &PipelineContext) -> Vec<Dependency> {
        vec![
            Dependency::KWinScript(SCRIPT.to_string()),
            Dependency::Display,
        ]
    }

    #[inline]
    fn get_id(&self) -> ActionId {
        self.id
    }
}
