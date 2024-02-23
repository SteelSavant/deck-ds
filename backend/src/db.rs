use std::path::PathBuf;

use native_db::transaction::{RTransaction, RwTransaction};
use native_db::Database;

use crate::pipeline::action_registar::PipelineActionRegistrar;
use crate::pipeline::data::PipelineDefinition;
use crate::pipeline::data::Template;
use crate::pipeline::data::TemplateId;
use crate::settings::AppId;
use crate::settings::AppProfile;

use self::model::{DbAppOverride, DbAppSettings, DATABASE_BUILDER};
use self::templates::build_templates;
use self::{migrate::Migrate, model::DbCategoryProfile};

use crate::settings::CategoryProfile;

use crate::settings::ProfileId;
use anyhow::Result;

mod convert;
mod migrate;
mod model;
mod templates;

pub struct ProfileDb {
    db: Database<'static>,
    templates: Vec<Template>,
}

impl ProfileDb {
    pub fn new(db_path: PathBuf, registrar: PipelineActionRegistrar) -> Self {
        let db = DATABASE_BUILDER
            .create(db_path)
            .expect("database should be instantiable");

        db.migrate().expect("db migrations should succeed");

        let templates = build_templates(registrar);

        ProfileDb { db, templates }
    }

    pub fn create_profile(&self, pipeline: PipelineDefinition) -> Result<CategoryProfile> {
        let id = ProfileId::new();
        let profile = CategoryProfile {
            id,
            tags: vec![],
            pipeline,
        };

        let rw = self.read_write();
        profile.save_all(&rw)?;

        let saved = rw
            .get()
            .primary::<DbCategoryProfile>(id)?
            .expect("inserted profile should exist");

        rw.commit()?;

        // Ideally, the reconstruct would happen inside the rw transaction,
        // but the db types make that more complicated than I'd like

        let ro = self.read_only();

        let profile = saved.reconstruct(&ro)?;

        Ok(profile)
    }

    pub fn delete_profile(&self, id: &ProfileId) -> Result<()> {
        let rw = self.read_write();
        let profile = rw.get().primary::<DbCategoryProfile>(*id)?;
        if let Some(profile) = profile {
            profile.remove_all(&rw)?;
            rw.commit()?;
        }

        Ok(())
    }

    pub fn get_profile(&self, id: &ProfileId) -> Result<Option<CategoryProfile>> {
        let ro = self.read_only();
        let profile = ro.get().primary::<DbCategoryProfile>(*id)?;

        profile.map(|p| p.reconstruct(&ro)).transpose()
    }

    pub fn set_profile(&self, profile: CategoryProfile) -> Result<()> {
        let rw = self.read_write();
        profile.save_all(&rw)?;
        Ok(rw.commit()?)
    }

    pub fn get_profiles(&self) -> Result<Vec<CategoryProfile>> {
        let ro = self.read_only();
        let profiles = ro
            .scan()
            .primary::<DbCategoryProfile>()
            .expect("failed to scan category profiles")
            .all()
            .map(|p| p.reconstruct(&ro))
            .collect::<Result<_>>()?;
        Ok(profiles)
    }

    pub fn get_app_profile(&self, id: &AppId) -> Result<AppProfile> {
        let ro = self.read_only();

        AppProfile::load(id, &ro)
    }

    pub fn set_app_profile_override(
        &self,
        app_id: AppId,
        profile_id: ProfileId,
        definition: PipelineDefinition,
    ) -> Result<()> {
        let rw = self.read_write();
        rw.insert(DbAppOverride {
            id: (app_id, profile_id),
            pipeline: definition.save_all_and_transform(&rw)?,
        })?;

        Ok(rw.commit()?)
    }

    pub fn set_app_profile_settings(
        &self,
        app_id: AppId,
        default_profile: Option<ProfileId>,
    ) -> Result<()> {
        let rw = self.read_write();

        rw.insert(DbAppSettings {
            app_id,
            default_profile,
        })?;

        Ok(rw.commit()?)
    }

    // In-memory configuration (currently readonly, but should ideally be configurable)
    pub fn get_template(&self, id: &TemplateId) -> Option<&Template> {
        self.templates.iter().find(|t| t.id == *id)
    }

    pub fn get_templates(&self) -> &[Template] {
        &self.templates
    }

    fn read_only(&self) -> RTransaction {
        self.db
            .r_transaction()
            .expect("failed to create ro_transaction")
    }

