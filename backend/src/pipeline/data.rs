use derive_more::Display;
use std::collections::HashMap;
use strum::{EnumIter, IntoEnumIterator};

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

        PipelineActionId::new(&format!("{}:{variant}", self.no_variant().0))
    }

    pub fn no_variant(&self) -> PipelineActionId {
        PipelineActionId::new(
            &self
                .0
                .split_terminator(':')
                .take(3)
                .collect::<Vec<_>>()
                .join(":"),
        )
    }

    pub fn eq_variant(&self, id: &PipelineActionId, target: PipelineTarget) -> bool {
        self == id || *self == id.variant(target)
    }
}

#[derive(
    Copy, Debug, Display, Clone, PartialEq, Eq, Hash, EnumIter, Serialize, Deserialize, JsonSchema,
)]
pub enum PipelineTarget {
    Desktop,
    Gamemode,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Template {
    pub id: TemplateId,
    pub pipeline: PipelineDefinition,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PipelineDefinition {
    pub id: PipelineDefinitionId,
    pub name: String,
    pub register_exit_hooks: bool,
    pub primary_target_override: Option<PipelineTarget>,
    pub platform: PipelineActionId,
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
    UserDefined, // TODO::matching rules for which actions can be selected (or just get rid of this)
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
    ) -> Option<&PipelineActionSettings<ConfigSelection>> {
        let variant = id.variant(target);

        self.actions.get(&variant).or_else(|| self.actions.get(id))
    }
}

impl PipelineDefinition {
    pub fn reify<'a>(
        &'a self,
        profiles: &[CategoryProfile],
        registrar: &'a PipelineActionRegistrar,
    ) -> Result<Pipeline> {
        let targets = PipelineTarget::iter()
            .map(|t: PipelineTarget| {
                let mut toplevel: Vec<_> = registrar.toplevel().into_keys().collect();

                toplevel.sort_by(|a, b| a.0.cmp(&b.0));
                toplevel.insert(0, &self.platform);

                let reified: Vec<_> = toplevel
                    .into_iter()
                    .filter(|v| actions_have_target(v, &registrar.make_lookup(v), t, registrar))
                    .map(|v| v.reify(t, self, profiles, registrar))
                    .filter_map(|v| v.transpose())
                    .collect::<Result<_>>()?;

                Ok((t, RuntimeSelection::AllOf(reified)))
            })
            .collect::<Result<HashMap<_, _>>>()?
            .into_iter()
            .filter(|v| match &v.1 {
                RuntimeSelection::AllOf(v) => !v.is_empty(),
                _ => panic!("expected toplevel in reify to be AllOf"),
            })
            .collect();

        let description = registrar
            .get(&self.platform, PipelineTarget::Desktop)
            .and_then(|v| v.description.clone())
            .unwrap_or_default();

        Ok(Pipeline {
            name: self.name.clone(),
            description,
            register_exit_hooks: self.register_exit_hooks,
            primary_target_override: self.primary_target_override,
            targets,
        })
    }
}

impl PipelineActionId {
    fn reify(
        &self,
        target: PipelineTarget,
        pipeline: &PipelineDefinition,
        profiles: &[CategoryProfile],
        registrar: &PipelineActionRegistrar,
    ) -> Result<Option<PipelineAction>> {
        let config = pipeline.actions.get(self, target).cloned().or_else(|| {
            registrar.get(self, target).map(|v| PipelineActionSettings {
                enabled: v.settings.enabled,
                is_visible_on_qam: v.settings.is_visible_on_qam,
                profile_override: v.settings.profile_override,
                selection: match &v.settings.selection {
                    DefinitionSelection::Action(a) => ConfigSelection::Action(a.clone()),
                    DefinitionSelection::OneOf { selection, .. } => ConfigSelection::OneOf {
                        selection: selection.clone(),
                    },
                    DefinitionSelection::AllOf(_) => ConfigSelection::AllOf,
                    DefinitionSelection::UserDefined => ConfigSelection::UserDefined(vec![]),
                },
            })
        });

        match config {
            Some(config) => {
                let definition = registrar.get(self, target).with_context(|| {
                    format!("Failed to get registered action {:?} @ {}", self, target)
                })?;

                let settings = config
                    .profile_override
                    .and_then(|profile| {
                        profiles
                            .iter()
                            .find(|p| p.id == profile)
                            .and_then(|p| p.pipeline.actions.get(self, target))
                            .map(|config| (Some(profile), config))
                    })
                    .unwrap_or((None, &config));

                let resolved_action = settings.1.reify(
                    settings.0, definition, target, pipeline, profiles, registrar,
                )?;

                Ok(Some(resolved_action))
            }
            None => Ok(None),
        }
    }
}

impl PipelineActionSettings<ConfigSelection> {
    fn reify(
        &self,
        profile_override: Option<ProfileId>,
        definition: &PipelineActionDefinition,
        target: PipelineTarget,
        pipeline: &PipelineDefinition,
        profiles: &[CategoryProfile],
        registrar: &PipelineActionRegistrar,
    ) -> Result<PipelineAction> {
        let selection =
            self.selection
                .reify(&definition.id, target, pipeline, profiles, registrar)?;
        Ok(PipelineAction {
            name: definition.name.clone(),
            description: definition.description.clone(),
            id: definition.id.clone(),
            enabled: self.enabled,
            is_visible_on_qam: self.is_visible_on_qam,
            profile_override,
            selection,
        })
    }
}

