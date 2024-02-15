use std::collections::HashMap;

/// Data transforms between working data and the database format
//
use native_db::transaction::{RTransaction, RwTransaction};

use crate::{
    db::model::{
        DbAction, DbAppOverride, DbAppSettings, DbCategoryProfile, DbCemuLayout, DbCitraLayout,
        DbDisplayConfig, DbMelonDSLayout, DbMultiWindow, DbPipelineActionLookup,
        DbPipelineActionSettings, DbPipelineDefinition, DbSelection, DbSourceFile, DbVirtualScreen,
    },
    pipeline::{
        action::{Action, ActionId, ErasedPipelineAction},
        data::{
            PipelineActionId, PipelineActionLookup, PipelineActionSettings, PipelineDefinition,
            Selection,
        },
    },
    settings::{AppId, AppProfile, CategoryProfile},
};
use anyhow::{Context, Result};

use super::model::DbDesktopSessionHandler;

impl CategoryProfile {
    pub fn save_all(&self, rw: &RwTransaction) -> Result<()> {
        impl Action {
            fn save_and_transform(&self, rw: &RwTransaction) -> Result<DbAction> {
                let cloned = {
                    let id = self.get_id();
                    // if id not set, create new id
                    if id == ActionId::nil() {
                        self.cloned_with_id(ActionId::new())
                    } else {
                        self.clone()
                    }
                };

                let id = cloned.get_id();

                let transformed = match cloned {
                    Action::DesktopSessionHandler(action) => {
                        rw.insert::<DbDesktopSessionHandler>(action.into())?;
                        DbAction::DesktopSessionHandler(id)
                    }
                    Action::DisplayConfig(action) => {
                        rw.insert::<DbDisplayConfig>(action.into())?;
                        DbAction::DisplayConfig(id)
                    }
                    Action::VirtualScreen(action) => {
                        rw.insert::<DbVirtualScreen>(action.into())?;
                        DbAction::VirtualScreen(id)
                    }
                    Action::MultiWindow(action) => {
                        rw.insert::<DbMultiWindow>(action.into())?;
                        DbAction::MultiWindow(id)
                    }
                    Action::CitraLayout(action) => {
                        rw.insert::<DbCitraLayout>(action.into())?;
                        DbAction::CitraLayout(id)
                    }
                    Action::CemuLayout(action) => {
                        rw.insert::<DbCemuLayout>(action.into())?;
                        DbAction::CemuLayout(id)
                    }
                    Action::MelonDSLayout(action) => {
                        rw.insert::<DbMelonDSLayout>(action.into())?;
                        DbAction::MelonDSLayout(id)
                    }
                    Action::SourceFile(action) => {
                        rw.insert::<DbSourceFile>(action.into())?;
                        DbAction::SourceFile(id)
                    }
                };

                Ok(transformed)
            }
        }

        impl Selection<PipelineActionId> {
            fn save_all_and_transform(
                &self,
                rw: &RwTransaction,
            ) -> Result<DbSelection<PipelineActionId>> {
                let selection = match self {
                    crate::pipeline::data::generic::Selection::Action(action) => {
                        DbSelection::Action(action.save_and_transform(rw)?)
                    }
                    crate::pipeline::data::generic::Selection::OneOf { selection, actions } => {
                        DbSelection::OneOf {
                            selection: selection.clone(),
                            actions: actions.clone(),
                        }
                    }
                    crate::pipeline::data::generic::Selection::AllOf(actions) => {
                        DbSelection::AllOf(actions.clone())
                    }
                };

                Ok(selection)
            }
        }

        impl PipelineActionLookup {
            fn save_all_and_transform(&self, rw: &RwTransaction) -> Result<DbPipelineActionLookup> {
                let actions = self
                    .actions
                    .iter()
                    .map(|(k, v)| {
                        let settings = DbPipelineActionSettings {
                            enabled: v.enabled,
                            profile_override: v.profile_override,
                            selection: v.selection.save_all_and_transform(rw)?,
                            is_visible_on_qam: v.is_visible_on_qam,
                        };

                        Ok((k.clone(), settings))
                    })
                    .collect::<Result<HashMap<_, _>>>()?;

                Ok(DbPipelineActionLookup { actions })
            }
        }

        impl PipelineDefinition {
            pub(super) fn save_all_and_transform(
                &self,
                rw: &RwTransaction,
            ) -> Result<DbPipelineDefinition> {
                let targets = self
                    .targets
                    .iter()
                    .map(|(k, v)| -> Result<_> {
                        let def = v.save_all_and_transform(rw)?;
                        Ok((*k, def))
                    })
                    .collect::<Result<HashMap<_, _>>>()?;

                let actions = self.actions.save_all_and_transform(rw)?;

                let db_pipeline = DbPipelineDefinition {
                    name: self.name.clone(),
                    description: self.description.clone(),
                    source_template: self.source_template.clone(),
                    register_exit_hooks: self.register_exit_hooks,
                    primary_target_override: self.primary_target_override,
                    targets,
                    actions,
                };

                Ok(db_pipeline)
            }
        }

        let db_profile = DbCategoryProfile {
            id: self.id,
            tags: self.tags.clone(),
            pipeline: self.pipeline.save_all_and_transform(rw)?,
        };

        rw.insert(db_profile)?;

        Ok(())
    }
}

