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
        /// Root action in the tree. Should be a platform action.
        platform: PipelineActionId,
        action_overrides: HashMap<PipelineActionId, Action>,
        enabled_overrides: HashMap<PipelineActionId, Option<bool>>,
        is_visible_on_qam_overrides: HashMap<PipelineActionId, bool>,
    }

    impl TemplateBuilder {
        fn build(self, registrar: &PipelineActionRegistrar) -> Template {
            let mut actions = registrar.make_lookup(&self.platform);
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
                .get(&self.platform, PipelineTarget::Desktop)
                .or_else(|| registrar.get(&self.platform, PipelineTarget::Gamemode))
                .unwrap();

            Template {
                id: self.id,
                pipeline: PipelineDefinition {
                    id: PipelineDefinitionId::nil(),
                    name: root_action.name.clone(),
                    platform: self.platform,
                    primary_target_override: None,
                    register_exit_hooks: true,
                    actions,
                },
            }
        }
    }

    let templates = vec![
        // melonDS
        TemplateBuilder {
            id: TemplateId::parse("c6430131-50e0-435e-a917-5ae3cfa46e3c"),
            platform: PipelineActionId::new("core:melonds:platform"),
            action_overrides: Default::default(),
            enabled_overrides: Default::default(),
            is_visible_on_qam_overrides: Default::default(),
        },
        // Citra
        TemplateBuilder {
            id: TemplateId::parse("fe82be74-22b9-4135-b7a0-cb6d8f51aecd"),
            platform: PipelineActionId::new("core:citra:platform"),
            action_overrides: Default::default(),
            enabled_overrides: Default::default(),
            is_visible_on_qam_overrides: Default::default(),
        },
        // Cemu
        TemplateBuilder {
            id: TemplateId::parse("33c863e5-2739-4bc3-b9bc-4798bac8682d"),
            platform: PipelineActionId::new("core:cemu:platform"),
            action_overrides: Default::default(),
            enabled_overrides: Default::default(),
            is_visible_on_qam_overrides: Default::default(),
        },
        // App
        TemplateBuilder {
            id: TemplateId::parse("84f870e9-9491-41a9-8837-d5a6f591f687"),
            platform: PipelineActionId::new("core:app:platform"),
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
