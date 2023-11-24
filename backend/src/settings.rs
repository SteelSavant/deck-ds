use std::{
    collections::HashMap,
    fmt::Debug,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    macros::{newtype_strid, newtype_uuid},
    pipeline::config::{
        Enabled, PipelineActionDefinitionId, PipelineTarget, Selection, TemplateDefinition,
        TemplateDefinitionId,
    },
    util::create_dir_all,
};

pub mod patch;

newtype_uuid!(ProfileId);
newtype_strid!("", AppId);

#[derive(Debug, Clone)]
pub struct Settings {
    // Path vars
    profiles_dir: PathBuf,
    apps_dir: PathBuf,
    autostart_path: PathBuf,
    system_autostart_dir: PathBuf,

    // in-memory configurations -- consider moving
    templates: Vec<TemplateDefinition>,
}

impl Settings {
    pub fn new<P: AsRef<Path>>(config_dir: P) -> Self {
        let config_dir = config_dir.as_ref();

        let templates = vec![
            // melonDS
            TemplateDefinition::new(
                TemplateDefinitionId::parse("d92ca87d-282f-4897-86f0-86a3af16bf3e"),
                "melonDS".to_string(),
                "Maps the internal and external monitor to a single virtual screen, as melonDS does not currently support multiple windows. Allows optional melonDS layout configuration.".to_string(),
                vec!["NDS".to_string(), "nds".to_string()],
                HashMap::from_iter([
                    (PipelineTarget::Desktop, Selection::AllOf(vec![
                        Enabled::force(PipelineActionDefinitionId::new("core:melonds:layout")),
                        Enabled::force(PipelineActionDefinitionId::new("core:display:display_config")), 
                        Enabled::force(PipelineActionDefinitionId::new("core:display:virtual_screen"))
                    ])),
                    (PipelineTarget::Gamemode, Selection::AllOf(vec![
                        Enabled::force(PipelineActionDefinitionId::new("core:melonds:layout")),
                    ]))
                ]),
            ),

            // Citra
            TemplateDefinition::new(
                TemplateDefinitionId::parse("ed6fc4bb-ec6d-4b0f-97e7-70709066dcba"),
                "Citra".to_string(),
                "Maps primary and secondary windows to different screens for Citra. Allows optional Citra layout configuration".to_string(),
                vec!["3DS".to_string(),"3ds".to_string()],
                HashMap::from_iter([
                    (PipelineTarget::Desktop, Selection::AllOf(vec![
                        Enabled::force(PipelineActionDefinitionId::new("core:citra:layout")),
                        Enabled::force(PipelineActionDefinitionId::new("core:display:display_config")), 
                        Enabled::force(PipelineActionDefinitionId::new("core:display:multi_window"))
                    ])),
                    (PipelineTarget::Gamemode, Selection::AllOf(vec![
                        Enabled::force(PipelineActionDefinitionId::new("core:citra:layout")),
                    ]))
                ]),
            ),

            // Cemu
            TemplateDefinition::new(
                TemplateDefinitionId::parse("b0d6443d-6ae7-4085-87c1-b52aae5001a1"),
                "Cemu".to_string(),
                "Maps primary and secondary windows to different screens for Cemu.".to_string(),
                vec!["WIIU".to_string(), "WiiU".to_string()],
                HashMap::from_iter([
                    (PipelineTarget::Desktop,
                        Selection::AllOf(vec![
                            Enabled::force(PipelineActionDefinitionId::new("core:cemu:layout")),
                            Enabled::force(PipelineActionDefinitionId::new("core:display:display_config")), 
                            Enabled::force(PipelineActionDefinitionId::new("core:display:multi_window"))
                    ])),
                    (PipelineTarget::Gamemode,
                        Selection::AllOf(vec![
                            Enabled::force(PipelineActionDefinitionId::new("core:cemu:layout"))
                    ]))
                ]),
            )
        ];

        Self {
            profiles_dir: config_dir.join("profiles"),
            apps_dir: config_dir.join("apps"),
            autostart_path: config_dir.join("autostart.json"),
            system_autostart_dir: config_dir.join("../autostart"), // quick hack
            templates,
        }
    }

    // File data

    pub fn create_profile(
        &self,
        name: String,
        template_id: &TemplateDefinitionId,
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
        // always set system autostart, since we (eventually) want to be able to auto-configure displays
        // whether or not an app is run
        create_dir_all(&self.system_autostart_dir)?;

        let autostart_parent = self
            .autostart_path
            .parent()
            .expect("autostart.json path should have parent");

        let desktop_contents = r"[Desktop Entry]
        Comment=Runs DeckDS plugin autostart script for dual screen applications.
        Exec=$Exec
        Path=$Path
        Name=DeckDS
        Type=Application"
            .replace("$Exec", "$HOME/homebrew/plugins/DeckDS/bin/backend") // hardcode for now
            .replace("$Path", &autostart_parent.to_string_lossy());

        // set autostart config
        create_dir_all(autostart_parent)?;

        let autostart_cfg = serde_json::to_string_pretty(autostart)?;

        std::fs::write(
            self.system_autostart_dir.join("deck-ds.desktop"),
            desktop_contents,
        )
        .with_context(|| "failed to create autostart desktop file")
        .and_then(move |_| {
            std::fs::write(&self.autostart_path, autostart_cfg)
                .with_context(|| "failed to create autostart config file")
        })
    }

    // In-memory configuration (currently readonly, but should ideally be configurable)
    pub fn get_template(&self, id: &TemplateDefinitionId) -> Option<&TemplateDefinition> {
        self.templates.iter().find(|t| t.id == *id)
    }

    pub fn get_templates(&self) -> &[TemplateDefinition] {
        &self.templates
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
    pub template: TemplateDefinitionId,
    pub tags: Vec<String>,
    pub overrides: Overrides,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct App {
    id: AppId,
    active_profile: TemplateDefinitionId,
    pub overrides: HashMap<TemplateDefinitionId, Overrides>,
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
