use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pipeline::{action::ActionId, executor::PipelineContext};

use super::{
    super::{Action, ActionImpl, ActionType, ErasedPipelineAction},
    citra_layout::{CitraLayout, CitraLayoutState},
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(transparent)]
pub struct Lime3dsLayout(pub CitraLayout);

impl ActionImpl for Lime3dsLayout {
    type State = CitraLayoutState;

    const TYPE: ActionType = ActionType::Lime3dsLayout;

    fn get_id(&self) -> ActionId {
        self.0.id
    }

    fn setup(&self, ctx: &mut PipelineContext) -> anyhow::Result<()> {
        Action::from(self.0).setup(ctx)
    }

    fn teardown(&self, ctx: &mut PipelineContext) -> anyhow::Result<()> {
        Action::from(self.0).teardown(ctx)
    }

    fn get_dependencies(
        &self,
        ctx: &PipelineContext,
    ) -> Vec<crate::pipeline::dependency::Dependency> {
        Action::from(self.0).get_dependencies(ctx)
    }
}
