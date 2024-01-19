use std::collections::HashMap;

/// Data transforms between working data and the database format
//
use native_db::transaction::{RTransaction, RwTransaction};

use crate::{
    db::model::{
        DbAction, DbCategoryProfile, DbCemuLayout, DbCitraLayout, DbMelonDSLayout, DbMultiWindow,
        DbPipelineActionLookup, DbPipelineActionSettings, DbPipelineDefinition, DbSelection,
        DbSourceFile, DbVirtualScreen,
    },
    pipeline::{
        action::{Action, ActionId, ErasedPipelineAction},
        data::{
            PipelineActionId, PipelineActionLookup, PipelineActionSettings, PipelineDefinition,
            Selection,
        },
    },
    settings::CategoryProfile,
};
use anyhow::{Context, Result};

use super::model::DbUIManagement;

impl CategoryProfile {
    pub fn save_all(&self, rw: &RwTransaction) -> Result<()> {
        let pipeline = &self.pipeline;

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
                    Action::UIManagement(action) => {
                        rw.insert::<DbUIManagement>(action.into())?;
                        DbAction::UIManagement(id)
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

        let targets = pipeline
            .targets
            .iter()
            .map(|(k, v)| -> Result<_> {
                let def = v.save_all_and_transform(rw)?;
                Ok((*k, def))
            })
            .collect::<Result<HashMap<_, _>>>()?;

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
                        };

                        Ok((k.clone(), settings))
                    })
                    .collect::<Result<HashMap<_, _>>>()?;

                Ok(DbPipelineActionLookup { actions })
            }
        }

        let actions = pipeline.actions.save_all_and_transform(rw)?;

        let db_pipeline = DbPipelineDefinition {
            name: pipeline.name.clone(),
            description: pipeline.description.clone(),
            targets,
            actions,
        };

        let db_profile = DbCategoryProfile {
            id: self.id,
            tags: self.tags.clone(),
            pipeline: db_pipeline,
        };

        rw.insert(db_profile)?;

        Ok(())
    }
}

impl DbCategoryProfile {
    pub fn reconstruct(self, ro: &RTransaction) -> Result<CategoryProfile> {
        let pipeline = &self.pipeline;

        impl DbAction {
            fn transform(&self, ro: &RTransaction) -> Result<Action> {
                let id = self.get_id();

                let transformed = match *self {
                    DbAction::UIManagement(id) => {
                        let action = ro.get().primary::<DbUIManagement>(id)?;
                        action.map(|a| Action::UIManagement(a.into()))
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

        let targets = pipeline
            .targets
            .iter()
            .map(|(k, v)| -> Result<_> {
                let def = v.transform(ro)?;
                Ok((*k, def))
            })
            .collect::<Result<HashMap<_, _>>>()?;

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
                        };

                        Ok((k.clone(), settings))
                    })
                    .collect::<Result<HashMap<_, _>>>()?;

                Ok(PipelineActionLookup { actions })
            }
        }

        let actions = pipeline.actions.transform(ro)?;

        let pipeline = PipelineDefinition {
            name: pipeline.name.clone(),
            description: pipeline.description.clone(),
            targets,
            actions,
        };

        let profile = CategoryProfile {
            id: self.id,
            tags: self.tags.clone(),
            pipeline,
        };

        Ok(profile)
    }

    pub fn remove_all(self, rw: &RwTransaction) -> Result<()> {
        let pipeline = &self.pipeline;

        impl DbAction {
            fn remove(&self, rw: &RwTransaction) -> Result<()> {
                let id = self.get_id();

                let transformed = match *self {
                    DbAction::UIManagement(id) => {
                        let action = rw.get().primary::<DbUIManagement>(id)?;
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