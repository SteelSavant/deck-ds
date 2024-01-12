use std::{
    collections::HashMap,
    fmt::Debug,
    path::{Path, PathBuf}, sync::{Arc, Mutex},
};

use once_cell::sync::Lazy;

use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use native_db::*;
use native_model::{native_model, Model};

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


newtype_uuid!(ProfileId);
newtype_strid!("", AppId);


pub struct Settings {
    // Path vars

    system_autostart_dir: PathBuf,

    global_config_path: PathBuf,
    autostart_path: PathBuf,
    exe_path: PathBuf,

    // Database
    db: Database<'static>,

    // in-memory templates -- consider moving
    templates: Vec<Template>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GlobalConfig {
    pub display_restoration: DisplayRestoration,
    pub restore_displays_if_not_executing_pipeline: bool,
    // other global settings as needed
}


static DATABASE_BUILDER: Lazy<native_db::DatabaseBuilder> = Lazy::new(|| {
    let mut builder = DatabaseBuilder::new();

    builder.define::<CategoryProfile>().expect("failed to define CategoryProfile v1");
    builder.define::<AppProfile>().expect("failed to define AppProfile v1");

    builder
});

impl Settings {
    pub fn new<P: AsRef<Path>>(exe_path: P, config_dir: P, system_autostart_dir: P, registrar: PipelineActionRegistrar) -> Self {
        let config_dir = config_dir.as_ref();


        let templates = build_templates(registrar);

        if !config_dir.exists() {
            create_dir_all(config_dir).unwrap();
        }

        let db_path = config_dir.join("profiles.db");
        let db = DATABASE_BUILDER.create(db_path).expect("database should be instantiable");

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

    pub fn create_profile(&self, pipeline: PipelineDefinition) -> Result<CategoryProfile> {
        let id =  ProfileId::new();
        let profile = CategoryProfile {
            id: id.clone(),
            tags: vec![],
            pipeline,
        };

        let rw = self.db.rw_transaction().expect("failed to create rw_transaction");
        rw.insert(profile)?;

        let profile = rw.get().primary(id)?.expect("inserted profile should exist");
        rw.commit()?;

        Ok(profile)
    }

    pub fn delete_profile(&self, id: &ProfileId) -> Result<()> {
        let rw = self.db.rw_transaction().expect("failed to create rw_transaction");
        let profile = rw.get().primary(*id)?;
        profile.map(|p: CategoryProfile | {
            rw.remove(p).and_then(|_| {
                rw.commit()
            })
        });

        Ok(())
    }

    pub fn get_profile(&self, id: &ProfileId) -> Result<Option<CategoryProfile>> {
        let ro = self.db.r_transaction().expect("failed to create ro_transaction");
        let profile = ro.get().primary(*id)?;

        Ok(profile)
    }

    pub fn set_profile(&self, profile: CategoryProfile) -> Result<()> {
        let rw = self.db.rw_transaction().expect("failed to create rw_transaction");
        rw.insert(profile)?;
        Ok(rw.commit()?)
    }

    pub fn get_profiles(&self) -> Result<Vec<CategoryProfile>> {
        let ro = self.db.r_transaction().expect("failed to create ro_transaction");
        let profiles: Vec<CategoryProfile>  = ro.scan().primary().expect("failed to scan category profiles").all().collect();
        Ok(profiles)
    }

    // pub fn delete_app(&self, id: &AppId) -> Result<()> {
    //      let rw = self.db.rw_transaction().expect("failed to create rw_transaction");

    //      rw.remove(item)

    // }

    // pub fn get_app(&self, id: &AppId) -> Result<Option<AppProfile>> {
    //     create_dir_all(&self.apps_dir)?;

    //     let raw = id.raw();

    //     let app_path = self.apps_dir.join(raw).with_extension("json");

    //     if app_path.exists() {
    //         let app = std::fs::read_to_string(app_path)?;

    //         Ok(serde_json::from_str(&app)?)
    //     } else {
    //         Ok(None)
    //     }
    // }

    // pub fn set_app(&self, app: &AppProfile) -> Result<()> {
    //     create_dir_all(&self.apps_dir)?;

    //     let raw = app.id.raw();

    //     let serialized = serde_json::to_string_pretty(app)?;
    //     let app_path = self.apps_dir.join(raw).with_extension("json");

    //     Ok(std::fs::write(app_path, serialized)?)
    // }

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

pub type CategoryProfile = v1::CategoryProfile;
pub type AppProfile = v1::AppProfile;

pub mod v1 {
    use crate::native_model_serde_json::NativeModelJSON;

    use super::*;

    #[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
    #[native_model(id = 1, version = 1, with = NativeModelJSON)]
    #[native_db]
    pub struct CategoryProfile {
        #[primary_key]
        pub id: ProfileId,
        pub tags: Vec<String>,
        pub pipeline: v1::PipelineDefinition,
    }

    #[derive(Debug, Deserialize, Serialize)]
    #[native_model(id = 2, version = 1, with = NativeModelJSON)]
    #[native_db]
    pub struct AppProfile {
        #[primary_key]
        pub id: AppId,
        pub profiles: HashMap<ProfileId, v1::PipelineDefinition>
    }
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
    use uuid::Uuid;

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

    #[test]
    fn test_profile_crud() -> Result<()> {
        let registrar = PipelineActionRegistrar::builder().with_core().build();
        let settings = Settings::new(
            Path::new("test/out/homebrew/plugins/deck-ds/bin/backend"),
            Path::new("test/out/.config/deck-ds"),
            Path::new("test/out/.config/autostart"),
            registrar.clone(),
        );

        let targets = HashMap::from_iter([(
                    PipelineTarget::Desktop,
                    Selection::AllOf(vec![PipelineActionId::new("core:citra:layout")]),
                )]);

                let actions = registrar.make_lookup(&targets);

        let mut expected: CategoryProfile = CategoryProfile {
            id: ProfileId::from_uuid(Uuid::nil()),
            tags: vec!["Test".to_string()],
            pipeline: PipelineDefinition {
                name: "Test Pipeline".to_string(),
                description: "Test Description".to_string(),
                targets,
                actions,
            },
        };

        settings.set_profile(expected.clone())?;
        let actual = settings
            .get_profile(&expected.id)?
            .expect("profile should exist");

        assert_eq!(expected.id, actual.id);
        assert_eq!(expected.pipeline.name, actual.pipeline.name);

        expected.pipeline.name = "Updated".to_string();

        settings.set_profile(expected.clone())?;

        let actual = settings
            .get_profile(&expected.id)?
            .expect("saved profile should exist");

        assert_eq!(expected.id, actual.id);
        assert_eq!(expected.pipeline.name, actual.pipeline.name);

        let actual = settings
            .get_profiles()?
            .get(0)
            .cloned()
            .expect("get_profiles should find 1 profile");

        assert_eq!(expected.id, actual.id);
        assert_eq!(expected.pipeline.name, actual.pipeline.name);

        settings.delete_profile(&expected.id)?;

        Ok(())
    }

    // #[test]
    // fn test_app_crud() -> Result<()> {
    //     let settings = Settings::new(
    //         Path::new("$HOME/homebrew/plugins/deck-ds/bin/backend"),
    //         Path::new("$HOME/.config/deck-ds"),
    //         Path::new("$HOME/.config/autostart"),
    //     );

    //     let mut expected = App {
    //         id: AppId("test_app".to_string()),
    //         profiles: vec![ActionOrProfilePipeline {
    //             name: "Test Pipeline".to_string(),
    //             tags: vec!["TEST".to_string()],
    //             description: "Test Pipeline".to_string(),
    //             targets: HashMap::from_iter([(PipelineTarget::Desktop, Selection::AllOf(vec![]))]),
    //         }],
    //     };

    //     settings.set_app(&expected)?;
    //     let actual = settings
    //         .get_app(&expected.id)?
    //         .with_context(|| "app should exist")?;

    //     assert_eq!(expected.id, actual.id);
    //     assert_eq!(expected.profiles[0].name, actual.profiles[0].name);

    //     expected.profiles[0].name = "Updated".to_string();

    //     settings.set_app(&expected)?;

    //     let actual = settings
    //         .get_app(&expected.id)?
    //         .with_context(|| "app should exist")?;

    //     assert_eq!(expected.id, actual.id);
    //     assert_eq!(expected.profiles[0].name, actual.profiles[0].name);

    //     settings.delete_app(&expected.id)?;

    //     Ok(())
    // }
}
