use std::{
    collections::HashMap,
    fmt::Debug,
    path::{Path, PathBuf},
};

use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    macros::{newtype_strid, newtype_uuid},
    pipeline::{
        action::{
            cemu_config::{CemuConfig, CemuXmlSource},
            citra_config::{CitraConfig, CitraIniSource, CitraLayoutOption},
            display_config::{DisplayConfig, RelativeLocation, TeardownExternalSettings},
            melonds_config::{
                MelonDSConfig, MelonDSIniSource, MelonDSLayoutOption, MelonDSSizingOption,
            },
            multi_window::MultiWindow,
            virtual_screen::VirtualScreen,
        },
        config::{
            PipelineActionDefinition, PipelineActionDefinitionId, PipelineDefinition,
            PipelineDefinitionId, PipelineTarget, Selection,
        },
        dependency::Dependency,
    },
    util::create_dir_all,
};

pub mod patch;

newtype_uuid!(ProfileId);
newtype_strid!(AppId);

#[derive(Debug, Clone)]
pub struct Settings {
    // Path vars
    profiles_dir: PathBuf,
    apps_dir: PathBuf,
    autostart_path: PathBuf,

    // in-memory configurations -- consider moving
    templates: Vec<PipelineDefinition>,
    dependencies: Vec<Dependency>,
}

