use std::{
    collections::HashMap,
    fmt::Debug,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    macros::{newtype_strid, newtype_uuid},
    pipeline::data::{
        ActionOrProfilePipeline, ActionPipeline, DefinitionPipeline, Enabled, PipelineActionId,
        PipelineTarget, Selection, Template, TemplateId,
    },
    util::create_dir_all,
    PACKAGE_NAME,
};

pub mod patch;

newtype_uuid!(ProfileId);
newtype_strid!("", AppId);

#[derive(Debug, Clone)]
pub struct Settings {
    // Path vars
    profiles_dir: PathBuf,
    apps_dir: PathBuf,
    system_autostart_dir: PathBuf,

    autostart_path: PathBuf,
    exe_path: PathBuf,

    // in-memory configurations -- consider moving
    templates: Vec<Template>,
}

impl Settings {
    pub fn new<P: AsRef<Path>>(exe_path: P, config_dir: P, system_autostart_dir: P) -> Self {
        let config_dir = config_dir.as_ref();

        let templates = vec![
            // melonDS
            Template {
                id: TemplateId::parse("c6430131-50e0-435e-a917-5ae3cfa46e3c"),
                pipeline: DefinitionPipeline::new(
                    "melonDS".to_string(),
                    "Maps the internal and external monitor to a single virtual screen, as melonDS does not currently support multiple windows. Allows optional melonDS layout configuration.".to_string(),
                    vec!["NDS".to_string(), "nds".to_string()],
                    HashMap::from_iter([
                        (PipelineTarget::Desktop, Selection::AllOf(vec![
                            Enabled::force(PipelineActionId::new("core:melonds:config")),
                            Enabled::force(PipelineActionId::new("core:display:display_config")),
                            Enabled::force(PipelineActionId::new("core:display:virtual_screen"))
                        ])),
                        (PipelineTarget::Gamemode, Selection::AllOf(vec![
                            Enabled::force(PipelineActionId::new("core:melonds:config")),
                        ]))
                    ]),
                )
            },

            // Citra
            Template {
                id: TemplateId::parse("fe82be74-22b9-4135-b7a0-cb6d8f51aecd"),
                pipeline: DefinitionPipeline::new(
                    "Citra".to_string(),
                    "Maps primary and secondary windows to different screens for Citra. Allows optional Citra layout configuration".to_string(),
                    vec!["3DS".to_string(),"3ds".to_string()],
                    HashMap::from_iter([
                        (PipelineTarget::Desktop, Selection::AllOf(vec![
                            Enabled::force(PipelineActionId::new("core:citra:config")),
                            Enabled::force(PipelineActionId::new("core:display:display_config")),
                            Enabled::force(PipelineActionId::new("core:display:multi_window"))
                        ])),
                        (PipelineTarget::Gamemode, Selection::AllOf(vec![
                            Enabled::force(PipelineActionId::new("core:citra:config")),
                        ]))
                    ]),
                )
            },

            // Cemu
            Template {
                id: TemplateId::parse("33c863e5-2739-4bc3-b9bc-4798bac8682d"),
                pipeline: DefinitionPipeline::new(
                    "Cemu".to_string(),
                    "Maps primary and secondary windows to different screens for Cemu.".to_string(),
                    vec!["WIIU".to_string(), "WiiU".to_string()],
                    HashMap::from_iter([
                        (PipelineTarget::Desktop,
                            Selection::AllOf(vec![
                                Enabled::force(PipelineActionId::new("core:cemu:config")),
                                Enabled::force(PipelineActionId::new("core:display:display_config")),
                                Enabled::force(PipelineActionId::new("core:display:multi_window"))
                        ])),
                        (PipelineTarget::Gamemode,
                            Selection::AllOf(vec![
                                Enabled::force(PipelineActionId::new("core:cemu:config"))
                        ]))
                    ]),
                )
            }
        ];

        Self {
            profiles_dir: config_dir.join("profiles"),
            apps_dir: config_dir.join("apps"),
            autostart_path: config_dir.join("autostart.json"),
            system_autostart_dir: system_autostart_dir.as_ref().to_owned(),
            exe_path: exe_path.as_ref().to_owned(),
            templates,
        }
    }

    // File data

    pub fn create_profile(&self, pipeline: ActionPipeline) -> Result<Profile> {
        Ok(Profile {
            id: ProfileId::new(),
            pipeline,
        })
    }

