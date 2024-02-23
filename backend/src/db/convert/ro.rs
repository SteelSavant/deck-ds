use std::collections::HashMap;

use anyhow::{Context, Result};

use native_db::transaction::RTransaction;

use crate::{
    db::model::{
        DbAction, DbCemuLayout, DbCitraLayout, DbConfigSelection, DbDesktopSessionHandler,
        DbDisplayConfig, DbMelonDSLayout, DbMultiWindow, DbPipelineActionSettings,
        DbPipelineDefinition, DbSourceFile, DbVirtualScreen,
    },
    pipeline::{
        action::{Action, ActionType},
        data::{ConfigSelection, PipelineActionLookup, PipelineActionSettings, PipelineDefinition},
    },
};

impl DbAction {
    fn transform(&self, ro: &RTransaction) -> Result<Action> {
        let id = self.id;
        let dtype: ActionType = serde_json::from_str(&self.dtype)?;

        let transformed = match dtype {
            ActionType::DesktopSessionHandler => {
                let action = ro.get().primary::<DbDesktopSessionHandler>(id)?;
                action.map(|a| Action::DesktopSessionHandler(a.into()))
            }
            ActionType::DisplayConfig => {
                let action = ro.get().primary::<DbDisplayConfig>(id)?;
                action.map(|a| Action::DisplayConfig(a.into()))
            }
            ActionType::VirtualScreen => {
                let action = ro.get().primary::<DbVirtualScreen>(id)?;
                action.map(|a| Action::VirtualScreen(a.into()))
            }
            ActionType::MultiWindow => {
                let action = ro.get().primary::<DbMultiWindow>(id)?;
                action.map(|a| Action::MultiWindow(a.into()))
            }
            ActionType::CitraLayout => {
                let action = ro.get().primary::<DbCitraLayout>(id)?;
                action.map(|a| Action::CitraLayout(a.into()))
            }
            ActionType::CemuLayout => {
                let action = ro.get().primary::<DbCemuLayout>(id)?;
                action.map(|a| Action::CemuLayout(a.into()))
            }
            ActionType::MelonDSLayout => {
                let action = ro.get().primary::<DbMelonDSLayout>(id)?;
                action.map(|a| Action::MelonDSLayout(a.into()))
            }
            ActionType::SourceFile => {
                let action = ro.get().primary::<DbSourceFile>(id)?;
                action.map(|a| Action::SourceFile(a.into()))
            }
        };

        transformed.with_context(|| format!("failed to recover action {id:?}"))
    }
}

impl DbConfigSelection {
    fn transform(&self, ro: &RTransaction) -> Result<ConfigSelection> {
        let selection = match self {
            DbConfigSelection::Action(action) => ConfigSelection::Action(action.transform(ro)?),
            DbConfigSelection::OneOf { selection } => ConfigSelection::OneOf {
                selection: selection.clone(),
            },
            DbConfigSelection::AllOf => ConfigSelection::AllOf,
            DbConfigSelection::UserDefined(actions) => {
                ConfigSelection::UserDefined(actions.clone())
            }
        };

        Ok(selection)
    }
}

impl DbPipelineDefinition {
    pub fn transform(&self, ro: &RTransaction) -> Result<PipelineDefinition> {
        let actions = self
            .actions
            .iter()
            .filter_map(|v| {
                ro.get()
                    .primary::<DbPipelineActionSettings>((self.id, v.clone()))
                    .transpose()
            })
            .map(|v| {
                v.map(|v| {
                    v.selection.transform(ro).map(|s| {
                        (
                            v.id.1,
                            PipelineActionSettings {
                                enabled: v.enabled,
                                is_visible_on_qam: v.is_visible_on_qam,
                                profile_override: v.profile_override,
                                selection: s,
                            },
                        )
                    })
                })
            })
            .map(|v| match v {
                Ok(Ok(ok)) => Ok(ok),
                Ok(Err(e)) => Err(e)?,
                Err(e) => Err(e)?,
            })
            .collect::<Result<HashMap<_, _>>>()?;

        Ok(PipelineDefinition {
            id: self.id,
            name: self.name.clone(),
            description: self.description.clone(),
            source_template: self.source_template.into(),
            register_exit_hooks: self.register_exit_hooks,
            primary_target_override: self.primary_target_override,
            targets: self.targets.clone(),
            actions: PipelineActionLookup { actions },
        })
    }
}
