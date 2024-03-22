use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::macros::newtype_uuid;

newtype_uuid!(SecondaryAppPresetId);

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "value")]
pub enum SecondaryApp {
    Flatpak(FlatpakApp),
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize, JsonSchema)]
pub struct FlatpakApp {
    pub app_id: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize, JsonSchema)]
pub struct SecondaryAppPreset {
    id: SecondaryAppPresetId,
    name: String,
    app: SecondaryApp,
}

// TODO::system for saving/loading default + user presets
