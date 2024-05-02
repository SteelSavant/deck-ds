use std::collections::HashMap;

use anyhow::{Context, Result};

use native_db::transaction::RTransaction;

use crate::{
    db::model::{
        DbAction, DbCemuLayout, DbCitraLayout, DbConfigSelection, DbDesktopSessionHandler,
        DbDisplayConfig, DbLaunchSecondaryApp, DbLaunchSecondaryAppPreset, DbLime3dsLayout,
        DbMainAppAutomaticWindowing, DbMelonDSLayout, DbMultiWindow, DbPipelineActionSettings,
        DbPipelineDefinition, DbSourceFile, DbTopLevelDefinition, DbVirtualScreen,
    },
    pipeline::{
        action::{Action, ActionType},
        data::{
            ConfigSelection, PipelineActionLookup, PipelineActionSettings, PipelineDefinition,
            PipelineDefinitionId, TopLevelDefinition,
        },
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
            ActionType::LaunchSecondaryFlatpakApp => {
                let action = ro.get().primary::<DbLaunchSecondaryApp>(id)?;
                action.map(|a| Action::LaunchSecondaryFlatpakApp(a.into()))
            }
            ActionType::LaunchSecondaryAppPreset => {
                let action = ro.get().primary::<DbLaunchSecondaryAppPreset>(id)?;
                action.map(|a| Action::LaunchSecondaryAppPreset(a.into()))
            }
            ActionType::MainAppAutomaticWindowing => {
                let action = ro.get().primary::<DbMainAppAutomaticWindowing>(id)?;
                action.map(|a| Action::MainAppAutomaticWindowing(a.into()))
            }
            ActionType::Lime3dsLayout => {
                let action = ro.get().primary::<DbLime3dsLayout>(id)?;
                action.map(|a| Action::Lime3dsLayout(a.into()))
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
        };

        Ok(selection)
    }
}

impl DbPipelineDefinition {
    pub fn transform(&self, ro: &RTransaction) -> Result<PipelineDefinition> {
        let platform = self.platform.transform(self.id, ro)?;
        let toplevel = self
            .toplevel
            .iter()
            .map(|v| v.transform(self.id, ro))
            .collect::<Result<_>>()?;

        Ok(PipelineDefinition {
            id: self.id,
            name: self.name.clone(),
            register_exit_hooks: self.register_exit_hooks,
            primary_target_override: self.primary_target_override,
            platform,
            toplevel,
        })
    }
}

impl DbTopLevelDefinition {
    fn transform(
        &self,
        pipeline_definition_id: PipelineDefinitionId,
        ro: &RTransaction,
    ) -> Result<TopLevelDefinition> {
        let actions = self
            .actions
            .iter()
            .filter_map(|v| {
                ro.get()
                    .primary::<DbPipelineActionSettings>((
                        pipeline_definition_id,
                        self.id,
                        v.clone(),
                    ))
                    .transpose()
                // .inspect(|s| {
                //     log::error!(
                //         "TMP::loading action id {:?}: {:?}",
                //         (pipeline_definition_id, self.id, v),
                //         s
                //     );
                // })
            })
            .map(|v| {
                v.map(|v| {
                    v.selection.transform(ro).map(|s| {
                        (
                            v.id.2,
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

        Ok(TopLevelDefinition {
            id: self.id,
            root: self.root.clone(),
            actions: PipelineActionLookup { actions: actions },
        })
    }
}
