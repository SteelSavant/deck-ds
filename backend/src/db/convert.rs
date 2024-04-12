/// Data transforms between working data and the database format
use std::collections::HashMap;

use anyhow::Result;

use native_db::transaction::{RTransaction, RwTransaction};

use crate::{
    db::model::{DbAppOverride, DbCategoryProfile, DbPipelineActionSettings, DbPipelineDefinition},
    pipeline::data::{PipelineActionId, PipelineDefinition, PipelineDefinitionId, TopLevelId},
    settings::{AppId, AppProfile, CategoryProfile},
};

use super::model::{DbAppSettings, DbTopLevelDefinition};

mod ro;
mod rw;

// Primary types

impl CategoryProfile {
    /// Saves the [CategoryProfile]. Because it may set new ids internally, `save_all_and_transform` cosumes self.
    pub fn save_all(self, rw: &RwTransaction) -> Result<()> {
        let db_profile = DbCategoryProfile {
            id: self.id,
            tags: self.tags.clone(),
            pipeline: self.pipeline.save_all_and_transform(rw)?,
        };

        rw.insert(db_profile)?;

        Ok(())
    }
}

impl AppProfile {
    pub fn load(app_id: &AppId, ro: &RTransaction) -> Result<Self> {
        // TODO::figure out if/how native_db supports multiple primary keys, so this can be done more efficiently
        let mut overrides = HashMap::from_iter(
            ro.scan()
                .primary()?
                .all()
                .filter(|app: &DbAppOverride| app.id.0 == *app_id)
                .map(|app: DbAppOverride| Ok((app.id.1, app.pipeline.transform(ro)?)))
                .collect::<Result<Vec<_>>>()?,
        );

        for (profile_id, o) in overrides.iter_mut() {
            let profile = ro.get().primary::<DbCategoryProfile>(*profile_id)?;

            if let Some(profile) = profile {
                let profile = profile.reconstruct(ro)?;

                // override the visibility with the profile visibility, since the QAM can't actually set it;
                // same with name && platform && exit hooks

                o.register_exit_hooks = profile.pipeline.register_exit_hooks;
                o.name = profile.pipeline.name;
                o.platform = profile.pipeline.platform;

                let tl_actions = Some(&mut o.platform)
                    .iter()
                    .chain(o.toplevel.iter_mut().map(|v| &v));

                let profile_tl_actions = Some(profile.pipeline.platform)
                    .iter()
                    .chain(profile.pipeline.toplevel.iter())
                    .collect::<Vec<_>>();

                for tl in tl_actions {
                    let profile_tl = profile_tl_actions.iter().find(|v| v.id == tl.id);
                    if let Some(profile_tl) = profile_tl {
                        for (action_id, action) in tl.actions.actions.iter_mut() {
                            if let Some(profile_action) = profile_tl.actions.actions.get(action_id)
                            {
                                action.is_visible_on_qam = profile_action.is_visible_on_qam
                            }
                        }
                    }
                }
            }
        }

        let default_profile = ro
            .get()
            .primary(app_id.clone())?
            .and_then(|settings: DbAppSettings| settings.default_profile);

        Ok(Self {
            id: app_id.clone(),
            default_profile,
            overrides,
        })
    }
}

impl PipelineDefinition {
    /// Saves the [PipelineDefinition]. Because it may set new ids internally, `save_all_and_transform` cosumes self.
    pub fn save_all_and_transform(self, rw: &RwTransaction) -> Result<DbPipelineDefinition> {
        let id = if self.id == PipelineDefinitionId::nil() {
            PipelineDefinitionId::new()
        } else {
            self.id
        };

        let platform = self.platform.save_all_and_transform(id, rw)?;
        let toplevel = self
            .toplevel
            .into_iter()
            .enumerate()
            .map(|(i, v)| v.save_all_and_transform(id, rw))
            .collect::<Result<_>>()?;

        let db_pipeline = DbPipelineDefinition {
            id,
            name: self.name.clone(),
            register_exit_hooks: self.register_exit_hooks,
            primary_target_override: self.primary_target_override,
            platform,
            toplevel,
        };

        Ok(db_pipeline)
    }
}

// DB types

impl DbCategoryProfile {
    pub fn remove_all(mut self, rw: &RwTransaction) -> Result<()> {
        self.remove_app_overrides(rw)?;

        let actions = Some(self.pipeline.platform)
            .into_iter()
            .chain(self.pipeline.toplevel.into_iter())
            .enumerate();

        for (i, tl) in actions {
            for id in tl.actions {
                let action: Option<DbPipelineActionSettings> =
                    rw.get().primary((self.pipeline.id, id, i as u32))?;
                if let Some(action) = action {
                    action.selection.remove_all(rw)?;
                    rw.remove(action)?;
                }
            }
        }

        self.pipeline.platform = DbTopLevelDefinition {
            id: TopLevelId::nil(),
            root: PipelineActionId::new(""),
            actions: vec![],
        };

        Ok(rw.remove(self)?)
    }

    pub fn reconstruct(self, ro: &RTransaction) -> Result<CategoryProfile> {
        let profile = CategoryProfile {
            id: self.id,
            tags: self.tags.clone(),
            pipeline: self.pipeline.transform(ro)?,
        };

        Ok(profile)
    }
}