impl ConfigSelection {
    fn reify(
        &self,
        id: &PipelineActionId,
        target: PipelineTarget,
        pipeline: &PipelineDefinition,
        profiles: &[CategoryProfile],
        registrar: &PipelineActionRegistrar,
    ) -> Result<RuntimeSelection> {
        let registered_selection = registrar
            .get(id, target)
            .map(|v| v.settings.selection.clone())
            .with_context(|| {
                format!("unable to find registered pipline action {id:?} when reifying config")
            })?;

        match self {
            ConfigSelection::Action(action) => Ok(RuntimeSelection::Action(action.clone())),
            ConfigSelection::OneOf { selection } => match registered_selection {
                DefinitionSelection::OneOf { actions, .. } => {
                    let actions = actions
                        .iter()
                        .map(|a| a.reify(target, pipeline, profiles, registrar))
                        .collect::<Result<Vec<_>>>();
                    actions.map(|actions| RuntimeSelection::OneOf {
                        selection: selection.clone(),
                        actions: actions.into_iter().flatten().collect(),
                    })
                }
                _ => Err(anyhow::anyhow!("selection type mismatch in reify config")),
            },
            ConfigSelection::AllOf => match registered_selection {
                DefinitionSelection::AllOf(actions) => actions
                    .iter()
                    .map(|a| a.reify(target, pipeline, profiles, registrar))
                    .collect::<Result<Vec<_>>>()
                    .map(|v| RuntimeSelection::AllOf(v.into_iter().flatten().collect())),
                _ => Err(anyhow::anyhow!("selection type mismatch in reify config")),
            },
            ConfigSelection::UserDefined(_actions) => todo!(), // actions
                                                               //     .iter()
                                                               //     .map(|a| a.reify(target, pipeline, profiles, registrar))
                                                               //     .collect::<Result<Vec<_>>>()
                                                               //     .map(RuntimeSelection::UserDefined(actions.clone())),
        }
    }
}

fn actions_have_target(
    root: &PipelineActionId,
    map: &PipelineActionLookup,
    target: PipelineTarget,
    registrar: &PipelineActionRegistrar,
) -> bool {
    fn search_settings(
        id: &PipelineActionId,
        map: &PipelineActionLookup,
        target: PipelineTarget,
        registrar: &PipelineActionRegistrar,
    ) -> bool {
        let settings = map.get(id, target);

        match settings {
            Some(PipelineActionSettings { selection, .. }) => match selection {
                ConfigSelection::Action(_) => true,
                ConfigSelection::AllOf | ConfigSelection::OneOf { .. } => {
                    match registrar.get(id, target) {
                        Some(values) => match &values.settings.selection {
                            DefinitionSelection::AllOf(values)
                            | DefinitionSelection::OneOf {
                                actions: values, ..
                            } => values
                                .iter()
                                .any(|v| search_settings(v, map, target, registrar)),
                            _ => panic!(),
                        },
                        None => false,
                    }
                }
                ConfigSelection::UserDefined(values) => values
                    .into_iter()
                    .any(|v| search_settings(v, map, target, registrar)),
            },
            None => false,
        }
    }

    search_settings(&root, map, target, registrar)
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use crate::{
        db::ProfileDb,
        pipeline::{action_registar::PipelineActionRegistrar, data::actions_have_target},
    };

    use super::*;

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
            .map(|t| (&t.pipeline, t.pipeline.clone().reify(&[], &registrar)))
            .collect();

        assert!(res.len() > 0);

        for (tp, p) in res {
            match p {
                Ok(p) => {
                    assert_eq!(tp.name, p.name);
                    let target_count = PipelineTarget::iter().fold(0, |a, v| {
                        if actions_have_target(&tp.platform, &tp.actions, v, &registrar) {
                            a + 1
                        } else {
                            a
                        }
                    });
                    assert!(target_count > 0);
                    assert_eq!(
                        target_count,
                        p.targets.len(),
                        "target mismatch for {}; expected {target_count}, got {:?}",
                        tp.name,
                        p.targets
                    );

                    let desktop = p.targets.get(&PipelineTarget::Desktop).unwrap();

                    match desktop {
                        crate::pipeline::data::RuntimeSelection::AllOf(v) => {
                            assert!(
                                v.iter()
                                    .any(|v| v.id.no_variant() == tp.platform.no_variant()),
                                "platform not found toplevel for {:?}, got {:?}",
                                tp.platform,
                                v.iter().map(|v| &v.id).collect::<Vec<_>>()
                            );

                            assert_eq!(
                                v.len(),
                                registrar.toplevel().into_keys().len() + 1,
                                "not all toplevel found for {:?}: {:?}",
                                tp.platform,
                                v.iter().map(|v| v.id.clone()).collect::<Vec<_>>(),
                            )
                            // may need revision of toplevel actions change
                        }
                        _ => panic!(),
                    }
                }
                Err(err) => panic!("{err}"),
            }
        }
    }
}
