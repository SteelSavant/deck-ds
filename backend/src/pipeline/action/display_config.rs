use anyhow::{Ok, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use xrandr::{Relation, XId};

use crate::{pipeline::executor::PipelineContext, sys::x_display::ModePreference};

use super::{PipelineActionId, PipelineActionImpl};

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DisplayConfig {
    pub teardown_external_settings: TeardownExternalSettings,
    pub teardown_deck_location: RelativeLocation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayState {
    previous_output_id: XId,
    previous_output_mode: Option<XId>,
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, JsonSchema)]
pub enum RelativeLocation {
    Above,
    #[default]
    Below,
    LeftOf,
    RightOf,
    SameAs,
}

impl Into<Relation> for RelativeLocation {
    fn into(self) -> Relation {
        match self {
            RelativeLocation::Above => Relation::Above,
            RelativeLocation::Below => Relation::Below,
            RelativeLocation::LeftOf => Relation::LeftOf,
            RelativeLocation::RightOf => Relation::RightOf,
            RelativeLocation::SameAs => Relation::SameAs,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, JsonSchema)]
pub enum TeardownExternalSettings {
    /// Previous resolution, before setup
    #[default]
    Previous,
    /// Native resolution
    Native,
    /// Resolution based on specific settings
    Preference(ModePreference),
}

impl PipelineActionImpl for DisplayConfig {
    type State = DisplayState;

    fn id(&self) -> PipelineActionId {
        PipelineActionId::parse("be4b11ef-288f-4493-a28a-3dd790d05813")
    }

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        let preferred = ctx.display.get_preferred_external_output()?;
        match preferred {
            Some(output) => {
                ctx.set_state::<Self>(DisplayState {
                    previous_output_id: output.xid,
                    previous_output_mode: output.current_mode,
                });
                Ok(())
            }
            None => Err(anyhow::anyhow!(
                "Unable to find external display for dual screen"
            )),
        }
    }

    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()> {
        let current_output = ctx.display.get_preferred_external_output()?;

        match ctx.get_state::<Self>() {
            Some(state) => {
                let output = state.previous_output_id;

                // Gets the current output. If it matches the saved, return it,
                // otherwise exit teardown to avoid changing current monitor to
                // old monitor settings.
                let current_output = match current_output {
                    Some(current) => {
                        if current.xid == output {
                            current
                        } else {
                            return Ok(());
                        }
                    }
                    None => return Ok(()),
                };

                match self.teardown_external_settings {
                    TeardownExternalSettings::Previous => match state.previous_output_mode {
                        Some(mode) => {
                            let mode = ctx.display.get_mode(mode)?;
                            ctx.display.set_output_mode(&current_output, &mode)
                        }
                        None => DisplayConfig {
                            teardown_external_settings: TeardownExternalSettings::Native,
                            ..*self
                        }
                        .teardown(ctx),
                    },
                    TeardownExternalSettings::Native => {
                        let mode = current_output
                            .preferred_modes
                            .iter()
                            .map(|mode| ctx.display.get_mode(*mode))
                            .collect::<Result<Vec<_>, _>>()?;
                        let native_mode = mode.iter().reduce(|acc, e| {
                            match (acc.width * acc.height).cmp(&(e.width * e.height)) {
                                std::cmp::Ordering::Less => e,
                                std::cmp::Ordering::Greater => acc,
                                std::cmp::Ordering::Equal => {
                                    if acc.rate > e.rate {
                                        acc
                                    } else {
                                        e
                                    }
                                }
                            }
                        });
                        if let Some(mode) = native_mode {
                            ctx.display.set_output_mode(&current_output, &mode)
                        } else {
                            Ok(())
                        }
                    }
                    TeardownExternalSettings::Preference(preference) => ctx
                        .display
                        .set_or_create_preferred_mode(&current_output, &preference),
                }?;

                let deck = ctx.display.get_embedded_output()?.unwrap();
                ctx.display.set_output_position(
                    &deck,
                    &self.teardown_deck_location.into(),
                    &current_output,
                )
            }
            // No state, nothing to tear down
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {}
