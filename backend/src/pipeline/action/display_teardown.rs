use std::cmp::Ordering;

use anyhow::{Ok, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use xrandr::{Mode, Monitor, Output, Relation, ScreenResources, XHandle};

use crate::pipeline::executor::PipelineContext;

use super::PipelineActionExecutor;

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DisplayTeardown {
    teardown_external_settings: TeardownExternalSettings,
    teardown_deck_location: RelativeLocation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayState {
    previous_output_configuration: Output,
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
    /// Highest resolution under h by v with refresh rate r. If use_native_aspect_ratio is true, select closest with native aspect ratio.
    Limited {
        h: u16,
        v: u16,
        r: u16,
        use_native_aspect_ratio: bool,
    },
}

impl PipelineActionExecutor for DisplayTeardown {
    type State = DisplayState;

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        // let mut handle = xrandr::XHandle::open()?;

        // let preferred = self.get_preferred_output(&mut handle)?;
        // match preferred {
        //     Some(output) => {
        //         ctx.set_state::<Self>(DisplayState {
        //             previous_output_configuration: output,
        //         });
        //         Ok(())
        //     },
        //     None => Err(anyhow::anyhow!("Unable to find external display for dual screen")),
        // }

        Ok(())
    }

    fn tear_down(&self, ctx: &mut PipelineContext) -> Result<()> {
        // match ctx.get_state::<Self>() {
        //     Some(state) => {
        //         let mut handle = xrandr::XHandle::open()?;
        //         let resources = ScreenResources::new(&mut handle)?;

        //         let output = resources.output(&mut handle, state.previous_output_configuration.xid)?;

        //                 match self.teardown_external_settings {
        //     TeardownExternalSettings::Previous => {
        //         state.previous_output_configuration.current_mode
        //     } handle.set_mode(&output, ),
        //     TeardownExternalSettings::Native => todo!(),
        //     TeardownExternalSettings::Limited {
        //         h,
        //         v,
        //         r,
        //         use_native_aspect_ratio,
        //     } => todo!(),
        // }
        //     },
        //     /// No state, nothing to tear down
        //     None => Ok(()),
        // }

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