    pub fn delete_profile(&self, id: &ProfileId) -> Result<()> {
        let profile_path = self.profiles_dir.join(id.raw()).with_extension("json");
        std::fs::remove_file(profile_path)
            .with_context(|| format!("failed to remove profile settings {id:?}"))
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

    pub fn delete_app(&self, id: &AppId) -> Result<()> {
        let raw = id.raw();

        let app_path = self.apps_dir.join(raw).with_extension("json");
        std::fs::remove_file(app_path)
            .with_context(|| format!("failed to remove app settings for {id:?}"))
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

    pub fn get_autostart_cfg(&self) -> Option<AutoStart> {
        std::fs::read_to_string(&self.autostart_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
    }

    pub fn delete_autostart_cfg(&self) -> Result<()> {
        if self.autostart_path.exists() {
            std::fs::remove_file(&self.autostart_path)
                .with_context(|| "failed to remove autostart config")
        } else {
            Ok(())
        }
    }

    pub fn set_autostart_cfg(&self, autostart: &AutoStart) -> Result<()> {
        // always set system autostart, since we (eventually) want to be able to auto-configure displays
        // whether or not an app is run
        create_dir_all(&self.system_autostart_dir)?;

        let desktop_contents = self.create_desktop_contents();

        let autostart_parent = self
            .autostart_path
            .parent()
            .expect("autostart.json path should have parent");

        // set autostart config

        create_dir_all(autostart_parent)?;

        let autostart_cfg = serde_json::to_string_pretty(autostart)?;

        std::fs::write(
            self.system_autostart_dir
                .join(format!("{PACKAGE_NAME}.desktop")),
            desktop_contents,
        )
        .with_context(|| "failed to create autostart desktop file")
        .and_then(move |_| {
            std::fs::write(&self.autostart_path, autostart_cfg)
                .with_context(|| "failed to create autostart config file")
        })
    }

    fn create_desktop_contents(&self) -> String {
        r"[Desktop Entry]
Comment=Runs DeckDS plugin autostart script for dual screen applications.
Exec=$Exec
Path=$Path
Name=DeckDS
Type=Application"
            .replace(
                "$Exec",
                &format!(
                    "{} autostart",
                    self.exe_path
                        .to_str()
                        .expect("DeckDS server path should be valid unicode")
                ),
            )
            .replace(
                "$Path",
                self.exe_path
                    .parent()
                    .expect("DeckDS server path should have parent")
                    .to_str()
                    .expect("DeckDS server path should be valid unicode"),
            )
    }

    // In-memory configuration (currently readonly, but should ideally be configurable)
    pub fn get_template(&self, id: &TemplateId) -> Option<&Template> {
        self.templates.iter().find(|t| t.id == *id)
    }

    pub fn get_templates(&self) -> &[Template] {
        &self.templates
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutoStart {
    pub app_id: AppId,
    pub pipeline: ActionOrProfilePipeline,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Profile {
    pub id: ProfileId,
    pub pipeline: ActionPipeline,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct App {
    id: AppId,
    profiles: Vec<ActionOrProfilePipeline>,
}

#[cfg(test)]
mod tests {
    use crate::pipeline::{action::virtual_screen::VirtualScreen, data::PipelineAction};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_desktop_contents_correct() {
        let settings = Settings::new(
            Path::new("$HOME/homebrew/plugins")
                .join(PACKAGE_NAME)
                .join("bin/backend"),
            Path::new("$HOME/.config").join(PACKAGE_NAME),
            Path::new("$HOME/.config/autostart").to_path_buf(),
        );

        let actual = settings.create_desktop_contents();
        let expected = r"[Desktop Entry]
Comment=Runs DeckDS plugin autostart script for dual screen applications.
Exec=$HOME/homebrew/plugins/DeckDS/bin/backend autostart
Path=$HOME/homebrew/plugins/DeckDS/bin
Name=DeckDS
Type=Application";

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_profile_crud() -> Result<()> {
        let settings = Settings::new(
            Path::new("$HOME/homebrew/plugins/deck-ds/bin/backend"),
            Path::new("$HOME/.config/deck-ds"),
            Path::new("$HOME/.config/autostart"),
        );

        let mut expected = Profile {
            id: ProfileId::from_uuid(uuid::Uuid::nil()),
            pipeline: ActionPipeline {
                name: "Test Pipeline".to_string(),
                tags: vec!["TEST".to_string()],
                description: "Test Pipeline".to_string(),
                targets: HashMap::from_iter([(
                    PipelineTarget::Desktop,
                    Selection::AllOf(vec![Enabled::force(
                        PipelineAction {
                            name: "test action".to_string(),
                            id: PipelineActionId::new("test:test:action"),
                            description: None,
                            selection: VirtualScreen.into(),
                        }
                        .into(),
                    )]),
                )]),
            },
        };

        settings.set_profile(&expected)?;
        let actual = settings.get_profile(&expected.id)?;

        assert_eq!(expected.id, actual.id);
        assert_eq!(expected.pipeline.name, actual.pipeline.name);

        expected.pipeline.name = "Updated".to_string();

        settings.set_profile(&expected)?;

        let actual = settings.get_profile(&expected.id)?;

        assert_eq!(expected.id, actual.id);
        assert_eq!(expected.pipeline.name, actual.pipeline.name);

        settings.delete_profile(&expected.id)?;

        Ok(())
    }

    #[test]
    fn test_app_crud() -> Result<()> {
        let settings = Settings::new(
            Path::new("$HOME/homebrew/plugins/deck-ds/bin/backend"),
            Path::new("$HOME/.config/deck-ds"),
            Path::new("$HOME/.config/autostart"),
        );

        let mut expected = App {
            id: AppId("test_app".to_string()),
            profiles: vec![ActionOrProfilePipeline {
                name: "Test Pipeline".to_string(),
                tags: vec!["TEST".to_string()],
                description: "Test Pipeline".to_string(),
                targets: HashMap::from_iter([(PipelineTarget::Desktop, Selection::AllOf(vec![]))]),
            }],
        };

        settings.set_app(&expected)?;
        let actual = settings
            .get_app(&expected.id)?
            .with_context(|| "app should exist")?;

        assert_eq!(expected.id, actual.id);
        assert_eq!(expected.profiles[0].name, actual.profiles[0].name);

        expected.profiles[0].name = "Updated".to_string();

        settings.set_app(&expected)?;

        let actual = settings
            .get_app(&expected.id)?
            .with_context(|| "app should exist")?;

        assert_eq!(expected.id, actual.id);
        assert_eq!(expected.profiles[0].name, actual.profiles[0].name);

        settings.delete_app(&expected.id)?;

        Ok(())
    }
}
