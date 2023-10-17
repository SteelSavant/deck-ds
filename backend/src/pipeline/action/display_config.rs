use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
// use xrandr::Monitor;

use crate::pipeline::executor::PipelineContext;

use super::PipelineActionExecutor;

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DisplayConfig {
    teardown_external_settings: TeardownExternalSettings,
    teardown_deck_location: TeardownLocation,
    timing_fallback_method: TimingFallbackMethod,
}

#[derive(Debug)]
pub struct DisplayState {
    // previous_configuration: Vec<Monitor>,
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

impl PipelineActionExecutor for DisplayConfig {
    type S = Self;
    type State = DisplayState;

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        // let mut handle = xrandr::XHandle::open()?;

        // let monitors = handle.monitors()?;
        // let m = monitors[0];
        // let o = m.outputs[0];

        // let crtc =

        // handle.set_position(output, relation, relative_output)
        // self.set_state(ctx, DisplayState {
        //     previous_configuration:  monitors
        // });
        Ok(())
    }

    fn tear_down(&self, ctx: &mut PipelineContext) -> Result<()> {
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
