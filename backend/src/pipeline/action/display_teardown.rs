use std::cmp::Ordering;

use anyhow::{Ok, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use xrandr::{Mode, Monitor, Output, Relation, ScreenResources, XHandle, XId};

use crate::pipeline::executor::PipelineContext;

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
    /// Highest resolution <= h by v with refresh rate r. If use_native_aspect_ratio is true, select closest with native aspect ratio.
    Limited {
        h: u16,
        v: u16,
        r: u16,
        use_native_aspect_ratio: bool,
    },
}

impl PipelineActionImpl for DisplayConfig {
    type State = DisplayState;

    fn id(&self) -> PipelineActionId {
        PipelineActionId(
            Uuid::parse_str("be4b11ef-288f-4493-a28a-3dd790d05813").expect("guid should be valid"),
        )
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
        match ctx.get_state::<Self>() {
            Some(state) => {
                let output = state.previous_output_id;

                match self.teardown_external_settings {
                    TeardownExternalSettings::Previous => match state.previous_output_mode {
                        Some(mode) => {
                            let current_output = ctx.display.get_preferred_external_output()?;
                            match current_output {
                                Some(current) => {
                                    if current.xid == output {
                                        let mode = ctx.display.get_mode(mode)?;
                                        ctx.display.set_output_mode(&current, &mode)
                                    } else {
                                        Ok(())
                                    }
                                }
                                None => Ok(()),
                            }
                        }
                        None => DisplayConfig {
                            teardown_external_settings: TeardownExternalSettings::Native,
                            ..*self
                        }
                        .teardown(ctx),
                    },
                    TeardownExternalSettings::Native => todo!(),
                    TeardownExternalSettings::Limited {
                        h,
                        v,
                        r,
                        use_native_aspect_ratio,
                    } => todo!(),
                }
            }
            // No state, nothing to tear down
            None => Ok(()),
        }
    }

    fn get_schema(&self) -> schemars::schema::Schema {
        todo!()
    }
}

#[cfg(test)]
mod tests {}
