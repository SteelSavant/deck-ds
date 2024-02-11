use anyhow::Context;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pipeline::action::{ActionId, ActionImpl};

pub use super::common::{ExternalDisplaySettings, RelativeLocation};
use super::session_handler::UiEvent;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DisplayConfig {
    pub id: ActionId,
    pub external_display_settings: ExternalDisplaySettings,
    // Some(Location) for relative location, None for disabled
    pub deck_location: Option<RelativeLocation>,
    pub disable_splash: bool,
}

impl ActionImpl for DisplayConfig {
    type State = ();

    const NAME: &'static str = "DisplayConfig";

    fn setup(&self, ctx: &mut crate::pipeline::executor::PipelineContext) -> anyhow::Result<()> {
        if self.disable_splash {
            let _ = ctx.send_ui_event(UiEvent::Close);
        }

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
            if let Some(embedded) = embedded {
                match self.deck_location {
                    Some(location) => {
                        display.set_output_position(&embedded, &location.into(), &preferred)?;
                    }
                    None => {
                        display.set_output_enabled(&embedded, false)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn teardown(&self, ctx: &mut crate::pipeline::executor::PipelineContext) -> anyhow::Result<()> {
        if self.deck_location.is_none() {
            let display = ctx
                .display
                .as_mut()
                .with_context(|| "DisplayConfig requires x11 to be running")?;

            if let Some(embedded) = display.get_embedded_output()? {
                display.set_output_enabled(&embedded, true)?;
            }
        }

        Ok(())
    }

    fn get_id(&self) -> ActionId {
        self.id
    }
}
