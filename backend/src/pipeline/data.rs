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

use super::{
    action::{desktop_controller_layout_hack::DesktopControllerLayoutHack, Action},
    action_registar::PipelineActionRegistrar,
};

newtype_strid!(
    r#"Id in the form "plugin:group:action" | "plugin:group:action:variant""#,
    PipelineActionId
);
newtype_uuid!(PipelineDefinitionId);
newtype_uuid!(TopLevelId);
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

    pub fn get_target(&self) -> Option<PipelineTarget> {
        self.0
            .split_terminator(':')
            .nth(3)
            .and_then(|v| serde_json::from_str(v).ok())
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
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PipelineDefinition {
    pub id: PipelineDefinitionId,
    pub name: String,
    pub should_register_exit_hooks: bool,
    pub exit_hooks_override: Option<ExitHooks>,
    pub primary_target_override: Option<PipelineTarget>,
    pub platform: TopLevelDefinition,
    // Additional top-level actions besides the main platform.
    pub toplevel: Vec<TopLevelDefinition>,
    pub desktop_controller_layout_hack: DesktopControllerLayoutHack,
}

/// The required button chord to hold to exit. At least 2 buttons are required.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExitHooks(pub GamepadButton, pub GamepadButton, pub Vec<GamepadButton>);

impl ExitHooks {
    pub fn iter(&self) -> impl Iterator<Item = &GamepadButton> {
        [&self.0, &self.1].into_iter().chain(self.2.iter())
    }
}

impl Default for ExitHooks {
    fn default() -> Self {
        // Start + Select would be ideal defaults, but if desktop layouts break (again),
        // it becomes impossible to use because holding Start swaps the action set.
        Self(GamepadButton::L1, GamepadButton::East, vec![])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum GamepadButton {
    Start,
    Select,
    North,
    East,
    South,
    West,
    RightThumb,
    LeftThumb,
    DPadUp,
    DPadLeft,
    DPadRight,
    DPadDown,
    L1,
    L2,
    R1,
    R2,
}

impl std::fmt::Display for GamepadButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                GamepadButton::Start => "Start",
                GamepadButton::Select => "Select",
                GamepadButton::North => "Y (North)",
                GamepadButton::East => "B (East)",
                GamepadButton::South => "A (South)",
                GamepadButton::West => "X (West)",
                GamepadButton::RightThumb => "Right Thumb",
                GamepadButton::LeftThumb => "Left Thumb",
                GamepadButton::DPadUp => "DPad Up",
                GamepadButton::DPadLeft => "DPad Left",
                GamepadButton::DPadRight => "DPad Right",
                GamepadButton::DPadDown => "DPad Down",
                GamepadButton::L1 => "L1 (Left Bumper)",
                GamepadButton::L2 => "L2 (Left Trigger)",
                GamepadButton::R1 => "R1 (Right Bumper)",
                GamepadButton::R2 => "R2 (Right Trigger)",
            }
        )
    }
}