    fn read_write(&self) -> RwTransaction {
        self.db
            .rw_transaction()
            .expect("failed to create rw_transaction")
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use std::{collections::HashMap, hash::RandomState};

    use uuid::Uuid;

    use crate::{
        pipeline::{
            action_registar::PipelineActionRegistrar,
            data::{PipelineActionId, PipelineDefinitionId, PipelineTarget},
        },
        util::create_dir_all,
    };

    use super::*;

    #[test]
    fn test_profile_crud() -> Result<()> {
        let registrar = PipelineActionRegistrar::builder().with_core().build();

        let path: PathBuf = "test/out/.config/deck-ds/profile_crud.db".into();
        if path.exists() {
            std::fs::remove_file(&path)?;
        }

        let parent = path.parent().unwrap();
        create_dir_all(parent).unwrap();

        let db = ProfileDb::new(path.clone(), registrar.clone());

        let pipeline_action_id = PipelineActionId::new("core:citra:layout");

        let targets =
            HashMap::from_iter([(PipelineTarget::Desktop, vec![pipeline_action_id.clone()])]);

        let actions = registrar.make_lookup(&targets);

        let mut expected: CategoryProfile = CategoryProfile {
            id: ProfileId::from_uuid(Uuid::nil()),
            tags: vec!["Test".to_string()],
            pipeline: PipelineDefinition {
                id: PipelineDefinitionId::nil(),
                name: "Test Pipeline".to_string(),
                description: "Test Description".to_string(),
                source_template: Default::default(),
                register_exit_hooks: true,
                primary_target_override: None,
                targets,
                actions,
            },
        };

        db.set_profile(expected.clone())?;
        let actual = db.get_profile(&expected.id)?.expect("profile should exist");
        let actual_action = actual
            .pipeline
            .actions
            .get(&pipeline_action_id, PipelineTarget::Desktop, &registrar)
            .expect("saved action should exist");

        assert_eq!(expected.id, actual.id);
        assert_eq!(expected.pipeline.name, actual.pipeline.name);

        expected.pipeline.name = "Updated".to_string();

        let mut expected_action = actual_action.clone();
        expected_action.settings.enabled = expected_action.settings.enabled.map(|v| !v);
        expected_action.settings.is_visible_on_qam = !expected_action.settings.is_visible_on_qam;

        expected.pipeline.actions.actions.insert(
            PipelineActionId::new("core:citra:layout:desktop"),
            expected_action.settings.clone().into(),
        );

        db.set_profile(expected.clone())?;

        let actual = db
            .get_profile(&expected.id)?
            .expect("saved profile should exist");
        let actual_action =
            actual
                .pipeline
                .actions
                .get(&pipeline_action_id, PipelineTarget::Desktop, &registrar);

        assert_eq!(expected.id, actual.id);
        assert_eq!(expected.pipeline.name, actual.pipeline.name);
        assert_eq!(Some(expected_action), actual_action);

        let actual = db
            .get_profiles()?
            .get(0)
            .cloned()
            .expect("get_profiles should find 1 profile");

        assert_eq!(expected.id, actual.id);

        db.delete_profile(&expected.id)?;

        assert!(db.get_profile(&expected.id)?.is_none());

        std::fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn test_app_crud() -> Result<()> {
        let registrar = PipelineActionRegistrar::builder().with_core().build();

        let path: PathBuf = "test/out/.config/deck-ds/app_crud.db".into();
        if path.exists() {
            std::fs::remove_file(&path)?;
        }

        let parent = path.parent().unwrap();
        create_dir_all(parent).unwrap();

        let db = ProfileDb::new(path.clone(), registrar.clone());

        let app_id = AppId::new("appid");
        let pd_id_1 = PipelineDefinitionId::new();
        let pd_id_2 = PipelineDefinitionId::new();

        let profile1 = ProfileId::new();
        let profile2 = ProfileId::new();

        let targets1 = HashMap::new();
        let actions1 = registrar.make_lookup(&targets1);

        let targets2 = HashMap::new();
        let actions2 = registrar.make_lookup(&targets2);

        let overrides: HashMap<_, _, RandomState> = HashMap::from_iter(vec![
            (
                profile1,
                PipelineDefinition {
                    id: pd_id_1,
                    name: "Profile 1".into(),
                    description: "Profile 1".into(),
                    source_template: Default::default(),
                    register_exit_hooks: true,
                    primary_target_override: None,
                    targets: targets1,
                    actions: actions1,
                },
            ),
            (
                profile2,
                PipelineDefinition {
                    id: pd_id_2,
                    name: "Profile 2".into(),
                    description: "Profile 2".into(),
                    source_template: Default::default(),
                    register_exit_hooks: true,
                    primary_target_override: None,
                    targets: targets2,
                    actions: actions2,
                },
            ),
        ]);

        // app we're testing
        db.set_app_profile_override(app_id.clone(), profile1, overrides[&profile1].clone())?;
        db.set_app_profile_override(app_id.clone(), profile2, overrides[&profile2].clone())?;

        // dummy app to ensure only correct overrides are loaded
        db.set_app_profile_override(
            AppId::new("dummyapp"),
            ProfileId::nil(),
            overrides[&profile1].clone(),
        )?;

        db.set_app_profile_settings(app_id.clone(), Some(profile2))?;

        let expected = AppProfile {
            id: app_id.clone(),
            default_profile: Some(profile2),
            overrides,
        };

        let actual = db.get_app_profile(&app_id)?;

        assert_eq!(expected, actual);

        std::fs::remove_file(&path)?;

        Ok(())
    }
}
