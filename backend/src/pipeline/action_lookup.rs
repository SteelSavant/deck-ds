use std::{collections::HashMap, sync::Arc};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    action_registar::PipelineActionRegistrar,
    data::{PipelineActionDefinition, PipelineActionId, PipelineActionSettings, PipelineTarget},
};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PipelineActionLookup {
    actions: Arc<HashMap<PipelineActionId, PipelineActionSettings>>,
}

impl PipelineActionLookup {
    pub fn get(
        &self,
        id: &PipelineActionId,
        target: PipelineTarget,
        registrar: &PipelineActionRegistrar,
    ) -> Option<PipelineActionDefinition> {
        let variant = id.variant(target);

        registrar.get(id, target).map(|def| {
            let settings = self
                .actions
                .get(&variant)
                .or_else(|| self.actions.get(id))
                .cloned();
            PipelineActionDefinition {
                settings: settings.unwrap_or_else(|| def.settings.clone()),
                ..def.clone()
            }
        })
    }
}
