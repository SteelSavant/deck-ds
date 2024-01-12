use std::{
    collections::HashMap,
    fmt::Debug,
    path::{Path, PathBuf},
};


use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

newtype_uuid!(ProfileId);
newtype_strid!("", AppId);


use crate::{
    macros::{newtype_strid, newtype_uuid},
    pipeline::{
        action::display_restoration::DisplayRestoration,
        action_registar::PipelineActionRegistrar,
        data::{
            Pipeline, PipelineActionId, PipelineDefinition, PipelineTarget, Selection, Template,
            TemplateId,
        },
    },
    util::create_dir_all,
    PACKAGE_NAME,
};

use self::db::SettingsDb;

mod db;

pub struct Settings {
    // Path vars
    system_autostart_dir: PathBuf,

    global_config_path: PathBuf,
    autostart_path: PathBuf,
    exe_path: PathBuf,

    // db
    db: SettingsDb,

    // in-memory templates -- consider moving
    templates: Vec<Template>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GlobalConfig {
    pub display_restoration: DisplayRestoration,
    pub restore_displays_if_not_executing_pipeline: bool,
    // other global settings as needed
}


impl Settings {
    pub fn new<P: AsRef<Path>>(exe_path: P, config_dir: P, system_autostart_dir: P, registrar: PipelineActionRegistrar) -> Self {
        let config_dir = config_dir.as_ref();


        let templates = build_templates(registrar);

        if !config_dir.exists() {
            create_dir_all(config_dir).unwrap();
        }

        let db_path = config_dir.join("profiles.db");
        let db = SettingsDb::new(db_path);

        Self {
            autostart_path: config_dir.join("autostart.json"),
            global_config_path: config_dir.join("config.json"),
            db,
            system_autostart_dir: system_autostart_dir.as_ref().to_owned(),
            exe_path: exe_path.as_ref().to_owned(),
            templates,
        }
    }

    // File data

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

    pub fn get_global_cfg(&self) -> GlobalConfig {
        std::fs::read_to_string(&self.global_config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn delete_global_cfg(&self) -> Result<()> {
        if self.global_config_path.exists() {
            std::fs::remove_file(&self.global_config_path)
                .with_context(|| "failed to remove global config")
        } else {
            Ok(())
        }
    }

    pub fn set_global_cfg(&self, global: &GlobalConfig) -> Result<()> {
        let global_parent = self
            .global_config_path
            .parent()
            .expect("config.json path should have parent");

        // set global config

        create_dir_all(global_parent)?;

        let global_cfg = serde_json::to_string_pretty(global)?;

        std::fs::write(&self.global_config_path, global_cfg)
            .with_context(|| "failed to create autostart config file")
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

    // Db wrapper
    pub fn create_profile(&self, pipeline: PipelineDefinition) -> Result<CategoryProfile> {
        self.db.create_profile(pipeline)
    }

    pub fn delete_profile(&self, id: &ProfileId) -> Result<()> {
        self.db.delete_profile(id)
    }

    pub fn set_profile(&self, profile: CategoryProfile) -> Result<()> {
        self.db.set_profile(profile)
    }

    pub fn get_profile(&self, id: &ProfileId) -> Result<Option<CategoryProfile>> {
        self.db.get_profile(id)
    }

    pub fn get_profiles(&self) -> Result<Vec<CategoryProfile>> {
        self.db.get_profiles()
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
    pub pipeline: Pipeline,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CategoryProfile {
    pub id: ProfileId,
    pub tags: Vec<String>,
    pub pipeline: PipelineDefinition,
}

#[derive(Debug, Deserialize, Serialize)]

pub struct AppProfile {
    pub id: AppId,
    pub profiles: HashMap<ProfileId, PipelineDefinition>
}

fn build_templates(registrar: PipelineActionRegistrar) -> Vec<Template> {
    struct TemplateBuilder {
        id: TemplateId,
        name: String,
        description: String,
        targets: HashMap<PipelineTarget, Selection<PipelineActionId>>
    }

    impl TemplateBuilder {
        fn build(self, registrar: &PipelineActionRegistrar)-> Template {
            let actions = registrar.make_lookup(&self.targets);
            Template {
                id: self.id,
                pipeline: PipelineDefinition {
                    name: self.name,
                    description: self.description,
                    targets: self.targets,
                    actions,
                },
            }
        }
    }

    let templates = vec![
        // melonDS
        TemplateBuilder {
            id: TemplateId::parse("c6430131-50e0-435e-a917-5ae3cfa46e3c"),
            name: "melonDS".to_string(),
            description: "Maps the internal and external monitor to a single virtual screen, as melonDS does not currently support multiple windows. Allows optional melonDS layout configuration.".to_string(),
            targets: HashMap::from_iter([
                (PipelineTarget::Desktop, Selection::AllOf(vec![
                    PipelineActionId::new("core:melonds:config"),
                    PipelineActionId::new("core:display:virtual_screen"),
                ])),
                (PipelineTarget::Gamemode, Selection::AllOf(vec![
                    PipelineActionId::new("core:melonds:config"),
                ]))
            ]),  
        },

        // Citra
        TemplateBuilder {
            id: TemplateId::parse("fe82be74-22b9-4135-b7a0-cb6d8f51aecd"),
            name: "Citra".to_string(),
            description: "Maps primary and secondary windows to different screens for Citra. Allows optional Citra layout configuration".to_string(),
            targets: HashMap::from_iter([
                (PipelineTarget::Desktop, Selection::AllOf(vec![
                    PipelineActionId::new("core:citra:config"),
                    PipelineActionId::new("core:display:multi_window"),
                ])),
                (PipelineTarget::Gamemode, Selection::AllOf(vec![
                    PipelineActionId::new("core:citra:config"),
                ]))
            ]),   
        },

        // Cemu
        TemplateBuilder {
            id: TemplateId::parse("33c863e5-2739-4bc3-b9bc-4798bac8682d"),
            name: "Cemu".to_string(),
            description: "Maps primary and secondary windows to different screens for Cemu.".to_string(),
            targets: HashMap::from_iter([
                (PipelineTarget::Desktop,
                    Selection::AllOf(vec![
                        PipelineActionId::new("core:cemu:config"),
                        PipelineActionId::new("core:display:multi_window"),
                ])),
                (PipelineTarget::Gamemode,
                    Selection::AllOf(vec![
                        PipelineActionId::new("core:cemu:config")
                ]))
            ]),  
        }
    ];

    templates.into_iter().map(|t| t.build(&registrar)).collect()
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_desktop_contents_correct() {
        let settings = Settings::new(
            Path::new("test/out/homebrew/plugins")
                .join(PACKAGE_NAME)
                .join("bin/backend"),
            Path::new("test/out/.config").join(PACKAGE_NAME),
            Path::new("test/out/.config/autostart").to_path_buf(),
            PipelineActionRegistrar::builder().with_core().build(),
        );

        let actual = settings.create_desktop_contents();
        let expected = r"[Desktop Entry]
Comment=Runs DeckDS plugin autostart script for dual screen applications.
Exec=test/out/homebrew/plugins/DeckDS/bin/backend autostart
Path=test/out/homebrew/plugins/DeckDS/bin
Name=DeckDS
Type=Application";

        assert_eq!(expected, actual);
    }
}
