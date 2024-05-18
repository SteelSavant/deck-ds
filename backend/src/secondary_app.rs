use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{asset::AssetManager, macros::newtype_uuid};

newtype_uuid!(SecondaryAppPresetId);

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(tag = "type")]
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
    pub name: String,
    pub app: SecondaryApp,
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
                                self.asset_manager.get_file(file)
                            }
                        }
                    })
                    .filter_map(|v| {
                        let contents = v.contents_to_string().ok()?;
                        let de: HashMap<SecondaryAppPresetId, SecondaryAppPreset> =
                            serde_json::from_str(&contents)
                                .inspect_err(|err| {
                                    log::warn!(
                                        "failed to parse presets at {:?}; ignoring presets: {err}",
                                        v.file_path()
                                    )
                                })
                                .ok()?;
                        Some(de.into_iter())
                    })
                    .flatten()
                    .collect()
            })
            .unwrap_or_default()
    }

    // TODO::new/update preset user presets
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use log::Level;

    use crate::asset::AssetManager;

    use super::SecondaryAppManager;

    // TODO::fix this

    // #[test]
    // fn test_parsed_embedded_secondary_apps() {
    //     testing_logger::setup();

    //     let asset_manager =
    //         AssetManager::new(&ASSETS_DIR, Path::new("./not_a_real_path").to_path_buf());
    //     let secondary_apps = SecondaryAppManager::new(asset_manager).get_presets();

    //     testing_logger::validate(|logs| {
    //         for log in logs {
    //             assert!(log.level > Level::Warn, "{}", log.body);
    //         }
    //     });

    //     assert!(
    //         secondary_apps.keys().count() > 0,
    //         "should find at least one secondary app preset"
    //     )
    // }
}
