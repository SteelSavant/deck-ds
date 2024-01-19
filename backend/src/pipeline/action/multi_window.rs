use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use xrandr::{Output, Relation};

use crate::{
    pipeline::{
        action::ui_management::{Pos, Size},
        dependency::Dependency,
        executor::PipelineContext,
    },
    sys::x_display::XDisplay,
};

use super::{ui_management::UiEvent, ActionId, ActionImpl};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MultiWindow {
    pub id: ActionId,
}

impl ActionImpl for MultiWindow {
    type State = ();

    const NAME: &'static str = "MultiWindow";

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

        let res = display.set_output_position(&deck, &Relation::Below, &external);

        fn viewport_update(
            display: &mut XDisplay,
            external: &Output,
            deck: &Output,
        ) -> Result<UiEvent> {
            let external_mode = display
                .get_current_mode(external)
                .with_context(|| "failed to get mode for external display")?
                .with_context(|| "failed to get mode for external display")?;

            let deck_mode = display
                .get_current_mode(deck)
                .with_context(|| "failed to get mode for embedded display")?
                .with_context(|| "failed to get mode for embedded display")?;

            Ok(UiEvent::UpdateViewports {
                primary_size: Size(external_mode.height, external_mode.width),
                secondary_size: Size(deck_mode.height, deck_mode.width),
                primary_position: Pos(0, 0),
                secondary_position: Pos(0, external_mode.height),
            })
        }

        let update = viewport_update(display, &external, &deck);
        if let Ok(event) = update {
            ctx.send_ui_event(event);
        }

        res
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

    #[inline]
    fn get_id(&self) -> ActionId {
        self.id
    }
}