/// Defines a top-level action, with a root id and a unique set of actions.
/// This allows multiple top-level actions of the same type, without complicating
/// the structure too much.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TopLevelDefinition {
    pub id: TopLevelId,
    pub root: PipelineActionId,
    pub actions: PipelineActionLookup,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Pipeline {
    pub name: String,
    pub description: String,
    pub should_register_exit_hooks: bool,
    pub exit_hooks_override: Option<ExitHooks>,
    pub primary_target_override: Option<PipelineTarget>,
    pub targets: HashMap<PipelineTarget, RuntimeSelection>,
    pub desktop_controller_layout_hack: DesktopControllerLayoutHack,
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
    pub toplevel_id: TopLevelId,
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

impl PipelineActionLookup {
    pub fn empty() -> Self {
        Self {
            actions: Default::default(),
        }
    }
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
}

/// Configured selection for an specific pipeline. Only user values are saved;
/// everything else is pulled at runtime to ensure it's up to date.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "value")]
pub enum ConfigSelection {
    Action(Action),
    OneOf { selection: PipelineActionId },
    AllOf,
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
}

impl From<DefinitionSelection> for ConfigSelection {
    fn from(value: DefinitionSelection) -> Self {
        match value {
            DefinitionSelection::Action(action) => ConfigSelection::Action(action),
            DefinitionSelection::OneOf { selection, .. } => ConfigSelection::OneOf { selection },
            DefinitionSelection::AllOf(_) => ConfigSelection::AllOf,
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
                let platform_ref = &self.platform;
                let toplevel = [vec![platform_ref], self.toplevel.iter().collect()].concat();

                let reified: Vec<_> = toplevel
                    .iter()
                    .enumerate()
                    .filter(|(_, v)| actions_have_target(&v.root, t, registrar))
                    .map(|(i, v)| v.reify(i, t, profiles, registrar))
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
            .get(&self.platform.root, PipelineTarget::Desktop)
            .and_then(|v| v.description.clone())
            .unwrap_or_default();

        Ok(Pipeline {
            name: self.name.clone(),
            description,
            should_register_exit_hooks: self.should_register_exit_hooks,
            exit_hooks_override: self.exit_hooks_override.clone(),
            primary_target_override: self.primary_target_override,
            targets,
            desktop_controller_layout_hack: self.desktop_controller_layout_hack,
        })
    }
}

impl TopLevelDefinition {
    fn reify(
        &self,
        toplevel_index: usize,
        target: PipelineTarget,
        profiles: &[CategoryProfile],
        registrar: &PipelineActionRegistrar,
    ) -> Result<Option<PipelineAction>> {
        self.root.reify(&ReificationCtx {
            toplevel_index,
            toplevel_id: self.id,
            target,
            actions: &self.actions,
            profiles,
            registrar,
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct ReificationCtx<'a> {
    /// 0 for platform, otherwise (index - 1) into the toplevel array
    toplevel_index: usize,
    toplevel_id: TopLevelId,
    target: PipelineTarget,
    actions: &'a PipelineActionLookup,
    profiles: &'a [CategoryProfile],
    registrar: &'a PipelineActionRegistrar,
}

impl PipelineActionId {
    fn reify(&self, ctx: &ReificationCtx) -> Result<Option<PipelineAction>> {
        let config = ctx.actions.get(self, ctx.target).cloned().or_else(|| {
            log::warn!("missing action {self:?}; reifying from registry");
            ctx.registrar.get(self, ctx.target).and_then(|v| {
                Some(PipelineActionSettings {
                    enabled: v.settings.enabled,
                    is_visible_on_qam: v.settings.is_visible_on_qam,
                    profile_override: v.settings.profile_override,
                    selection: match &v.settings.selection {
                        DefinitionSelection::Action(a) => Some(ConfigSelection::Action(a.clone())),
                        DefinitionSelection::OneOf { selection, actions } => {
                            let action = ctx.registrar.get(selection, ctx.target).or_else(|| {
                                actions
                                    .iter()
                                    .map(|v| ctx.registrar.get(v, ctx.target))
                                    .next()
                                    .flatten()
                            });

                            action.map(|v| ConfigSelection::OneOf {
                                selection: v.id.clone(),
                            })
                        }
                        DefinitionSelection::AllOf(_) => Some(ConfigSelection::AllOf),
                    }?,
                })
            })
        });

        match config {
            Some(config) => {
                let definition = ctx.registrar.get(self, ctx.target).with_context(|| {
                    format!(
                        "Failed to get registered action {:?} @ {}",
                        self, ctx.target
                    )
                })?;

                let settings = config
                    .profile_override
                    .and_then(|profile| {
                        ctx.profiles
                            .iter()
                            .find(|p| p.id == profile)
                            .and_then(|p| resolve_action_from_profile_override(p, self, ctx))
                            .map(|config| (Some(profile), config.clone()))
                            .or_else(|| {
                                // if missing, use the registered defaults
                                let mut config: PipelineActionSettings<ConfigSelection> = ctx
                                    .registrar
                                    .get(self, ctx.target)
                                    .expect("action should exist if fetched for profile override")
                                    .settings
                                    .clone()
                                    .into();

                                config.profile_override = Some(profile);

                                Some((Some(profile), config))
                            })
                    })
                    .unwrap_or((None, config));

                log::debug!("reify pipeline action id {self:?} got config {settings:?}");

                let resolved_action = settings.1.reify(settings.0, definition, ctx)?;

                Ok(Some(resolved_action))
            }
            None => Ok(None),
        }
    }
}

fn resolve_action_from_profile_override<'a>(
    profile: &'a CategoryProfile,
    id: &PipelineActionId,
    ctx: &ReificationCtx,
) -> Option<&'a PipelineActionSettings<ConfigSelection>> {
    if ctx.toplevel_index == 0 {
        profile.pipeline.platform.actions.get(id, ctx.target)
    } else {
        profile.pipeline.toplevel[ctx.toplevel_index - 1]
            .actions
            .get(id, ctx.target)
    }
}

impl PipelineActionSettings<ConfigSelection> {
    fn reify(
        &self,
        profile_override: Option<ProfileId>,
        definition: &PipelineActionDefinition,
        ctx: &ReificationCtx,
    ) -> Result<PipelineAction> {
        let selection = self.selection.reify(&definition.id, ctx)?;
        Ok(PipelineAction {
            name: definition.name.clone(),
            description: definition.description.clone(),
            id: definition.id.clone(),
            toplevel_id: ctx.toplevel_id,
            enabled: self.enabled,
            is_visible_on_qam: self.is_visible_on_qam,
            profile_override,
            selection,
        })
    }
}

impl ConfigSelection {
    fn reify(&self, id: &PipelineActionId, ctx: &ReificationCtx) -> Result<RuntimeSelection> {
        let registered_selection = ctx
            .registrar
            .get(id, ctx.target)
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
                        .map(|a| a.reify(ctx))
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
                    .map(|a| a.reify(ctx))
                    .collect::<Result<Vec<_>>>()
                    .map(|v| RuntimeSelection::AllOf(v.into_iter().flatten().collect())),
                _ => Err(anyhow::anyhow!("selection type mismatch in reify config")),
            },
        }
    }
}

fn actions_have_target(
    root: &PipelineActionId,
    target: PipelineTarget,
    registrar: &PipelineActionRegistrar,
) -> bool {
    fn search_settings(
        id: &PipelineActionId,
        target: PipelineTarget,
        registrar: &PipelineActionRegistrar,
    ) -> bool {
        let settings = registrar.get(id, target);

        match settings.as_ref() {
            Some(PipelineActionDefinition { settings, .. }) => match &settings.selection {
                DefinitionSelection::Action(_) => true,
                DefinitionSelection::OneOf { actions, .. }
                | DefinitionSelection::AllOf(actions) => actions
                    .iter()
                    .map(|id| match registrar.get(id, target) {
                        Some(_) => search_settings(id, target, registrar),
                        None => false,
                    })
                    .any(|v| v),
            },
            None => false,
        }
    }

    search_settings(root, target, registrar)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

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
                        if actions_have_target(&tp.platform.root, v, &registrar) {
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

                    assert_pipeline_traversable(&p, &registrar);

                    let desktop = p.targets.get(&PipelineTarget::Desktop).unwrap();

                    match desktop {
                        crate::pipeline::data::RuntimeSelection::AllOf(v) => {
                            assert!(
                                v.iter()
                                    .any(|v| v.id.no_variant() == tp.platform.root.no_variant()),
                                "platform not found toplevel for {:?}, got {:?}",
                                tp.platform,
                                v.iter().map(|v| &v.id).collect::<Vec<_>>()
                            );
                        }
                        _ => panic!(),
                    }
                }
                Err(err) => panic!("{err}"),
            }
        }
    }

    // #[test]
    // fn test_toplevel_reification_for_override() -> Result<()> {
    //     use crate::settings::{AppId, AppProfile};

    //     let registrar = PipelineActionRegistrar::builder().with_core().build();
    //     let profiles = ProfileDb::new(
    //         "test/out/.config/DeckDS/toplevel_reification.db".into(),
    //         registrar,
    //     );

    //     let registrar = PipelineActionRegistrar::builder().with_core().build();

    //     let platform_root = PipelineActionId("core:app:platform".to_string());
    //     let toplevel_root = PipelineActionId("core:toplevel:secondary".to_string());

    //     let profile_pipeline = PipelineDefinition {
    //         id: PipelineDefinitionId::new(),
    //         name: "ToplevelProfile".to_string(),
    //         register_exit_hooks: false,
    //         primary_target_override: None,
    //         platform: TopLevelDefinition {
    //             id: TopLevelId::new(),
    //             actions: registrar.make_lookup(&platform_root),
    //             root: platform_root.clone(),
    //         },
    //         toplevel: vec![TopLevelDefinition {
    //             id: TopLevelId::new(),
    //             actions: registrar.make_lookup(&toplevel_root),
    //             root: toplevel_root.clone(),
    //         }],
    //     };

    //     let profile_id = ProfileId::new();

    //     let profile = CategoryProfile {
    //         id: profile_id,
    //         tags: vec![],
    //         pipeline: profile_pipeline,
    //     };

    //     profiles.set_profile(profile)?;

    //     let override_pipeline = PipelineDefinition {
    //         id: PipelineDefinitionId::new(),
    //         name: "ToplevelTest".to_string(),
    //         register_exit_hooks: false,
    //         primary_target_override: None,
    //         platform: TopLevelDefinition {
    //             id: TopLevelId::new(),
    //             actions: PipelineActionLookup::empty(),
    //             root: platform_root,
    //         },
    //         toplevel: vec![TopLevelDefinition {
    //             id: TopLevelId::new(),
    //             actions: PipelineActionLookup::empty(),
    //             root: toplevel_root,
    //         }],
    //     };

    //     let mut reified = override_pipeline.reify(&profiles.get_profiles()?, &registrar)?;

    //     let desktop_target = reified.targets.remove(&PipelineTarget::Desktop).unwrap();

    //     match desktop_target {

    //         RuntimeSelection::AllOf(actions) => actions.iter().skip(1).,
    //         _ => panic!("expected toplevel selection to be AllOf")
    //     }

    //     Ok(())
    // }

    fn assert_pipeline_traversable(p: &Pipeline, registrar: &PipelineActionRegistrar) {
        fn assert_selection_traversable(
            s: &RuntimeSelection,
            target: PipelineTarget,
            registrar: &PipelineActionRegistrar,
        ) {
            match s {
                RuntimeSelection::Action(_) => (),
                RuntimeSelection::OneOf { selection, actions } => {
                    assert!(
                        actions.iter().any(|v| v.id == *selection),
                        "could not find selection {selection:?} in available actions {actions:?}"
                    );
                    for a in actions {
                        assert_eq!(registrar.get(&a.id, target).unwrap().id, a.id);
                        assert_selection_traversable(&a.selection, target, registrar)
                    }
                }
                RuntimeSelection::AllOf(actions) => {
                    for a in actions {
                        assert_eq!(registrar.get(&a.id, target).unwrap().id, a.id);
                        assert_selection_traversable(&a.selection, target, registrar)
                    }
                }
            }
        }

        for target in PipelineTarget::iter() {
            let root = p.targets.get(&target);
            if let Some(root) = root {
                assert_selection_traversable(&root, target, registrar);
            }
        }
    }
}
