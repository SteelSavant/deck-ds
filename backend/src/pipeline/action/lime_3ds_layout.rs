use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    citra_layout::{CitraLayout, CitraLayoutState},
    ActionImpl, ActionType,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(transparent)]
pub struct Lime3dsLayout(pub CitraLayout);

impl ActionImpl for Lime3dsLayout {
    type State = CitraLayoutState;

    const TYPE: ActionType = ActionType::Lime3dsLayout;

    fn get_id(&self) -> super::ActionId {
        self.0.id
    }

    fn setup(&self, ctx: &mut crate::pipeline::executor::PipelineContext) -> anyhow::Result<()> {
        self.0.setup(ctx)
    }

    fn teardown(&self, ctx: &mut crate::pipeline::executor::PipelineContext) -> anyhow::Result<()> {
        self.0.teardown(ctx)
    }
}
