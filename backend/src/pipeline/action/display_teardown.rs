use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use xrandr::{Monitor, Relation};

use crate::pipeline::executor::PipelineContext;

use super::PipelineActionExecutor;

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DisplayTeardown {
    teardown_external_settings: TeardownExternalSettings,
    teardown_deck_location: RelativeLocation,
    timing_fallback_method: TimingFallbackMethod,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayState {
    previous_configuration: Vec<Monitor>,
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

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, JsonSchema)]
pub enum TimingFallbackMethod {
    #[default]
    CvtR,
    Cvt,
    Gtf,
    // Manual
}

impl PipelineActionExecutor for DisplayTeardown {
    type State = DisplayState;

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        let mut handle = xrandr::XHandle::open()?;

        let monitors = handle.monitors()?;

        ctx.set_state::<Self>(DisplayState {
            previous_configuration: monitors,
        });
        Ok(())
    }

    fn tear_down(&self, ctx: &mut PipelineContext) -> Result<()> {
        let state = ctx.get_state::<Self>();

        let mut handle = xrandr::XHandle::open()?;
        let monitors = handle.monitors()?;
        for monitor in monitors {}

        // match self.teardown_external_settings {
        //     TeardownExternalSettings::Previous => todo!(),
        //     TeardownExternalSettings::Native => todo!(),
        //     TeardownExternalSettings::Limited {
        //         h,
        //         v,
        //         r,
        //         use_native_aspect_ratio,
        //     } => todo!(),
        // }

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
