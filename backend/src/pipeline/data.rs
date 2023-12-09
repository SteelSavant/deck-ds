use derive_more::Display;
use std::{borrow::Cow, collections::HashMap};

use crate::{
    macros::{newtype_strid, newtype_uuid},
    settings::{Profile, ProfileId},
};
use anyhow::{anyhow, Result};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{action::Action, registar::PipelineActionRegistrar};

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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PipelineDefinition {
    pub name: String,
    pub tags: Vec<String>,
    pub description: String,
    pub targets: HashMap<PipelineTarget, Selection<PipelineActionNode>>,
    pub actions: Cow<'static, PipelineActionRegistrar>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Pipeline {
    pub name: String,
    pub tags: Vec<String>,
    pub description: String,
    pub targets: HashMap<PipelineTarget, Selection<PipelineAction>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PipelineActionNode {
    pub id: PipelineActionId,
    /// Flags whether the selection is overridden by the setting from a different profile.
    pub profile_override: Option<ProfileId>,
}

impl PipelineActionNode {
    pub fn new(id: &str) -> Self {
        Self {
            id: PipelineActionId::new(id),
            profile_override: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PipelineActionDefinition {
    pub name: String,
    pub description: Option<String>,
    pub id: PipelineActionId,
    /// Flags whether the selection is enabled. If None, not optional. If Some(true), optional and enabled, else disabled.
    pub enabled: Option<bool>,
    /// The value of the pipeline action
    pub selection: Selection<PipelineActionNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PipelineAction {
    pub name: String,
    pub description: Option<String>,
    pub id: PipelineActionId,
    /// Flags whether the selection is enabled. If None, not optional. If Some(true), optional and enabled, else disabled.
    pub enabled: Option<bool>,
    /// Flags whether the selection is overridden by the setting from a different profile.
    pub profile_override: Option<ProfileId>,
    /// The value of the pipeline action
    pub selection: Selection<PipelineAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "value")]
pub enum Selection<T> {
    Action(Action),
    OneOf {
        selection: PipelineActionId,
        actions: Vec<T>,
    },
    AllOf(Vec<T>),
}

// Reification

impl PipelineDefinition {
    pub fn reify(&self, profiles: &[Profile]) -> Result<Pipeline> {
        let targets = self
            .targets
            .iter()
            .map(|(t, s)| s.reify(*t, &self, profiles).map(|s| (*t, s)))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .collect::<HashMap<_, _>>();

        Ok(Pipeline {
            name: self.name.clone(),
            description: self.description.clone(),
            tags: self.tags.clone(),
            targets,
        })
    }
}

impl Selection<PipelineActionNode> {
    fn reify(
        &self,
        target: PipelineTarget,
        pipeline: &PipelineDefinition,
        profiles: &[Profile],
    ) -> Result<Selection<PipelineAction>> {
        match self {
            Selection::Action(action) => Ok(Selection::Action(action.clone())),
            Selection::OneOf { selection, actions } => {
                let actions = actions
                    .iter()
                    .map(|a| a.reify(target, pipeline, profiles))
                    .collect::<Result<Vec<_>>>();
                actions.map(|actions| Selection::OneOf {
                    selection: selection.clone(),
                    actions,
                })
            }
            Selection::AllOf(actions) => actions
                .iter()
                .map(|a| a.reify(target, pipeline, profiles))
                .collect::<Result<Vec<_>>>()
                .map(Selection::AllOf),
        }
    }
}

impl PipelineActionNode {
    fn reify(
        &self,
        target: PipelineTarget,
        pipeline: &PipelineDefinition,
        profiles: &[Profile],
    ) -> Result<PipelineAction> {
        let target_pipeline = match self.profile_override {
            Some(id) => profiles
                .iter()
                .find(|p| p.id == id)
                .map(|p| &p.pipeline)
                .ok_or(anyhow!("Could not find profile {id:?}"))?,
            None => pipeline,
        };

        let action = target_pipeline
            .actions
            .get(&self.id, target)
            .ok_or(anyhow!(
                "Unable to find action {:?} for target {} of profile {:?}",
                self.id,
                target,
                self.profile_override
                    .map(|p| p.raw())
                    .unwrap_or_else(|| "local".to_string())
            ))?;

        action.reify(self.profile_override, target, pipeline, profiles)
    }
}

impl PipelineActionDefinition {
    fn reify(
        &self,
        profile_override: Option<ProfileId>,
        target: PipelineTarget,
        pipeline: &PipelineDefinition,
        profiles: &[Profile],
    ) -> Result<PipelineAction> {
        let selection = self.selection.reify(target, pipeline, profiles)?;

        Ok(PipelineAction {
            name: self.name.clone(),
            description: self.description.clone(),
            id: self.id.clone(),
            enabled: self.enabled,
            profile_override: profile_override,
            selection: selection,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::settings::Settings;

    #[test]
    fn test_template_reification() {
        let settings = Settings::new(
            Path::new("$HOME/homebrew/plugins/deck-ds/bin/backend"),
            Path::new("$HOME/.config/deck-ds"),
            Path::new("$HOME/.config/autostart"),
        );

        let res: Vec<_> = settings
            .get_templates()
            .into_iter()
            .map(|t| t.pipeline.clone().reify(&[]))
            .collect();

        for p in res {
            if let Err(err) = p {
                panic!("{err}");
            }
        }
    }
}
