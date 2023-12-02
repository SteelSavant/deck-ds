use derive_more::Display;
use either::Either::{self, Left, Right};
use std::collections::HashMap;

use crate::{
    macros::{newtype_strid, newtype_uuid},
    settings::{Profile, ProfileId},
};
use anyhow::{anyhow, Result};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use self::internal::{PipelineActionImpl, PipelineImpl};

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
    pub pipeline: DefinitionPipeline,
}

mod internal {
    use std::collections::HashMap;

    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    use super::{PipelineActionId, PipelineTarget, Selection};

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct PipelineImpl<T> {
        pub name: String,
        pub tags: Vec<String>,
        pub description: String,
        pub targets: HashMap<PipelineTarget, Selection<T>>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct PipelineActionImpl<T> {
        pub id: PipelineActionId,
        /// The name of the action
        pub name: String,
        /// An optional description of what the action does.
        pub description: Option<String>,
        /// The value of the pipeline action
        pub selection: Selection<T>,
    }
}

pub type DefinitionPipeline = PipelineImpl<PipelineActionId>;
pub type ActionPipeline = PipelineImpl<WrappedPipelineAction>;
pub type ActionOrProfilePipeline = PipelineImpl<PipelineActionOrProfile>;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
// #[serde(transparent)] causes schemars to stack overflow, so we dont' use it
pub struct WrappedPipelineAction(pub PipelineAction);

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct WrappedPipelineActionOrProfile {
    #[serde(
        serialize_with = "either::serde_untagged::serialize",
        deserialize_with = "either::serde_untagged::deserialize"
    )]
    item: Either<WrappedPipelineAction, ProfileAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProfileAction {
    profile: ProfileId,
    action: PipelineActionId,
}

pub type PipelineActionDefinition = PipelineActionImpl<PipelineActionId>;
pub type PipelineAction = PipelineActionImpl<WrappedPipelineAction>;
pub type PipelineActionOrProfile = PipelineActionImpl<WrappedPipelineActionOrProfile>;

impl PipelineActionDefinition {
    pub fn new(
        id: PipelineActionId,
        name: String,
        description: Option<String>,
        selection: Selection<PipelineActionId>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            selection,
        }
    }
}

impl PipelineAction {
    pub fn new(
        id: PipelineActionId,
        name: String,
        description: Option<String>,
        selection: Selection<WrappedPipelineAction>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            selection,
        }
    }
}

impl PipelineActionOrProfile {
    pub fn new(
        id: PipelineActionId,
        name: String,
        description: Option<String>,
        selection: Selection<WrappedPipelineActionOrProfile>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            selection,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "value")]
pub enum Selection<T> {
    Action(Action),
    OneOf {
        selection: PipelineActionId,
        actions: Vec<T>,
    },
    AllOf(Vec<Enabled<T>>),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Enabled<T> {
    /// Flags whether the selection is enabled. If None, not optional. If Some(true), optional and enabled, else disabled.
    pub enabled: Option<bool>,
    pub selection: T,
}

impl<T> Enabled<T> {
    pub fn force(selection: T) -> Self {
        Self {
            enabled: None,
            selection,
        }
    }

    pub fn default_true(selection: T) -> Self {
        Self {
            enabled: Some(true),
            selection,
        }
    }

