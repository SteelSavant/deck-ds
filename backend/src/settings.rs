use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::pipeline::{
    action::PipelineActionId,
    config::{PipelineActionDefinitionId, PipelineDefinition, PipelineDefinitionId},
};

pub struct Settings {
    dir: PathBuf,
    templates: Vec<PipelineDefinition>,
    profiles: Vec<Profile>,
}

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProfileId(Uuid);

pub struct Profile {
    template_id: PipelineDefinitionId,
    profile_id: ProfileId,
    actions: HashMap<PipelineActionDefinitionId, (PipelineActionId, HashMap<String, Value>)>,
}
