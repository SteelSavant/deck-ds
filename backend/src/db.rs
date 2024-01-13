use std::path::PathBuf;

use native_db::{Database, DatabaseBuilder};
use once_cell::sync::Lazy;

use crate::pipeline::action_registar::PipelineActionRegistrar;
use crate::pipeline::data::PipelineDefinition;
use crate::pipeline::data::Template;
use crate::pipeline::data::TemplateId;

use self::templates::build_templates;
use self::{migrate::Migrate, model::DbCategoryProfile};

use crate::settings::CategoryProfile;

use crate::settings::ProfileId;
use anyhow::Result;

mod convert;
mod migrate;
mod model;
mod templates;

use model::v1;

static DATABASE_BUILDER: Lazy<native_db::DatabaseBuilder> = Lazy::new(|| {
    let mut builder = DatabaseBuilder::new();
    // V1
    {
        // Profiles

        builder
            .define::<v1::DbCategoryProfile>()
            .expect("failed to define CategoryProfile v1");
        builder
            .define::<v1::DbAppProfile>()
            .expect("failed to define AppProfile v1");

        // Actions

        builder
            .define::<v1::DbCemuLayout>()
            .expect("failed to define CemuLayout v1");
        builder
            .define::<v1::DbCitraLayout>()
            .expect("failed to define CitraLayout v1");
        builder
            .define::<v1::DbMelonDSLayout>()
            .expect("failed to define MelonDSLayout v1");
        builder
            .define::<v1::DbDisplayRestoration>()
            .expect("failed to define DisplayRestoration v1");
        builder
            .define::<v1::DbMultiWindow>()
            .expect("failed to define MultiWindow v1");
        builder
            .define::<v1::DbVirtualScreen>()
            .expect("failed to define VirtualScreen v1");
        builder
            .define::<v1::DbSourceFile>()
            .expect("failed to define SourceDbSourceFile v1");
    }

    builder
});

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

        let rw = self
            .db
            .rw_transaction()
            .expect("failed to create rw_transaction");
        profile.save_all(&rw)?;

        let saved = rw
            .get()
            .primary::<DbCategoryProfile>(id)?
            .expect("inserted profile should exist");

        rw.commit()?;

        // Ideally, the reconstruct would happen inside the rw transaction,
        // but the db types make that more complicated than I'd like

        let ro = self
            .db
            .r_transaction()
            .expect("Failed to create ro_transaction");

        let profile = saved.reconstruct(&ro)?;

        Ok(profile)
    }

    pub fn delete_profile(&self, id: &ProfileId) -> Result<()> {
        let rw = self
            .db
            .rw_transaction()
            .expect("failed to create rw_transaction");
        let profile = rw.get().primary::<DbCategoryProfile>(*id)?;
        profile
            .map(|p| p.remove_all(&rw).and_then(|_| Ok(rw.commit()?)))
            .transpose()?;

        Ok(())
    }

    pub fn get_profile(&self, id: &ProfileId) -> Result<Option<CategoryProfile>> {
        let ro = self
            .db
            .r_transaction()
            .expect("failed to create ro_transaction");
        let profile = ro.get().primary::<DbCategoryProfile>(*id)?;

        profile.map(|p| p.reconstruct(&ro)).transpose()
    }

    pub fn set_profile(&self, profile: CategoryProfile) -> Result<()> {
        let rw = self
            .db
            .rw_transaction()
            .expect("failed to create rw_transaction");
        profile.save_all(&rw)?;
        Ok(rw.commit()?)
    }

    pub fn get_profiles(&self) -> Result<Vec<CategoryProfile>> {
        let ro = self
            .db
            .r_transaction()
            .expect("failed to create ro_transaction");
        let profiles = ro
            .scan()
            .primary::<DbCategoryProfile>()
            .expect("failed to scan category profiles")
            .all()
            .map(|p| p.reconstruct(&ro))
            .collect::<Result<_>>()?;
        Ok(profiles)
    }

    // In-memory configuration (currently readonly, but should ideally be configurable)
    pub fn get_template(&self, id: &TemplateId) -> Option<&Template> {
        self.templates.iter().find(|t| t.id == *id)
    }

    pub fn get_templates(&self) -> &[Template] {
        &self.templates
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use uuid::Uuid;

    use crate::{
        pipeline::{
            action_registar::PipelineActionRegistrar,
            data::{PipelineActionId, PipelineTarget, Selection},
        },
        util::create_dir_all,
    };

    use super::*;

    #[test]
    fn test_profile_crud() -> Result<()> {
        let registrar = PipelineActionRegistrar::builder().with_core().build();

        let path: PathBuf = "test/out/.config/deck-ds/profile_crud.db".into();
        let parent = path.parent().unwrap();
        create_dir_all(parent).unwrap();

        let db = ProfileDb::new(path.clone(), registrar.clone());

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

        db.set_profile(expected.clone())?;
        let actual = db.get_profile(&expected.id)?.expect("profile should exist");

        assert_eq!(expected.id, actual.id);
        assert_eq!(expected.pipeline.name, actual.pipeline.name);

        expected.pipeline.name = "Updated".to_string();

        db.set_profile(expected.clone())?;

        let actual = db
            .get_profile(&expected.id)?
            .expect("saved profile should exist");

        assert_eq!(expected.id, actual.id);
        assert_eq!(expected.pipeline.name, actual.pipeline.name);

        let actual = db
            .get_profiles()?
            .get(0)
            .cloned()
            .expect("get_profiles should find 1 profile");

        assert_eq!(expected.id, actual.id);
        assert_eq!(expected.pipeline.name, actual.pipeline.name);

        db.delete_profile(&expected.id)?;

        std::fs::remove_file(path)?;
        Ok(())
    }
}
