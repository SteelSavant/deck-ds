/// Data transforms between working data and the database format
use std::collections::HashMap;

use anyhow::Result;

use native_db::transaction::{RTransaction, RwTransaction};

use crate::{
    db::model::{DbAppOverride, DbCategoryProfile, DbPipelineActionSettings, DbPipelineDefinition},
    pipeline::data::{
        PipelineActionId, PipelineActionLookup, PipelineDefinition, PipelineDefinitionId,
        TopLevelDefinition, TopLevelId,
    },
    settings::{AppId, AppProfile, CategoryProfile, ProfileId},
};

use super::model::{DbAppSettings, DbTopLevelDefinition};
use ext::RwExt;

mod ext;
mod ro;
mod rw;

// Primary types

impl CategoryProfile {
    /// Saves the [CategoryProfile]. Because it may set new ids internally, `save_all_and_transform` cosumes self.
    pub fn save_all(self, rw: &RwTransaction) -> Result<()> {
        log::error!("TMP::saving profile with id {:?}", self.id);

        assert_ne!(self.id, ProfileId::nil());

        let db_profile = DbCategoryProfile {
            id: self.id,
            tags: self.tags.clone(),
            pipeline: self.pipeline.save_all_and_transform(rw)?,
        };

        rw.upsert(db_profile)?;

        Ok(())
    }
}

impl AppProfile {
    pub fn load(app_id: &AppId, ro: &RTransaction) -> Result<Self> {
        // TODO::figure out if/how native_db supports multiple primary keys, so this can be done more efficiently
        let mut overrides = HashMap::from_iter(
            ro.scan()
                .primary()?
                .all()?
                .filter_map(|app: Result<DbAppOverride, _>| app.ok()) // TODO::log/error on failure
                .filter(|app| app.id.0 == *app_id)
                .map(|app| Ok((app.id.1, app.pipeline.transform(ro)?)))
                .collect::<Result<Vec<_>>>()?,
        );

        for (profile_id, o) in overrides.iter_mut() {
            let profile = ro.get().primary::<DbCategoryProfile>(*profile_id)?;

            if let Some(profile) = profile {
                let profile = profile.reconstruct(ro)?;

                // override the visibility with the profile visibility, since the QAM can't actually set it;
                // same with name && platform.root && exit hooks

                // o.should_register_exit_hooks = profile.pipeline.should_register_exit_hooks;
                // o.exit_hooks_override = profile.pipeline.exit_hooks_override;
                o.name = profile.pipeline.name;
                o.platform.root = profile.pipeline.platform.root.clone();

                let platform = &mut o.platform;

                let mut tl_actions = vec![platform];
                tl_actions.append(&mut o.toplevel.iter_mut().collect::<Vec<_>>());

                let profile_platform = &profile.pipeline.platform;

                let mut profile_tl_actions = vec![profile_platform];
                profile_tl_actions
                    .append(&mut profile.pipeline.toplevel.iter().collect::<Vec<_>>());

                for tl in tl_actions.iter_mut() {
                    let profile_tl = profile_tl_actions.iter_mut().find(|v| v.id == tl.id);
                    if let Some(profile_tl) = profile_tl {
                        for (action_id, action) in tl.actions.actions.iter_mut() {
                            if let Some(profile_action) = profile_tl.actions.actions.get(action_id)
                            {
                                action.copy_qam_values(&profile_action);
                            }
                        }
                    }
                }
                let mut actual_toplevel = vec![];

                log::debug!("profile actions: {profile_tl_actions:?}");
                log::debug!("found actions: {tl_actions:?}");

                for ptl in profile_tl_actions.into_iter().skip(1) {
                    if let Some(action) = tl_actions
                        .iter()
                        .find(|v| v.id == ptl.id)
                        .map(|v| (**v).clone())
                    {
                        log::debug!("pushing found toplevel action {action:?}");
                        actual_toplevel.push(action);
                    } else {
                        let action = TopLevelDefinition {
                            actions: PipelineActionLookup::empty(),
                            ..ptl.clone()
                        };
                        log::debug!("pushing new toplevel action {action:?}");
                        actual_toplevel.push(action);
                    }
                }

                o.toplevel = actual_toplevel;
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

        log::error!(
            "TMP::saving pipeline definition with id {id:?}; changed: {}",
            id != self.id
        );

        let platform = self.platform.save_all_and_transform(id, rw)?;
        let existing_toplevel = rw
            .scan()
            .primary()?
            .all()?
            .filter_map(|app: Result<DbPipelineActionSettings, _>| app.ok()) // TODO::log/error on failure
            .filter_map(|v| {
                if v.id.0 == id && v.id.1 != platform.id {
                    Some(v.id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let toplevel = self
            .toplevel
            .into_iter()
            .map(|v| v.save_all_and_transform(id, rw))
            .collect::<Result<Vec<_>>>()?;

        // remove removed toplevel items from DB
        for etl in existing_toplevel.iter() {
            if !toplevel.iter().any(|v| v.id == etl.1) {
                let item = rw.get().primary::<DbPipelineActionSettings>(etl.clone())?;

                if let Some(item) = item {
                    rw.remove(item)?;
                }
            }
        }

        let db_pipeline = DbPipelineDefinition {
            id,
            name: self.name.clone(),
            // should_register_exit_hooks: self.should_register_exit_hooks,
            // exit_hooks_override: self.exit_hooks_override.map(DbExitHooks::from),
            primary_target_override: self.primary_target_override,
            platform,
            toplevel,
            desktop_controller_layout_hack: self.desktop_controller_layout_hack.into(),
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
            .chain(self.pipeline.toplevel);

        for tl in actions {
            for id in tl.actions {
                let action: Option<DbPipelineActionSettings> =
                    rw.get().primary((self.pipeline.id, tl.id, id))?;
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
        self.pipeline.toplevel = vec![];

        Ok(rw.remove_blind(self)?)
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