impl DbCategoryProfile {
    pub fn reconstruct(self, ro: &RTransaction) -> Result<CategoryProfile> {
        impl DbAction {
            fn transform(&self, ro: &RTransaction) -> Result<Action> {
                let id = self.get_id();

                let transformed = match *self {
                    DbAction::DesktopSessionHandler(id) => {
                        let action = ro.get().primary::<DbDesktopSessionHandler>(id)?;
                        action.map(|a| Action::DesktopSessionHandler(a.into()))
                    }
                    DbAction::DisplayConfig(id) => {
                        let action = ro.get().primary::<DbDisplayConfig>(id)?;
                        action.map(|a| Action::DisplayConfig(a.into()))
                    }
                    DbAction::VirtualScreen(id) => {
                        let action = ro.get().primary::<DbVirtualScreen>(id)?;
                        action.map(|a| Action::VirtualScreen(a.into()))
                    }
                    DbAction::MultiWindow(id) => {
                        let action = ro.get().primary::<DbMultiWindow>(id)?;
                        action.map(|a| Action::MultiWindow(a.into()))
                    }
                    DbAction::CitraLayout(id) => {
                        let action = ro.get().primary::<DbCitraLayout>(id)?;
                        action.map(|a| Action::CitraLayout(a.into()))
                    }
                    DbAction::CemuLayout(id) => {
                        let action = ro.get().primary::<DbCemuLayout>(id)?;
                        action.map(|a| Action::CemuLayout(a.into()))
                    }
                    DbAction::MelonDSLayout(id) => {
                        let action = ro.get().primary::<DbMelonDSLayout>(id)?;
                        action.map(|a| Action::MelonDSLayout(a.into()))
                    }
                    DbAction::SourceFile(id) => {
                        let action = ro.get().primary::<DbSourceFile>(id)?;
                        action.map(|a| Action::SourceFile(a.into()))
                    }
                };

                transformed.with_context(|| format!("failed to recover action {id:?}"))
            }
        }

        impl DbSelection<PipelineActionId> {
            fn transform(&self, ro: &RTransaction) -> Result<Selection<PipelineActionId>> {
                let selection = match self {
                    DbSelection::Action(action) => Selection::Action(action.transform(ro)?),
                    DbSelection::OneOf { selection, actions } => Selection::OneOf {
                        selection: selection.clone(),
                        actions: actions.clone(),
                    },
                    DbSelection::AllOf(actions) => Selection::AllOf(actions.clone()),
                };

                Ok(selection)
            }
        }

        impl DbPipelineActionLookup {
            fn transform(&self, ro: &RTransaction) -> Result<PipelineActionLookup> {
                let actions = self
                    .actions
                    .iter()
                    .map(|(k, v)| {
                        let settings = PipelineActionSettings {
                            enabled: v.enabled,
                            profile_override: v.profile_override,
                            selection: v.selection.transform(ro)?,
                            is_visible_on_qam: v.is_visible_on_qam,
                        };

                        Ok((k.clone(), settings))
                    })
                    .collect::<Result<HashMap<_, _>>>()?;

                Ok(PipelineActionLookup { actions })
            }
        }

        impl DbPipelineDefinition {
            fn transform(&self, ro: &RTransaction) -> Result<PipelineDefinition> {
                let actions = self.actions.transform(ro)?;

                let targets = self
                    .targets
                    .iter()
                    .map(|(k, v)| -> Result<_> {
                        let def = v.transform(ro)?;
                        Ok((*k, def))
                    })
                    .collect::<Result<HashMap<_, _>>>()?;

                Ok(PipelineDefinition {
                    name: self.name.clone(),
                    description: self.description.clone(),
                    source_template: self.source_template.clone(),
                    register_exit_hooks: self.register_exit_hooks,
                    primary_target_override: self.primary_target_override,
                    targets,
                    actions,
                })
            }
        }

        let profile = CategoryProfile {
            id: self.id,
            tags: self.tags.clone(),
            pipeline: self.pipeline.transform(ro)?,
        };

        Ok(profile)
    }

