use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use xrandr::Output;

use crate::{
    pipeline::{
        action::{
            session_handler::{Pos, Size},
            ActionId, ActionImpl,
        },
        dependency::Dependency,
    },
    sys::x_display::XDisplay,
};

pub use super::common::{ExternalDisplaySettings, RelativeLocation};
use super::session_handler::UiEvent;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DisplayConfig {
    pub id: ActionId,
    pub external_display_settings: ExternalDisplaySettings,
    // Some(Location) for relative location, None for disabled
    pub deck_location: Option<RelativeLocation>,
    pub deck_is_primary_display: bool,
}

impl ActionImpl for DisplayConfig {
    type State = ();

    const NAME: &'static str = "DisplayConfig";

    fn setup(&self, ctx: &mut crate::pipeline::executor::PipelineContext) -> anyhow::Result<()> {
        let display = ctx
            .display
            .as_mut()
            .with_context(|| "DisplayConfig requires x11 to be running")?;

        if let Some(preferred) = display.get_preferred_external_output()? {
            match self.external_display_settings {
                ExternalDisplaySettings::Previous => Ok(()),
                ExternalDisplaySettings::Native => {
                    let native_mode = display.get_native_mode(&preferred)?;
                    if let Some(mode) = native_mode {
                        display.set_output_mode(&preferred, &mode)
                    } else {
                        Ok(())
                    }
                }
                ExternalDisplaySettings::Preference(preference) => {
                    display.set_or_create_preferred_mode(&preferred, &preference)
                }
            }?;

            let embedded = display.get_embedded_output()?;

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

            if let Some(mut embedded) = embedded {
                match self.deck_location {
                    Some(location) => {
                        display
                            .reconfigure_embedded(
                                &mut embedded,
                                &location.into(),
                                Some(&preferred),
                                self.deck_is_primary_display,
                            )
                            .with_context(|| "reconfigure embedded failed")?;

                        let update = viewport_update(display, &preferred, &embedded);
                        if let Ok(event) = update {
                            ctx.send_ui_event(event);
                        }
                    }
                    None => {
                        // TODO:: viewport update for the remaining display
                        display.set_output_enabled(&mut embedded, false)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn teardown(
        &self,
        _ctx: &mut crate::pipeline::executor::PipelineContext,
    ) -> anyhow::Result<()> {
        // teardown handled by session handler

        Ok(())
    }

    fn get_id(&self) -> ActionId {
        self.id
    }

    fn get_dependencies(
        &self,
        _ctx: &mut crate::pipeline::executor::PipelineContext,
    ) -> Vec<crate::pipeline::dependency::Dependency> {
        vec![Dependency::Display]
    }
}
