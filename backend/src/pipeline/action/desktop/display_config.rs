use anyhow::Context;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    pipeline::{
        action::{ActionId, ActionImpl, ActionType},
        dependency::Dependency,
    },
    settings_db::SystemDisplay,
    sys::display_info::get_display_info,
};

pub use super::common::{ExternalDisplaySettings, RelativeLocation};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DisplayConfig {
    pub id: ActionId,
    /// External display settings. If `None`, use value from global settings.
    pub external_display_settings: Option<ExternalDisplaySettings>,
    /// If `true`, deck display is enabled. If `false`, deck display is disabled. If `None`, use global settings.
    pub deck_is_enabled: Option<bool>,
}

// TODO::ideally, this would listen for changes to connected monitors and re-run accordingly
impl ActionImpl for DisplayConfig {
    type State = ();

    const TYPE: ActionType = ActionType::DisplayConfig;

    fn setup(&self, ctx: &mut crate::pipeline::executor::PipelineContext) -> anyhow::Result<()> {
        let display = ctx
            .display
            .as_mut()
            .context("DisplayConfig requires x11 to be running")?;

        let display_info = get_display_info()?;

        let display_settings = ctx
            .settings_db
            .get_monitor_display_settings(&display_info)?;

        let preferred = display.get_preferred_external_output(&display_settings)?;

        let mut embedded = display.get_embedded_output()?;

        if let Some((preferred, monitor_settings)) = preferred.as_ref() {
            if preferred.connected {
                match self
                    .external_display_settings
                    .unwrap_or(monitor_settings.external_display_settings)
                {
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
                    if self
                        .deck_is_enabled
                        .unwrap_or(monitor_settings.deck_is_enabled)
                    {
                        display
                            .reconfigure_embedded(
                                embedded,
                                &monitor_settings.deck_location.into(),
                                Some(preferred),
                                monitor_settings.system_display == SystemDisplay::Embedded,
                            )
                            .with_context(|| "reconfigure embedded failed")?;
                    } else {
                        // TODO:: viewport update for the remaining display
                        display.set_output_enabled(embedded, false)?;
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
