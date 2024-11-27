use anyhow::Context;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pipeline::{
    action::{ActionId, ActionImpl, ActionType},
    dependency::Dependency,
};

pub use super::common::{ExternalDisplaySettings, RelativeLocation};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DisplayConfig {
    pub id: ActionId,
    pub external_display_settings: ExternalDisplaySettings,
    // Some(Location) for relative location, None for disabled
    pub deck_location: Option<RelativeLocation>,
    pub deck_is_primary_display: bool,
}

// TODO::ideally, this would listen for changes to connected monitors and re-run accordingly
impl ActionImpl for DisplayConfig {
    type State = ();

    const TYPE: ActionType = ActionType::DisplayConfig;

    fn setup(&self, ctx: &mut crate::pipeline::executor::PipelineContext) -> anyhow::Result<()> {
        let display = ctx
            .display
            .as_mut()
            .with_context(|| "DisplayConfig requires x11 to be running")?;

        let preferred = display.get_preferred_external_output()?;
        let mut embedded = display.get_embedded_output()?;

        if let Some(preferred) = preferred.as_ref() {
            if preferred.connected {
                match self.external_display_settings {
                    ExternalDisplaySettings::Previous => Ok(()),
                    ExternalDisplaySettings::Native => {
                        let native_mode = display.get_native_mode(preferred)?;
                        if let Some(mode) = native_mode {
                            display.set_output_mode(preferred, &mode)
                        } else {
                            Ok(())
                        }
                    }
                    ExternalDisplaySettings::Preference(preference) => {
                        display.set_or_create_preferred_mode(preferred, &preference)
                    }
                }?;

                if let Some(embedded) = embedded.as_mut() {
                    match self.deck_location {
                        Some(location) => {
                            display
                                .reconfigure_embedded(
                                    embedded,
                                    &location.into(),
                                    Some(preferred),
                                    self.deck_is_primary_display,
                                )
                                .with_context(|| "reconfigure embedded failed")?;
                        }
                        None => {
                            // TODO:: viewport update for the remaining display
                            display.set_output_enabled(embedded, false)?;
                        }
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
        _ctx: &crate::pipeline::executor::PipelineContext,
    ) -> Vec<crate::pipeline::dependency::Dependency> {
        vec![Dependency::Display]
    }
}
