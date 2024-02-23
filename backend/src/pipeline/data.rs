use derive_more::Display;
use std::collections::HashMap;

use crate::{
    macros::{newtype_strid, newtype_uuid},
    settings::{CategoryProfile, ProfileId},
};
use anyhow::{Context, Result};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{action::Action, action_registar::PipelineActionRegistrar};

newtype_strid!(
    r#"Id in the form "plugin:group:action" | "plugin:group:action:variant""#,
    PipelineActionId
);
newtype_uuid!(PipelineDefinitionId);
newtype_uuid!(TemplateId);

impl PipelineActionId {
    pub fn variant(&self, target: PipelineTarget) -> PipelineActionId {
        let variant = match target {
            PipelineTarget::Desktop => "desktop",
            PipelineTarget::Gamemode => "gamemode",
        };

        PipelineActionId::new(&format!("{}:{variant}", self.0))
    }

    pub fn eq_variant(&self, id: &PipelineActionId, target: PipelineTarget) -> bool {
        self == id || *self == id.variant(target)
    }
}

#[derive(Copy, Debug, Display, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema)]
pub enum PipelineTarget {
    Desktop,
    Gamemode,
}

#[cfg_attr(test, derive(Default))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TemplateInfo {
    pub id: TemplateId,
    /// The template's version; should be updated each time an action is moved, added, or removed
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Template {
    pub id: TemplateId,
    /// The template's version; should be updated each time an action is moved, added, or removed
    pub version: u32,
    pub pipeline: PipelineDefinition,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PipelineDefinition {
    pub id: PipelineDefinitionId,
    pub name: String,
    pub source_template: TemplateInfo,
    pub description: String,
    pub register_exit_hooks: bool,
    pub primary_target_override: Option<PipelineTarget>,
    pub targets: HashMap<PipelineTarget, Vec<PipelineActionId>>,
    pub actions: PipelineActionLookup,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Pipeline {
    pub name: String,
    pub description: String,
    pub register_exit_hooks: bool,
    pub primary_target_override: Option<PipelineTarget>,
    pub targets: HashMap<PipelineTarget, RuntimeSelection>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PipelineActionDefinition {
    pub id: PipelineActionId,
    pub name: String,
    pub description: Option<String>,
    pub settings: PipelineActionSettings<DefinitionSelection>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Deserialize, JsonSchema)]
pub struct PipelineAction {
    pub name: String,
    pub description: Option<String>,
    pub id: PipelineActionId,
    /// Flags whether the selection is enabled. If None, not optional. If Some(true), optional and enabled, else disabled.
    pub enabled: Option<bool>,
    /// Whether or not the pipeline action is hidden on the QAM
    pub is_visible_on_qam: bool,
    /// Flags whether the selection is overridden by the setting from a different profile.
    pub profile_override: Option<ProfileId>,
    /// The value of the pipeline action
    pub selection: RuntimeSelection,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PipelineActionSettings<Selection> {
    /// Flags whether the selection is enabled. If None, not optional. If Some(true), optional and enabled, else disabled.
    pub enabled: Option<bool>,
    /// Whether or not the pipeline action is hidden on the QAM
    pub is_visible_on_qam: bool,
    /// Flags whether the selection is overridden by the setting from a different profile.
    pub profile_override: Option<ProfileId>,
    /// The value of the pipeline action
    pub selection: Selection,
}

impl From<PipelineActionSettings<DefinitionSelection>> for PipelineActionSettings<ConfigSelection> {
    fn from(value: PipelineActionSettings<DefinitionSelection>) -> Self {
        Self {
            enabled: value.enabled,
            is_visible_on_qam: value.is_visible_on_qam,
            profile_override: value.profile_override,
            selection: value.selection.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct PipelineActionLookup {
    pub actions: HashMap<PipelineActionId, PipelineActionSettings<ConfigSelection>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum DefinitionSelection {
    Action(Action),
    OneOf {
        selection: PipelineActionId,
        actions: Vec<PipelineActionId>,
    },
    AllOf(Vec<PipelineActionId>),
    UserDefined, // TODO::matching rules for which actions can be selected
}

/// Configured selection for an specific pipeline. Only user values are saved;
/// everything else is pulled at runtime to ensure it's up to date.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "value")]
pub enum ConfigSelection {
    Action(Action),
    OneOf { selection: PipelineActionId },
    AllOf,
    UserDefined(Vec<PipelineActionId>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "value")]
pub enum RuntimeSelection {
    Action(Action),
    OneOf {
        selection: PipelineActionId,
        actions: Vec<PipelineAction>,
    },
    AllOf(Vec<PipelineAction>),
    UserDefined(Vec<PipelineAction>), // TODO::matching rules for which actions can be selected
}

impl From<DefinitionSelection> for ConfigSelection {
    fn from(value: DefinitionSelection) -> Self {
        match value {
            DefinitionSelection::Action(action) => ConfigSelection::Action(action),
            DefinitionSelection::OneOf { selection, .. } => ConfigSelection::OneOf { selection },
            DefinitionSelection::AllOf(_) => ConfigSelection::AllOf,
            DefinitionSelection::UserDefined => ConfigSelection::UserDefined(vec![]),
        }
    }
}

// Reification

impl PipelineActionLookup {
    pub fn get(
        &self,
        id: &PipelineActionId,
        target: PipelineTarget,
        registrar: &PipelineActionRegistrar,
    ) -> Option<PipelineActionDefinition> {
        let variant = id.variant(target);

        registrar.get(id, target).map(|def| {
            let settings = self
                .actions
                .get(&variant)
                .or_else(|| self.actions.get(id))
                .cloned();
            PipelineActionDefinition {
                settings: settings.unwrap_or_else(|| def.settings.clone()),
                ..def.clone()
            }
        })
    }
}

impl PipelineDefinition {
    pub fn reify(
        &self,
        profiles: &[CategoryProfile],
        registrar: &PipelineActionRegistrar,
    ) -> Result<Pipeline> {
        let targets = self
            .targets
            .iter()
            .map(|(t, s)| {
                s.iter()
                    .map(|v| v.reify(*t, self, profiles, registrar))
                    .collect::<Result<Vec<_>>>()
                    .map(|v| (*t, RuntimeSelection::AllOf(v)))
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .collect::<HashMap<_, _>>();

        Ok(Pipeline {
            name: self.name.clone(),
            description: self.description.clone(),
            register_exit_hooks: self.register_exit_hooks,
            primary_target_override: self.primary_target_override,
            targets,
        })
    }
}

// impl ConfigSelection {
//     fn reify(
//         &self,
//         id: &PipelineActionId,
//         target: PipelineTarget,
//         pipeline: &PipelineDefinition,
//         profiles: &[CategoryProfile],
//         registrar: &PipelineActionRegistrar,
//     ) -> Result<RuntimeSelection> {
//         let registered_selection = registrar
//             .get(id, target)
//             .map(|v| v.settings.selection)
//             .with_context(|| {
//                 format!("unable to find registered pipline action {id:?} when reifying config")
//             })?;

//         match self {
//             ConfigSelection::Action(action) => Ok(RuntimeSelection::Action(action.clone())),
//             ConfigSelection::OneOf { selection } => match registered_selection {
//                 DefinitionSelection::OneOf { actions, .. } => {
//                     let actions = actions
//                         .iter()
//                         .map(|a| a.reify(target, pipeline, profiles, registrar))
//                         .collect::<Result<Vec<_>>>();
//                     actions.map(|actions| RuntimeSelection::OneOf {
//                         selection: selection.clone(),
//                         actions,
//                     })
//                 }
//                 _ => Err(anyhow::anyhow!("selection type mismatch in reify config")),
//             },
//             ConfigSelection::AllOf => match registered_selection {
//                 DefinitionSelection::AllOf(actions) => actions
//                     .iter()
//                     .map(|a| a.reify(target, pipeline, profiles, registrar))
//                     .collect::<Result<Vec<_>>>()
//                     .map(RuntimeSelection::AllOf),
//                 _ => Err(anyhow::anyhow!("selection type mismatch in reify config")),
//             },
//             ConfigSelection::UserDefined(actions) => actions
//                 .iter()
//                 .map(|a| a.reify(target, pipeline, profiles, registrar))
//                 .collect::<Result<Vec<_>>>()
//                 .map(RuntimeSelection::UserDefined),
//         }
//     }
// }

impl PipelineActionId {
    fn reify(
        &self,
        target: PipelineTarget,
        pipeline: &PipelineDefinition,
        profiles: &[CategoryProfile],
        registrar: &PipelineActionRegistrar,
    ) -> Result<PipelineAction> {
        let action = pipeline
            .actions
            .get(self, target, registrar)
            .with_context(|| {
                format!(
                    "Failed to get action {:?} for current pipline @ {}",
                    self, target
                )
            })?;

        let resolved_action: PipelineAction = action
            .settings
            .profile_override
            .and_then(|profile| {
                profiles
                    .iter()
                    .find(|p| p.id == profile)
                    .and_then(|p| p.pipeline.actions.get(self, target, registrar))
                    .map(|action| {
                        action.reify(Some(profile), target, pipeline, profiles, registrar)
                    })
            })
            .unwrap_or_else(|| action.reify(None, target, pipeline, profiles, registrar))?;

        Ok(resolved_action)
    }
}

// impl PipelineActionDefinition {
//     fn reify(
//         &self,
//         profile_override: Option<ProfileId>,
//         target: PipelineTarget,
//         pipeline: &PipelineDefinition,
//         profiles: &[CategoryProfile],
//         registrar: &PipelineActionRegistrar,
//     ) -> Result<PipelineAction> {
//         let selection = self
//             .settings
//             .selection
//             .reify(target, pipeline, profiles, registrar)?;

//         Ok(PipelineAction {
//             name: self.name.clone(),
//             description: self.description.clone(),
//             id: self.id.clone(),
//             enabled: self.settings.enabled,
//             profile_override,
//             selection,
//             is_visible_on_qam: self.settings.is_visible_on_qam,
//         })
//     }
// }

#[cfg(test)]
mod tests {

    use crate::{db::ProfileDb, pipeline::action_registar::PipelineActionRegistrar};

    #[test]
    fn test_template_reification() {
        let registrar = PipelineActionRegistrar::builder().with_core().build();
        let profiles = ProfileDb::new(
            "test/out/.config/DeckDS/template_reification.db".into(),
            registrar,
        );

        let registrar = PipelineActionRegistrar::builder().with_core().build();

        let res: Vec<_> = profiles
            .get_templates()
            .into_iter()
            .map(|t| t.pipeline.clone().reify(&[], &registrar))
            .collect();

        for p in res {
            if let Err(err) = p {
                panic!("{err}");
            }
        }
    }
}