impl Settings {
    pub fn new<P: AsRef<Path>>(config_dir: P) -> Self {
        let config_dir = config_dir.as_ref();

        let dependencies = vec![];
        let templates = vec![
            // melonDS
            PipelineDefinition::new(
                PipelineDefinitionId::parse("d92ca87d-282f-4897-86f0-86a3af16bf3e"),
                "melonDS".to_string(),
                "Maps the internal and external monitor to a single virtual screen, as melonDS does not currently support multiple windows. Allows optional melonDS configuration editing.".to_string(),
                vec!["NDS".to_string()],
                HashMap::from_iter([
                    (PipelineTarget::Desktop, 
                        Selection::AllOf(vec![
                            PipelineActionDefinition {
                                id: PipelineActionDefinitionId::parse("d7838726-448c-4817-bcd6-d982c0bad7f6"),
                                description: Some("Edits melonDS ini file to desired settings".to_string()),
                                name: "melonDS Config".to_string(),
                                selection: MelonDSConfig { 
                                    ini_source: MelonDSIniSource::Flatpak, 
                                    layout_option: MelonDSLayoutOption::Vertical, 
                                    sizing_option: MelonDSSizingOption::Even, 
                                    book_mode: false, 
                                    swap_screens: false, 
                                }.into(),
                                enabled: Some(false)
                            },
                            PipelineActionDefinition {
                                enabled: None,
                                id: PipelineActionDefinitionId::parse("4ff26ece-dcab-4dd3-b941-96bd96a2c045"),
                                name: "Display Configuration".to_string(),
                                description: None,
                                selection: DisplayConfig {
                                        teardown_external_settings:TeardownExternalSettings::Previous,
                                        teardown_deck_location:RelativeLocation::Below
                                    }.into(),
                            },
                            PipelineActionDefinition {
                                selection: VirtualScreen.into(),
                                enabled: None,
                                id: PipelineActionDefinitionId::parse("2c843c15-fafa-4ee1-b960-e0e0aaa60882"), 
                                name: "Virtual Screen".to_string(), 
                                description: None,
                            },
                
                        ])
                    ),
                    (PipelineTarget::Gamemode, 
                        Selection::AllOf(vec![
                            PipelineActionDefinition {
                                id: PipelineActionDefinitionId::parse("6714f40e-fd68-49b6-8ed6-eea238b677b8"),
                                description: Some("Edits melonDS ini file to desired settings".to_string()),
                                name: "melonDS Config".to_string(),
                                selection: MelonDSConfig { 
                                    ini_source: MelonDSIniSource::Flatpak, 
                                    layout_option: MelonDSLayoutOption::Horizontal, 
                                    sizing_option: MelonDSSizingOption::Even, 
                                    book_mode: false, 
                                    swap_screens: false, 
                                }.into(),
                                enabled: Some(false)
                            },
                        ]),
                    )
                ]),
            ),

            // Citra
            PipelineDefinition::new(
                PipelineDefinitionId::parse("ed6fc4bb-ec6d-4b0f-97e7-70709066dcba"),
                "Citra".to_string(),
                "Maps primary and secondary windows to different screens for Citra. Allows optional Citra configuration editing.".to_string(),
                vec!["3DS".to_string()],
                HashMap::from_iter([
                    (PipelineTarget::Desktop, 
                        Selection::AllOf(vec![
                            PipelineActionDefinition {
                                id: PipelineActionDefinitionId::parse("d7838726-448c-4817-bcd6-d982c0bad7f6"),
                                description: Some("Edits Citra ini file to desired settings".to_string()),
                                name: "Citra Config".to_string(),
                                selection: CitraConfig { 
                                    ini_source: CitraIniSource::Flatpak, 
                                    layout_option: CitraLayoutOption::Default,
                                }.into(),
                                enabled: Some(true)
                            },
                            PipelineActionDefinition {
                                enabled: None,
                                id: PipelineActionDefinitionId::parse("4ff26ece-dcab-4dd3-b941-96bd96a2c045"),
                                name: "Display Configuration".to_string(),
                                description: None,
                                selection: DisplayConfig {
                                        teardown_external_settings:TeardownExternalSettings::Previous,
                                        teardown_deck_location:RelativeLocation::Below
                                    }.into(),
                            },
                            PipelineActionDefinition {
                                selection: MultiWindow.into(),
                                enabled: None,
                                id: PipelineActionDefinitionId::parse("2c843c15-fafa-4ee1-b960-e0e0aaa60882"), 
                                name: "MultiWindow".to_string(), 
                                description: None,
                            },
                        ])
                    ),
                    (PipelineTarget::Gamemode,
                        Selection::AllOf(vec![
                            PipelineActionDefinition {
                                id: PipelineActionDefinitionId::parse("f39d953b-7cbe-43b0-acf9-3d789d26fb8b"),
                                description: Some("Edits Citra ini file to desired settings".to_string()),
                                name: "Citra Config".to_string(),
                                selection: CitraConfig {
                                    ini_source: CitraIniSource::Flatpak,
                                    layout_option: CitraLayoutOption::SideBySide,
                                }.into(),
                                enabled: Some(true)
                            },
                        ]))
                ]),
            ),

            // Cemu
            PipelineDefinition::new(
                PipelineDefinitionId::parse("b0d6443d-6ae7-4085-87c1-b52aae5001a1"),
                "Cemu".to_string(),
                "Maps primary and secondary windows to different screens for Cemu.".to_string(),
                vec!["WIIU".to_string()],
                HashMap::from_iter([
                    (PipelineTarget::Desktop, 
                        Selection::AllOf(vec![
                            PipelineActionDefinition {
                                id: PipelineActionDefinitionId::parse("461c1ea6-8dd3-434e-b87d-8981c82b94c2"),
                                description: Some("Edits Cemu settings.xml file to desired settings".to_string()),
                                name: "Cemu Config".to_string(),
                                selection: CemuConfig {
                                    xml_source: CemuXmlSource::Flatpak,
                                    separate_gamepad_view: true,
                                }.into(),
                                enabled: Some(true),
                            },
                            PipelineActionDefinition {
                                enabled: None,
                                id: PipelineActionDefinitionId::parse("4ff26ece-dcab-4dd3-b941-96bd96a2c045"),
                                name: "Display Configuration".to_string(),
                                description: None,
                                selection: DisplayConfig {
                                        teardown_external_settings:TeardownExternalSettings::Previous,
                                        teardown_deck_location:RelativeLocation::Below
                                    }.into(),
                            },
                            PipelineActionDefinition {
                                selection: MultiWindow.into(),
                                enabled: None,
                                id: PipelineActionDefinitionId::parse("2c843c15-fafa-4ee1-b960-e0e0aaa60882"), 
                                name: "MultiWindow".to_string(), 
                                description: None,
                            },
                        ])
                    ),
                    (PipelineTarget::Gamemode, 
                        Selection::AllOf(vec![
                            PipelineActionDefinition {
                                id: PipelineActionDefinitionId::parse("f05d002b-9712-45e5-89e1-99f59aeeb25b"),
                                description: Some("Edits Cemu settings.xml file to desired settings".to_string()),
                                name: "Cemu Config".to_string(),
                                selection: CemuConfig {
                                    xml_source: CemuXmlSource::Flatpak,
                                    separate_gamepad_view: false,
                                }.into(),
                                enabled: Some(true),
                            },
                        ])
                    )
                ]),
            )
        ];

        Self {
            profiles_dir: config_dir.join("profiles"),
            apps_dir: config_dir.join("apps"),
            autostart_path: config_dir.join("autostart.json"),
            dependencies,
            templates,
        }
    }

