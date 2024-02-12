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

pub type PipelineDefinition = generic::PipelineDefinition<Action>;
pub type Pipeline = generic::Pipeline<Action>;
pub type PipelineActionDefinition = generic::PipelineActionDefinition<Action>;
pub type PipelineAction = generic::PipelineAction<Action>;
pub type PipelineActionSettings = generic::PipelineActionSettings<Action>;
pub type Selection<T> = generic::Selection<Action, T>;

pub type PipelineActionLookup = generic::PipelineActionLookup<Action>;

pub mod generic {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
    pub struct PipelineDefinition<A> {
        pub name: String,
        pub source_template: TemplateInfo,
        pub description: String,
        pub register_exit_hooks: bool,
        pub primary_target_override: Option<PipelineTarget>,
        pub targets: HashMap<PipelineTarget, Selection<A, PipelineActionId>>,
        pub actions: PipelineActionLookup<A>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct Pipeline<A> {
        pub name: String,
        pub description: String,
        pub register_exit_hooks: bool,
        pub primary_target_override: Option<PipelineTarget>,
        pub targets: HashMap<PipelineTarget, Selection<A, PipelineAction<A>>>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct PipelineActionDefinition<A> {
        pub name: String,
        pub description: Option<String>,
        pub id: PipelineActionId,
        pub settings: PipelineActionSettings<A>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct PipelineAction<A> {
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
        pub selection: Selection<A, PipelineAction<A>>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
    pub struct PipelineActionSettings<A> {
        /// Flags whether the selection is enabled. If None, not optional. If Some(true), optional and enabled, else disabled.
        pub enabled: Option<bool>,
        /// Whether or not the pipeline action is hidden on the QAM
        pub is_visible_on_qam: bool,
        /// Flags whether the selection is overridden by the setting from a different profile.
        pub profile_override: Option<ProfileId>,
        /// The value of the pipeline action
        pub selection: Selection<A, PipelineActionId>,
    }

    #[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
    pub struct PipelineActionLookup<A> {
        pub actions: HashMap<PipelineActionId, generic::PipelineActionSettings<A>>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
    #[serde(tag = "type", content = "value")]
    pub enum Selection<A, T> {
        Action(A),
        OneOf {
            selection: PipelineActionId,
            actions: Vec<T>,
        },
        AllOf(Vec<T>),
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
            .map(|(t, s)| s.reify(*t, self, profiles, registrar).map(|s| (*t, s)))
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

impl Selection<PipelineActionId> {
    fn reify(
        &self,
        target: PipelineTarget,
        pipeline: &PipelineDefinition,
        profiles: &[CategoryProfile],
        registrar: &PipelineActionRegistrar,
    ) -> Result<Selection<PipelineAction>> {
        match self {
            Selection::Action(action) => Ok(Selection::Action(action.clone())),
            Selection::OneOf { selection, actions } => {
                let actions = actions
                    .iter()
                    .map(|a| a.reify(target, pipeline, profiles, registrar))
                    .collect::<Result<Vec<_>>>();
                actions.map(|actions| Selection::OneOf {
                    selection: selection.clone(),
                    actions,
                })
            }
            Selection::AllOf(actions) => actions
                .iter()
                .map(|a| a.reify(target, pipeline, profiles, registrar))
                .collect::<Result<Vec<_>>>()
                .map(Selection::AllOf),
        }
    }
}

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

impl PipelineActionDefinition {
    fn reify(
        &self,
        profile_override: Option<ProfileId>,
        target: PipelineTarget,
        pipeline: &PipelineDefinition,
        profiles: &[CategoryProfile],
        registrar: &PipelineActionRegistrar,
    ) -> Result<PipelineAction> {
        let selection = self
            .settings
            .selection
            .reify(target, pipeline, profiles, registrar)?;

        Ok(PipelineAction {
            name: self.name.clone(),
            description: self.description.clone(),
            id: self.id.clone(),
            enabled: self.settings.enabled,
            profile_override,
            selection,
            is_visible_on_qam: self.settings.is_visible_on_qam,
        })
    }
}

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
