use derive_more::Display;
use std::collections::HashMap;

use crate::{
    macros::{newtype_strid, newtype_uuid},
    settings::{CategoryProfile, ProfileId},
};
use anyhow::{Context, Result};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::action_registar::{PipelineActionLookup, PipelineActionRegistrar};

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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Template {
    pub id: TemplateId,
    pub pipeline: PipelineDefinition,
}

pub type PipelineDefinition = v1::PipelineDefinition;
pub type Pipeline = v1::Pipeline;
pub type PipelineActionDefinition = v1::PipelineActionDefinition;
pub type PipelineAction = v1::PipelineAction;
pub type PipelineActionSettings = v1::PipelineActionSettings;
pub type Selection<T> = v1::Selection<T>;

pub mod v1 {
    use crate::pipeline::action::v1;

    use super::versioned;

    pub type PipelineDefinition = versioned::PipelineDefinition<v1::Action>;
    pub type Pipeline = versioned::Pipeline<v1::Action>;
    pub type PipelineActionDefinition = versioned::PipelineActionDefinition<v1::Action>;
    pub type PipelineAction = versioned::PipelineAction<v1::Action>;
    pub type PipelineActionSettings = versioned::PipelineActionSettings<v1::Action>;
    pub type Selection<T> = versioned::Selection<v1::Action, T>;
}

mod versioned {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct PipelineDefinition<A> {
        pub name: String,
        pub description: String,
        pub targets: HashMap<PipelineTarget, Selection<A, PipelineActionId>>,
        pub actions: PipelineActionLookup,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct Pipeline<A> {
        pub name: String,
        pub description: String,
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
        /// Flags whether the selection is overridden by the setting from a different profile.
        pub profile_override: Option<ProfileId>,
        /// The value of the pipeline action
        pub selection: Selection<A, PipelineAction<A>>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct PipelineActionSettings<A> {
        /// Flags whether the selection is enabled. If None, not optional. If Some(true), optional and enabled, else disabled.
        pub enabled: Option<bool>,
        /// Flags whether the selection is overridden by the setting from a different profile.
        pub profile_override: Option<ProfileId>,
        /// The value of the pipeline action
        pub selection: Selection<A, PipelineActionId>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{pipeline::action_registar::PipelineActionRegistrar, settings::Settings};

    #[test]
    fn test_template_reification() {
        let settings = Settings::new(
            Path::new("$HOME/homebrew/plugins/deck-ds/bin/backend"),
            Path::new("test/out/.config/deck-ds2"),
            Path::new("$HOME/.config/autostart"),
            PipelineActionRegistrar::builder().with_core().build(),
        );

        let registrar = PipelineActionRegistrar::builder().with_core().build();

        let res: Vec<_> = settings
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
