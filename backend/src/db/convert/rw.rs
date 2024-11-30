use anyhow::{Context, Result};

use native_db::transaction::RwTransaction;

use crate::{
    db::{
        convert::ext::RwExt,
        model::{
            DbAction, DbAppOverride, DbCategoryProfile, DbCemuAudio, DbCemuLayout, DbCitraLayout,
            DbConfigSelection, DbDesktopControllerLayoutHack, DbDesktopSessionHandler,
            DbDisplayConfig, DbLaunchSecondaryApp, DbLaunchSecondaryAppPreset, DbLime3dsLayout,
            DbMainAppAutomaticWindowing, DbMelonDSLayout, DbMultiWindow, DbPipelineActionSettings,
            DbSourceFile, DbTopLevelDefinition, DbTouchConfig, DbVirtualScreen,
        },
    },
    pipeline::{
        action::{Action, ActionId, ActionType, ErasedPipelineAction},
        data::{
            ConfigSelection, PipelineActionId, PipelineActionLookup, PipelineDefinitionId,
            TopLevelDefinition, TopLevelId,
        },
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
                rw.upsert::<DbDesktopSessionHandler>(action.into())?;
            }
            Action::DisplayConfig(action) => {
                rw.upsert::<DbDisplayConfig>(action.into())?;
            }
            Action::VirtualScreen(action) => {
                rw.upsert::<DbVirtualScreen>(action.into())?;
            }
            Action::MultiWindow(action) => {
                rw.upsert::<DbMultiWindow>(action.into())?;
            }
            Action::CitraLayout(action) => {
                rw.upsert::<DbCitraLayout>(action.into())?;
            }
            Action::CemuLayout(action) => {
                rw.upsert::<DbCemuLayout>(action.into())?;
            }
            Action::CemuAudio(action) => {
                rw.upsert::<DbCemuAudio>(action.into())?;
            }
            Action::MelonDSLayout(action) => {
                rw.upsert::<DbMelonDSLayout>(action.into())?;
            }
            Action::SourceFile(action) => {
                rw.upsert::<DbSourceFile>(action.into())?;
            }
            Action::LaunchSecondaryFlatpakApp(action) => {
                rw.upsert::<DbLaunchSecondaryApp>(action.into())?;
            }
            Action::LaunchSecondaryAppPreset(action) => {
                rw.upsert::<DbLaunchSecondaryAppPreset>(action.into())?;
            }
            Action::MainAppAutomaticWindowing(action) => {
                rw.upsert::<DbMainAppAutomaticWindowing>(action.into())?;
            }
            Action::Lime3dsLayout(action) => {
                rw.upsert::<DbLime3dsLayout>(action.into())?;
            }
            Action::DesktopControllerLayoutHack(action) => {
                rw.upsert::<DbDesktopControllerLayoutHack>(action.into())?;
            }
            Action::TouchConfig(action) => {
                rw.upsert::<DbTouchConfig>(action.into())?;
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

impl TopLevelDefinition {
    pub fn save_all_and_transform(
        self,
        pipeline_id: PipelineDefinitionId,
        rw: &RwTransaction,
    ) -> Result<DbTopLevelDefinition> {
        let id = if self.id == TopLevelId::nil() {
            TopLevelId::new()
        } else {
            self.id
        };

        log::error!(
            "TMP::saving toplevel with (pipeline:{:?},toplevel:{:?}); changed: {}",
            pipeline_id,
            id,
            id != self.id,
        );

        Ok(DbTopLevelDefinition {
            id,
            root: self.root,
            actions: self.actions.save_all_and_transform(pipeline_id, id, rw)?,
        })
    }
}

impl PipelineActionLookup {
    /// Saves the [PipelineActionLookup]. Because it may set new ids internally, `save_all_and_transform` cosumes self.
    pub fn save_all_and_transform(
        self,
        pipeline_id: PipelineDefinitionId,
        toplevel_id: TopLevelId,
        rw: &RwTransaction,
    ) -> Result<Vec<PipelineActionId>> {
        self.actions
            .into_iter()
            .map(|(k, v)| {
                let settings = DbPipelineActionSettings {
                    id: (pipeline_id, toplevel_id, k.clone()),
                    enabled: v.enabled,
                    profile_override: v.profile_override,
                    selection: v.selection.save_all_and_transform(rw)?,
                    is_visible_on_qam: v.is_visible_on_qam,
                };

                let id = settings.id.clone();

                log::debug!("TMP::saving action with id:{:?}", id);

                rw.upsert(settings)?;

                log::debug!("TMP::saved action:{:?}", id);

                Ok(k)
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
            .all()?
            .filter_map(|app: Result<DbAppOverride, _>| app.ok()) // TODO::log/error on failure
            .filter(|app| app.id.1 == self.id)
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
                action.map(|a| rw.remove_blind(a))
            }
            ActionType::DisplayConfig => {
                let action = rw.get().primary::<DbDisplayConfig>(id)?;
                action.map(|a| rw.remove_blind(a))
            }
            ActionType::VirtualScreen => {
                let action = rw.get().primary::<DbVirtualScreen>(id)?;
                action.map(|a| rw.remove_blind(a))
            }
            ActionType::MultiWindow => {
                let action = rw.get().primary::<DbMultiWindow>(id)?;
                action.map(|a| rw.remove_blind(a))
            }
            ActionType::CitraLayout => {
                let action = rw.get().primary::<DbCitraLayout>(id)?;
                action.map(|a| rw.remove_blind(a))
            }
            ActionType::CemuLayout => {
                let action = rw.get().primary::<DbCemuLayout>(id)?;
                action.map(|a| rw.remove_blind(a))
            }
            ActionType::CemuAudio => {
                let action = rw.get().primary::<DbCemuAudio>(id)?;
                action.map(|a| rw.remove_blind(a))
            }
            ActionType::MelonDSLayout => {
                let action = rw.get().primary::<DbMelonDSLayout>(id)?;
                action.map(|a| rw.remove_blind(a))
            }
            ActionType::SourceFile => {
                let action = rw.get().primary::<DbSourceFile>(id)?;
                action.map(|a| rw.remove_blind(a))
            }
            ActionType::LaunchSecondaryFlatpakApp => {
                let action = rw.get().primary::<DbLaunchSecondaryApp>(id)?;
                action.map(|a| rw.remove_blind(a))
            }
            ActionType::LaunchSecondaryAppPreset => {
                let action = rw.get().primary::<DbLaunchSecondaryAppPreset>(id)?;
                action.map(|a| rw.remove_blind(a))
            }
            ActionType::MainAppAutomaticWindowing => {
                let action = rw.get().primary::<DbMainAppAutomaticWindowing>(id)?;
                action.map(|a| rw.remove_blind(a))
            }
            ActionType::Lime3dsLayout => {
                let action = rw.get().primary::<DbLime3dsLayout>(id)?;
                action.map(|a| rw.remove_blind(a))
            }
            ActionType::DesktopControllerLayoutHack => {
                let action = rw.get().primary::<DbDesktopControllerLayoutHack>(id)?;
                action.map(|a| rw.remove_blind(a))
            }
            ActionType::TouchConfig => {
                let action = rw.get().primary::<DbTouchConfig>(id)?;
                action.map(|a| rw.remove_blind(a))
            }
        }
        .transpose()
        .with_context(|| format!("failed to remove action {id:?}"))?;

        Ok(())
    }
}
