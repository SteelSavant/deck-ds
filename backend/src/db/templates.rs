use std::{collections::HashMap, fmt::Debug};

newtype_uuid!(ProfileId);
newtype_strid!("", AppId);

use crate::{
    macros::{newtype_strid, newtype_uuid},
    pipeline::{
        action::{
            display_config::DisplayConfig, session_handler::ExternalDisplaySettings, Action,
            ActionId,
        },
        action_registar::PipelineActionRegistrar,
        data::{
            PipelineActionId, PipelineDefinition, PipelineDefinitionId, PipelineTarget, Template,
            TemplateId,
        },
    },
};

pub fn build_templates(registrar: PipelineActionRegistrar) -> Vec<Template> {
    struct TemplateBuilder {
        id: TemplateId,
        /// Root action in the tree. Selection be an AllOf.
        root: PipelineActionId,
        action_overrides: HashMap<PipelineActionId, Action>,
        enabled_overrides: HashMap<PipelineActionId, Option<bool>>,
        is_visible_on_qam_overrides: HashMap<PipelineActionId, bool>,
    }

    impl TemplateBuilder {
        fn build(self, registrar: &PipelineActionRegistrar) -> Template {
            let mut actions = registrar.make_lookup(&self.root);
            for (id, action) in self.action_overrides {
                actions
                    .actions
                    .entry(id)
                    .and_modify(|v| v.selection = action.into());
            }

            for (id, enabled) in self.enabled_overrides {
                actions
                    .actions
                    .entry(id)
                    .and_modify(|v| v.enabled = enabled);
            }

            for (id, enabled) in self.is_visible_on_qam_overrides {
                actions
                    .actions
                    .entry(id)
                    .and_modify(|v| v.is_visible_on_qam = enabled);
            }

            let root_action = registrar
                .get(&self.root, PipelineTarget::Desktop)
                .or_else(|| registrar.get(&self.root, PipelineTarget::Gamemode))
                .unwrap();

            Template {
                id: self.id,
                pipeline: PipelineDefinition {
                    id: PipelineDefinitionId::nil(),
                    name: root_action.name.clone(),
                    description: root_action.description.clone().unwrap_or_default(),
                    root: self.root,
                    primary_target_override: None,
                    register_exit_hooks: true,
                    actions,
                    secondary_actions: vec![],
                },
            }
        }
    }

    let templates = vec![
        // melonDS
        TemplateBuilder {
            id: TemplateId::parse("c6430131-50e0-435e-a917-5ae3cfa46e3c"),
            root: PipelineActionId::new("core:melonds:root"),
            action_overrides: Default::default(),
            enabled_overrides: Default::default(),
            is_visible_on_qam_overrides: Default::default(),
        },
        // Citra
        TemplateBuilder {
            id: TemplateId::parse("fe82be74-22b9-4135-b7a0-cb6d8f51aecd"),
            root: PipelineActionId::new("core:citra:root"),
            action_overrides: Default::default(),
            enabled_overrides: Default::default(),
            is_visible_on_qam_overrides: Default::default(),
        },
        // Cemu
        TemplateBuilder {
            id: TemplateId::parse("33c863e5-2739-4bc3-b9bc-4798bac8682d"),
            root: PipelineActionId::new("core:cemu:root"),
            action_overrides: Default::default(),
            enabled_overrides: Default::default(),
            is_visible_on_qam_overrides: Default::default(),
        },
        // App
        TemplateBuilder {
            id: TemplateId::parse("84f870e9-9491-41a9-8837-d5a6f591f687"),
            root: PipelineActionId::new("core:app:root"),
            action_overrides: HashMap::from_iter([(
                PipelineActionId::new("core:display:display_config:desktop"),
                Action::DisplayConfig(DisplayConfig {
                    id: ActionId::nil(),
                    external_display_settings: ExternalDisplaySettings::Previous,
                    deck_location: None,
                    deck_is_primary_display: false,
                }),
            )]),
            enabled_overrides: Default::default(),
            is_visible_on_qam_overrides: HashMap::from_iter([(
                PipelineActionId::new("core:display:display_config:desktop"),
                true,
            )]),
        },
    ];

    templates.into_iter().map(|t| t.build(&registrar)).collect()
}
