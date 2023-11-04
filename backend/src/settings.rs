use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    macros::{newtype_strid, newtype_uuid},
    pipeline::config::{PipelineActionDefinitionId, PipelineDefinitionId},
};

pub mod autostart;
pub mod patch;

newtype_uuid!(ProfileId);
newtype_strid!(AppId);

pub struct Settings {
    profiles_dir: PathBuf,
    apps_dir: PathBuf,
}

impl Settings {
    pub fn new<P: AsRef<Path>>(config_dir: P) -> Self {
        let config_dir = config_dir.as_ref();
        Self {
            profiles_dir: config_dir.join("profiles"),
            apps_dir: config_dir.join("apps"),
        }
    }

    pub fn get_profile(&self, id: &PipelineActionDefinitionId) -> Result<Profile> {
        let raw = id.raw();

        let profile_path = self.profiles_dir.join(raw).with_extension("json");
        let profile = std::fs::read_to_string(profile_path)?;

        Ok(serde_json::from_str(&profile)?)
    }

    pub fn set_profile(&self, profile: &Profile) -> Result<()> {
        let raw = profile.id.raw();

        let serialized = serde_json::to_string_pretty(profile)?;
        let profile_path = self.profiles_dir.join(raw).with_extension("json");

        Ok(std::fs::write(profile_path, serialized)?)
    }

    pub fn get_app(&self, id: &PipelineActionDefinitionId) -> Result<App> {
        let raw = id.raw();

        let app_path = self.apps_dir.join(raw).with_extension("json");
        let app = std::fs::read_to_string(app_path)?;

        Ok(serde_json::from_str(&app)?)
    }

    pub fn set_app(&self, app: &App) -> Result<()> {
        let raw = app.id.raw();

        let serialized = serde_json::to_string_pretty(app)?;
        let app_path = self.apps_dir.join(raw).with_extension("json");

        Ok(std::fs::write(app_path, serialized)?)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Profile {
    id: ProfileId,
    template: PipelineDefinitionId,
    tags: Vec<String>,
    overrides: Overrides,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct App {
    id: AppId,
    active_profile: PipelineDefinitionId,
    overrides: HashMap<PipelineDefinitionId, Overrides>,
}

/// Overrides for a pipeline definition.
///
/// Json is in the format
///
/// ```json
/// {
///     "guid_for_action_selection": {
///         "overridden_field1": "value1",
///         "overridden_field2": 2,
///         "overridden_field3": {
///             "nested_field": 4.5
///         }
///     },
///     "guid_for_oneof": {
///         "selection": "some_guid",
///     },
/// }
/// ```
///
/// All guids are flattened top-level, so [Selection::AllOf] and [Selection::OneOf]::actions will not exist.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Overrides {
    pub fields: HashMap<PipelineActionDefinitionId, Value>,
    pub enabled: HashMap<PipelineActionDefinitionId, bool>,
}
