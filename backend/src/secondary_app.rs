use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{asset::AssetManager, macros::newtype_uuid, sys::flatpak::list_installed_flatpaks};

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
    name: String,
    app: SecondaryApp,
}

pub struct SecondaryAppManager {
    asset_manager: AssetManager<'static>,
}

impl SecondaryAppManager {
    pub fn new(asset_manager: AssetManager<'static>) -> Self {
        Self { asset_manager }
    }

    pub fn get_presets(&self) -> HashMap<SecondaryAppPresetId, SecondaryAppPreset> {
        self.asset_manager
            .get_dir("secondary_apps")
            .map(|v| {
                v.into_iter()
                    .filter_map(|e| {
                        match e {
                            crate::asset::AssetDirEntry::Dir(_) => None, // TODO::walkdir if needed
                            crate::asset::AssetDirEntry::File(file) => {
                                self.asset_manager.get_file(&file)
                            }
                        }
                    })
                    .filter_map(|v| {
                        let contents = v.contents_to_string().ok()?;
                        let de: HashMap<SecondaryAppPresetId, SecondaryAppPreset> =
                            serde_json::from_str(&contents).ok()?; // TODO::logging or better error handling
                        Some(de.into_iter())
                    })
                    .flatten()
                    .filter(|v| {
                        // Only return installed flatpaks

                        match v.1.app {
                            SecondaryApp::Flatpak(ref flatpak) => {
                                list_installed_flatpaks()
                                    .unwrap_or_default() // TODO::error handling                                 installed.into_iter().any(|v| v.app_id == flatpak.app_id)
                                    .into_iter()
                                    .any(|v| v.app_id == flatpak.app_id)
                            }
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    // TODO::new/update preset user presets
}
