use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pipeline::{
    action::{ActionId, ActionImpl, ActionType},
    data::PipelineActionId,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PlatformSelect {
    pub id: ActionId,
    pub platform: PipelineActionId,
}

impl ActionImpl for PlatformSelect {
    type State = ();

    const TYPE: ActionType = ActionType::PlatformSelect;

    fn get_id(&self) -> ActionId {
        self.id
    }
}