    pub fn default_false(selection: T) -> Self {
        Self {
            enabled: Some(false),
            selection,
        }
    }
}

impl DefinitionPipeline {
    pub fn new(
        name: String,
        description: String,
        tags: Vec<String>,
        targets: HashMap<PipelineTarget, Selection<PipelineActionId>>,
    ) -> Self {
        Self {
            targets,
            name,
            tags,
            description,
        }
    }
}

// From/Into

impl From<PipelineAction> for WrappedPipelineAction {
    fn from(value: PipelineAction) -> Self {
        WrappedPipelineAction(value)
    }
}

impl From<PipelineAction> for WrappedPipelineActionOrProfile {
    fn from(value: PipelineAction) -> Self {
        WrappedPipelineActionOrProfile {
            item: Left(value.into()),
        }
    }
}

impl From<ProfileAction> for WrappedPipelineActionOrProfile {
    fn from(value: ProfileAction) -> Self {
        WrappedPipelineActionOrProfile { item: Right(value) }
    }
}

// Reification

pub trait ReifiablePipeline<E> {
    fn reify(self, external: &E) -> Result<ActionPipeline>;
}

pub trait ReifiableSelection<T, E> {
    fn reify(
        self,
        target: PipelineTarget,
        external: &E,
    ) -> Result<Selection<WrappedPipelineAction>>;
}

pub trait ReifiablePipelineAction<E> {
    fn reify(self, target: PipelineTarget, external: &E) -> Result<WrappedPipelineAction>;
}

impl<T, E> ReifiablePipeline<E> for PipelineImpl<T>
where
    T: ReifiablePipelineAction<E> + Clone,
{
    fn reify(self, external: &E) -> Result<ActionPipeline> {
        let targets = self
            .targets
            .iter()
            .map(|(t, s)| s.clone().reify(*t, external).map(|s| (*t, s)))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .collect::<HashMap<_, _>>();

        Ok(ActionPipeline {
            name: self.name,
            description: self.description,
            tags: self.tags,
            targets: targets,
        })
    }
}

impl<T, E> ReifiableSelection<T, E> for Selection<T>
where
    T: ReifiablePipelineAction<E> + Clone,
{
    fn reify(
        self,
        target: PipelineTarget,
        external: &E,
    ) -> Result<Selection<WrappedPipelineAction>> {
        match self {
            Selection::Action(a) => Ok(Selection::Action(a.clone())),
            Selection::OneOf { selection, actions } => Ok(Selection::OneOf {
                selection: selection.clone(),
                actions: actions
                    .iter()
                    .map(|a| a.clone().reify(target, external).map(|a| a.into()))
                    .collect::<Result<Vec<WrappedPipelineAction>>>()?,
            }),
            Selection::AllOf(actions) => actions
                .iter()
                .map(|a| {
                    a.selection
                        .clone()
                        .reify(target, external)
                        .map(|selection| Enabled {
                            enabled: a.enabled,
                            selection: selection.into(),
                        })
                })
                .collect::<Result<_>>()
                .map(|a| Selection::AllOf(a).into()),
        }
    }
}

impl<T, E> ReifiablePipelineAction<E> for PipelineActionImpl<T>
where
    T: ReifiablePipelineAction<E> + Clone,
{
    fn reify(self, target: PipelineTarget, external: &E) -> Result<WrappedPipelineAction> {
        let selection = self.selection.reify(target, external)?;

        Ok(PipelineAction::new(self.id, self.name, self.description, selection).into())
    }
}

impl ReifiablePipelineAction<PipelineActionRegistrar> for PipelineActionId {
    fn reify(
        self,
        target: PipelineTarget,
        external: &PipelineActionRegistrar,
    ) -> Result<WrappedPipelineAction> {
        external
            .get(&self, target)
            .ok_or(anyhow::anyhow!(
                "Could not find action: {self:?} for target {target}"
            ))
            .and_then(|action| action.clone().reify(target, external))
    }
}

impl ReifiablePipelineAction<&[Profile]> for WrappedPipelineActionOrProfile {
    fn reify(self, target: PipelineTarget, external: &&[Profile]) -> Result<WrappedPipelineAction> {
        match self.item {
            Either::Left(action) => Ok(action.clone().into()),
            Either::Right(action) => external
                .iter()
                .find(|p| p.id == action.profile)
                .ok_or(anyhow::anyhow!(
                    "Could not find profile: {:?}",
                    action.profile,
                ))
                .and_then(|p| {
                    p.pipeline
                        .targets
                        .get(&target)
                        .ok_or(anyhow!("Could not find target: {target:?}"))
                })
                .and_then(|a| {
                    a.find(&action.action, target)
                        .ok_or(anyhow!("Could not find action: {:?}", action.action))
                })
                .cloned(),
        }
    }
}

impl Selection<WrappedPipelineAction> {
    fn find(
        &self,
        action: &PipelineActionId,
        target: PipelineTarget,
    ) -> Option<&WrappedPipelineAction> {
        match self {
            Selection::Action(_) => None,
            Selection::OneOf { actions, .. } => actions
                .iter()
                .find(|a| a.0.id.eq_variant(action, target))
                .or_else(|| {
                    actions
                        .iter()
                        .filter_map(|a| a.0.selection.find(action, target))
                        .last()
                }),
            Selection::AllOf(actions) => actions
                .iter()
                .find_map(|a| {
                    if a.selection.0.id.eq_variant(action, target) {
                        Some(&a.selection)
                    } else {
                        None
                    }
                })
                .or_else(|| {
                    actions
                        .iter()
                        .filter_map(|a| a.selection.0.selection.find(action, target))
                        .last()
                }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use anyhow::Ok;

    use crate::{pipeline::action::virtual_screen::VirtualScreen, settings::Settings};

    use super::*;

    #[test]
    fn test_either_action_json_unifies_with_action() -> Result<()> {
        let expected: WrappedPipelineAction = PipelineAction {
            id: PipelineActionId::new("test:id:action"),
            name: "Test".to_string(),
            description: None,
            selection: VirtualScreen.into(),
        }
        .into();

        let either: WrappedPipelineActionOrProfile = WrappedPipelineActionOrProfile {
            item: Either::Left(expected.clone().into()),
        };

        let json = serde_json::to_string_pretty(&either)?;

        let actual: WrappedPipelineAction = serde_json::from_str(&json)?;

        assert_eq!(expected.0.id, actual.0.id);

        Ok(())
    }

    #[test]
    fn test_action_json_unifies_with_either_action() -> Result<()> {
        let expected: WrappedPipelineActionOrProfile = WrappedPipelineActionOrProfile {
            item: Either::Left(
                PipelineAction {
                    id: PipelineActionId::new("test:id:action"),
                    name: "Test".to_string(),
                    description: None,
                    selection: VirtualScreen.into(),
                }
                .into(),
            ),
        };

        let action = expected.clone().item.left().unwrap();

        let json = serde_json::to_string_pretty(&action)?;

        let actual: WrappedPipelineActionOrProfile = serde_json::from_str(&json)?;

        assert_eq!(
            expected.item.left().unwrap().0.id,
            actual.item.left().unwrap().0.id
        );

        Ok(())
    }

    #[test]
    fn test_template_reification() {
        let settings = Settings::new(
            Path::new("$HOME/homebrew/plugins/deck-ds/bin/backend"),
            Path::new("$HOME/.config/deck-ds"),
            Path::new("$HOME/.config/autostart"),
        );

        let registrar = PipelineActionRegistrar::builder().with_core().build();

        let res: Vec<_> = settings
            .get_templates()
            .into_iter()
            .map(|t| t.pipeline.clone().reify(&registrar))
            .collect();

        for p in res {
            if let Err(err) = p {
                panic!("{err}");
            }
        }
    }
}