    // File data

    pub fn create_profile(
        &self,
        name: String,
        template_id: &PipelineDefinitionId,
    ) -> Result<Profile> {
        // ensure template exists
        self.templates
            .iter()
            .find(|t| t.id == *template_id)
            .ok_or(anyhow::anyhow!(
                "Could not find matching template for {template_id:?}"
            ))?;

        Ok(Profile {
            name,
            id: ProfileId::new(),
            template: *template_id,
            tags: vec![],
            overrides: Overrides::default(),
        })
    }

    pub fn get_profile(&self, id: &ProfileId) -> Result<Profile> {
        let raw = id.raw();

        let profile_path = self.profiles_dir.join(raw).with_extension("json");
        let profile = std::fs::read_to_string(profile_path)?;

        Ok(serde_json::from_str(&profile)?)
    }

    pub fn set_profile(&self, profile: &Profile) -> Result<()> {
        create_dir_all(&self.profiles_dir)?;

        let raw = profile.id.raw();

        let serialized = serde_json::to_string_pretty(profile)?;
        let profile_path = self.profiles_dir.join(raw).with_extension("json");

        Ok(std::fs::write(profile_path, serialized)?)
    }

    pub fn get_profiles(&self) -> Result<Vec<Profile>> {
        std::fs::read_dir(&self.profiles_dir)?
            .filter_map(|f| {
                f.ok().map(|entry| {
                    if entry.path().ends_with(".json") {
                        let contents = std::fs::read_to_string(entry.path()).ok();
                        contents.map(|c| Ok(serde_json::from_str(&c).ok()))
                    } else {
                        None
                    }
                })
            })
            .flatten()
            .filter_map(|f| f.transpose())
            .collect::<Result<_>>()
    }

    pub fn get_app(&self, id: &AppId) -> Result<Option<App>> {
        let raw = id.raw();

        let app_path = self.apps_dir.join(raw).with_extension("json");

        if app_path.exists() {
            let app = std::fs::read_to_string(app_path)?;

            Ok(serde_json::from_str(&app)?)
        } else {
            Ok(None)
        }
    }

    pub fn set_app(&self, app: &App) -> Result<()> {
        create_dir_all(&self.apps_dir)?;

        let raw = app.id.raw();

        let serialized = serde_json::to_string_pretty(app)?;
        let app_path = self.apps_dir.join(raw).with_extension("json");

        Ok(std::fs::write(app_path, serialized)?)
    }

    pub fn get_autostart(&self) -> Result<Option<AutoStart>> {
        let autostart = std::fs::read_to_string(&self.autostart_path)?;

        Ok(serde_json::from_str(&autostart)?)
    }

    pub fn set_autostart_cfg(&self, autostart: &Option<AutoStart>) -> Result<()> {
        create_dir_all(
            self.autostart_path
                .parent()
                .expect("autostart.json path should have parent"),
        )?;

        let serialized = serde_json::to_string_pretty(autostart)?;

        Ok(std::fs::write(&self.autostart_path, serialized)?)
    }

    // In-memory configuration (currently readonly, but dependencies should ideally be configurable)

    pub fn get_template(&self, id: &PipelineDefinitionId) -> Option<&PipelineDefinition> {
        self.templates.iter().find(|t| t.id == *id)
    }

    pub fn get_templates(&self) -> &[PipelineDefinition] {
        &self.templates
    }

    pub fn get_dependencies(&self) -> &[Dependency] {
        &self.dependencies
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutoStart {
    pub app_id: AppId,
    pub profile_id: ProfileId,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Profile {
    pub name: String,
    pub id: ProfileId,
    pub template: PipelineDefinitionId,
    pub tags: Vec<String>,
    pub overrides: Overrides,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct App {
    id: AppId,
    active_profile: PipelineDefinitionId,
    pub overrides: HashMap<PipelineDefinitionId, Overrides>,
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
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct Overrides {
    pub fields: HashMap<PipelineActionDefinitionId, Value>,
    pub enabled: HashMap<PipelineActionDefinitionId, bool>,
}
