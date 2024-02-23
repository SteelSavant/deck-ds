use anyhow::{Context, Result};

use native_db::transaction::RwTransaction;

use crate::{
    db::model::{
        DbAction, DbAppOverride, DbCategoryProfile, DbCemuLayout, DbCitraLayout,
        DbDesktopSessionHandler, DbDisplayConfig, DbMelonDSLayout, DbMultiWindow,
        DbPipelineActionSettings, DbSelection, DbSourceFile, DbVirtualScreen,
    },
    pipeline::{
        action::{Action, ActionId, ActionType, ErasedPipelineAction},
        data::{PipelineActionId, PipelineActionLookup, PipelineDefinitionId, Selection},
    },
};

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
        };

        Ok(DbAction {
            id,
            dtype: serde_json::to_string(&self.get_type())
                .expect("ActionType should be serializable"),
        })
    }
}

impl Selection<PipelineActionId> {
    fn save_all_and_transform(&self, rw: &RwTransaction) -> Result<DbSelection> {
        let selection = match self {
            Selection::Action(action) => DbSelection::Action(action.save_and_transform(rw)?),
            Selection::OneOf { selection, actions } => DbSelection::OneOf {
                selection: selection.clone(),
                actions: actions.clone(),
            },
            Selection::AllOf(actions) => DbSelection::AllOf(actions.clone()),
        };

        Ok(selection)
    }
}

impl PipelineActionLookup {
    pub fn save_all_and_transform(
        &self,
        pipeline_id: PipelineDefinitionId,
        rw: &RwTransaction,
    ) -> Result<Vec<PipelineActionId>> {
        self.actions
            .iter()
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

impl DbSelection {
    pub fn remove_all(&self, rw: &RwTransaction) -> Result<()> {
        match self {
            DbSelection::Action(action) => action.remove(rw)?,
            DbSelection::OneOf { .. } => (),
            DbSelection::AllOf(_) => (),
        };

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
        }
        .transpose()
        .with_context(|| format!("failed to remove action {id:?}"))?;

        Ok(())
    }
}