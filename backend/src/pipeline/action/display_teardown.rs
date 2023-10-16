use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pipeline::common::Context;

use super::PipelineActionExecutor;

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DisplayTeardown {
    teardown_deck_location: TeardownLocation,
    teardown_external_settings: TeardownExternalSettings,
    timing_fallback_method: TimingFallbackMethod,
}

pub struct DisplayState {
    // TODO::store state (previous display configuration, etc)
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, JsonSchema)]
pub enum TeardownLocation {
    #[default]
    Previous,
    Top(HorizontalLocation),
    Bottom(HorizontalLocation),
    Left(VerticalLocation),
    Right(VerticalLocation),
    Mirror,
}

#[derive(Serialize, Deserialize)]
pub struct VirtualScreenState {
    // TODO::previous location state
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema)]
pub enum VerticalLocation {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema)]
pub enum HorizontalLocation {
    Left,
    Center,
    Right,
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
    fn setup(&self, ctx: &mut Context) -> Result<(), String> {
        Ok(())
        // todo!("get previous display configurations")
    }
    fn tear_down(&self, ctx: &mut Context) -> Result<(), String> {
        // todo!("Teardown")

        Ok(())
    }
}