    pub fn remove_all(self, rw: &RwTransaction) -> Result<()> {
        let pipeline = &self.pipeline;

        impl DbAction {
            fn remove(&self, rw: &RwTransaction) -> Result<()> {
                let id = self.get_id();

                let transformed = match *self {
                    DbAction::DesktopSessionHandler(id) => {
                        let action = rw.get().primary::<DbDesktopSessionHandler>(id)?;
                        action.map(|a| rw.remove(a))
                    }
                    DbAction::DisplayConfig(id) => {
                        let action = rw.get().primary::<DbDisplayConfig>(id)?;
                        action.map(|a| rw.remove(a))
                    }
                    DbAction::VirtualScreen(id) => {
                        let action = rw.get().primary::<DbVirtualScreen>(id)?;
                        action.map(|a| rw.remove(a))
                    }
                    DbAction::MultiWindow(id) => {
                        let action = rw.get().primary::<DbMultiWindow>(id)?;
                        action.map(|a| rw.remove(a))
                    }
                    DbAction::CitraLayout(id) => {
                        let action = rw.get().primary::<DbCitraLayout>(id)?;
                        action.map(|a| rw.remove(a))
                    }
                    DbAction::CemuLayout(id) => {
                        let action = rw.get().primary::<DbCemuLayout>(id)?;
                        action.map(|a| rw.remove(a))
                    }
                    DbAction::MelonDSLayout(id) => {
                        let action = rw.get().primary::<DbMelonDSLayout>(id)?;
                        action.map(|a| rw.remove(a))
                    }
                    DbAction::SourceFile(id) => {
                        let action = rw.get().primary::<DbSourceFile>(id)?;
                        action.map(|a| rw.remove(a))
                    }
                }
                .transpose()?;

                transformed.with_context(|| format!("failed to recover action {id:?}"))
            }
        }

        impl DbSelection<PipelineActionId> {
            fn remove(&self, rw: &RwTransaction) -> Result<()> {
                match self {
                    DbSelection::Action(action) => action.remove(rw)?,
                    DbSelection::OneOf { .. } => (),
                    DbSelection::AllOf(_) => (),
                };

                Ok(())
            }
        }

        pipeline
            .targets
            .values()
            .map(|v| -> Result<_> {
                v.remove(rw)?;
                Ok(())
            })
            .collect::<Result<Vec<_>>>()?;

        impl DbPipelineActionLookup {
            fn remove(&self, rw: &RwTransaction) -> Result<()> {
                self.actions
                    .values()
                    .map(|v| v.selection.remove(rw))
                    .collect::<Result<Vec<_>>>()?;

                Ok(())
            }
        }

        pipeline.actions.remove(rw)
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
                // override the visibility with the profile visibility, since the QAM can't actually set it;
                // same with name && exit hooks

                o.register_exit_hooks = profile.pipeline.register_exit_hooks;
                o.name = profile.pipeline.name;
                o.description = profile.pipeline.description;

                for (action_id, action) in o.actions.actions.iter_mut() {
                    action.is_visible_on_qam = profile
                        .pipeline
                        .actions
                        .actions
                        .get(action_id)
                        .unwrap_or_else(|| {
                            panic!("action {action_id:?} should exist on profile {profile_id:?}")
                        })
                        .is_visible_on_qam;
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
