use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use xrandr::XId;

use crate::pipeline::action::{ActionId, ActionImpl};

pub use super::common::{ExternalDisplaySettings, RelativeLocation};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]

pub struct DisplayConfig {
    pub id: ActionId,
    pub external_display_settings: ExternalDisplaySettings,
    // Some(Location) for relative location, None for disabled
    pub deck_location: Option<RelativeLocation>,
}

#[cfg_attr(test, derive(Default))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplayState {
    previous_output_id: XId,
    previous_output_mode: Option<XId>,
}

impl ActionImpl for DisplayConfig {
    type State = DisplayState;

    const NAME: &'static str = "DisplayConfig";

    fn setup(&self, _ctx: &mut crate::pipeline::executor::PipelineContext) -> anyhow::Result<()> {
        todo!()
    }

    fn get_id(&self) -> ActionId {
        self.id
    }
}
