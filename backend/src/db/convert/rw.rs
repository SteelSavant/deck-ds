use anyhow::{Context, Result};

use native_db::transaction::RwTransaction;

use crate::{
    db::model::{
        DbAction, DbAppOverride, DbCategoryProfile, DbCemuLayout, DbCitraLayout, DbConfigSelection,
        DbDesktopSessionHandler, DbDisplayConfig, DbLaunchSecondaryApp, DbLaunchSecondaryAppPreset,
        DbMelonDSLayout, DbMultiWindow, DbPipelineActionSettings, DbSourceFile, DbVirtualScreen,
    },
    pipeline::{
        action::{Action, ActionId, ActionType, ErasedPipelineAction},
        data::{ConfigSelection, PipelineActionId, PipelineActionLookup, PipelineDefinitionId},
    },
};

// Primary types

impl Action {
    /// Saves the [Action]. Because it may set new ids internally, `save_all_and_transform` cosumes self.
    fn save_and_transform(self, rw: &RwTransaction) -> Result<DbAction> {
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

        match cloned {
            Action::DesktopSessionHandler(action) => {
                rw.insert::<DbDesktopSessionHandler>(action.into())?;
            }
            Action::DisplayConfig(action) => {
                rw.insert::<DbDisplayConfig>(action.into())?;
            }
            Action::VirtualScreen(action) => {
                rw.insert::<DbVirtualScreen>(action.into())?;
            }
            Action::MultiWindow(action) => {
                rw.insert::<DbMultiWindow>(action.into())?;
            }
            Action::CitraLayout(action) => {
                rw.insert::<DbCitraLayout>(action.into())?;
            }
            Action::CemuLayout(action) => {
                rw.insert::<DbCemuLayout>(action.into())?;
            }
            Action::MelonDSLayout(action) => {
                rw.insert::<DbMelonDSLayout>(action.into())?;
            }
            Action::SourceFile(action) => {
                rw.insert::<DbSourceFile>(action.into())?;
            }
            Action::LaunchSecondaryFlatpakApp(action) => {
                rw.insert::<DbLaunchSecondaryApp>(action.into())?;
            }
            Action::LaunchSecondaryAppPreset(action) => {
                rw.insert::<DbLaunchSecondaryAppPreset>(action.into())?;
            }
        };

        Ok(DbAction {
            id,
            dtype: serde_json::to_string(&self.get_type())
                .expect("ActionType should be serializable"),
        })
    }
}

impl ConfigSelection {
    /// Saves the [Selection]. Because it may set new ids internally, `save_all_and_transform` cosumes self.
    fn save_all_and_transform(self, rw: &RwTransaction) -> Result<DbConfigSelection> {
        let selection = match self {
            ConfigSelection::Action(action) => {
                DbConfigSelection::Action(action.save_and_transform(rw)?)
            }
            ConfigSelection::OneOf { selection } => DbConfigSelection::OneOf {
                selection: selection.clone(),
            },
            ConfigSelection::AllOf => DbConfigSelection::AllOf,
        };

        Ok(selection)
    }
}

impl PipelineActionLookup {
    /// Saves the [PipelineActionLookup]. Because it may set new ids internally, `save_all_and_transform` cosumes self.
    pub fn save_all_and_transform(
        self,
        pipeline_id: PipelineDefinitionId,
        rw: &RwTransaction,
    ) -> Result<Vec<PipelineActionId>> {
        self.actions
            .into_iter()
            .map(|(k, v)| {
                let settings = DbPipelineActionSettings {
                    id: (pipeline_id, k.clone()),
                    enabled: v.enabled,
                    profile_override: v.profile_override,
                    selection: v.selection.save_all_and_transform(rw)?,
                    is_visible_on_qam: v.is_visible_on_qam,
                };

                rw.insert(settings)?;

                Ok(k.clone())
            })
            .collect::<Result<Vec<_>>>()
    }
}

// DB types

impl DbCategoryProfile {
    pub fn remove_app_overrides(&self, rw: &RwTransaction) -> Result<()> {
        let overrides = rw
            .scan()
            .primary()?
            .all()
            .filter(|app: &DbAppOverride| app.id.1 == self.id)
            .map(|app: DbAppOverride| app)
            .collect::<Vec<_>>();

        for o in overrides {
            rw.remove(o)?;
        }

        Ok(())
    }
}

impl DbConfigSelection {
    pub fn remove_all(&self, rw: &RwTransaction) -> Result<()> {
        if let DbConfigSelection::Action(action) = self {
            action.remove(rw)?
        }

        Ok(())
    }
}

impl DbAction {
    fn remove(&self, rw: &RwTransaction) -> Result<()> {
        let id = self.id;
        let dtype: ActionType = serde_json::from_str(&self.dtype)?;

        match dtype {
            ActionType::DesktopSessionHandler => {
                let action = rw.get().primary::<DbDesktopSessionHandler>(id)?;
                action.map(|a| rw.remove(a))
            }
            ActionType::DisplayConfig => {
                let action = rw.get().primary::<DbDisplayConfig>(id)?;
                action.map(|a| rw.remove(a))
            }
            ActionType::VirtualScreen => {
                let action = rw.get().primary::<DbVirtualScreen>(id)?;
                action.map(|a| rw.remove(a))
            }
            ActionType::MultiWindow => {
                let action = rw.get().primary::<DbMultiWindow>(id)?;
                action.map(|a| rw.remove(a))
            }
            ActionType::CitraLayout => {
                let action = rw.get().primary::<DbCitraLayout>(id)?;
                action.map(|a| rw.remove(a))
            }
            ActionType::CemuLayout => {
                let action = rw.get().primary::<DbCemuLayout>(id)?;
                action.map(|a| rw.remove(a))
            }
            ActionType::MelonDSLayout => {
                let action = rw.get().primary::<DbMelonDSLayout>(id)?;
                action.map(|a| rw.remove(a))
            }
            ActionType::SourceFile => {
                let action = rw.get().primary::<DbSourceFile>(id)?;
                action.map(|a| rw.remove(a))
            }
            ActionType::LaunchSecondaryFlatpakApp => {
                let action = rw.get().primary::<DbLaunchSecondaryApp>(id)?;
                action.map(|a| rw.remove(a))
            }
            ActionType::LaunchSecondaryAppPreset => {
                let action = rw.get().primary::<DbLaunchSecondaryAppPreset>(id)?;
                action.map(|a| rw.remove(a))
            }
        }
        .transpose()
        .with_context(|| format!("failed to remove action {id:?}"))?;

        Ok(())
    }
}
