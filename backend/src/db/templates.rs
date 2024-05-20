use std::fmt::Debug;

newtype_strid!("", AppId);

use crate::{
    macros::newtype_strid,
    pipeline::{
        action_registar::PipelineActionRegistrar,
        data::{
            ExitHooks, PipelineActionId, PipelineActionLookup, PipelineDefinition,
            PipelineDefinitionId, PipelineTarget, Template, TemplateId, TopLevelDefinition,
            TopLevelId,
        },
    },
};

pub fn build_templates(registrar: PipelineActionRegistrar) -> Vec<Template> {
    struct TemplateBuilder {
        id: TemplateId,
        /// Root action in the tree. Should be a platform action.
        platform: PipelineActionId,
        tags: Vec<String>,
    }

    impl TemplateBuilder {
        fn build(self, registrar: &PipelineActionRegistrar) -> Template {
            let root_action = registrar
                .get(&self.platform, PipelineTarget::Desktop)
                .or_else(|| registrar.get(&self.platform, PipelineTarget::Gamemode))
                .unwrap();

            Template {
                id: self.id,
                tags: self.tags,
                pipeline: PipelineDefinition {
                    id: PipelineDefinitionId::nil(),
                    name: root_action.name.clone(),
                    platform: TopLevelDefinition {
                        id: TopLevelId::nil(),
                        root: self.platform,
                        actions: PipelineActionLookup::empty(),
                    },
                    primary_target_override: None,
                    exit_hooks: Some(ExitHooks::default()),
                    toplevel: vec![],
                },
            }
        }
    }

    let templates = vec![
        // App
        TemplateBuilder {
            id: TemplateId::parse("84f870e9-9491-41a9-8837-d5a6f591f687"),
            platform: PipelineActionId::new("core:app:platform"),
            tags: vec!["Steam", "App", "Native"]
                .into_iter()
                .map(|v| v.to_string())
                .collect(),
        },
        // melonDS
        TemplateBuilder {
            id: TemplateId::parse("c6430131-50e0-435e-a917-5ae3cfa46e3c"),
            platform: PipelineActionId::new("core:melonds:platform"),
            tags: vec!["melonDS", "nds", "Nintendo DS"]
                .into_iter()
                .map(|v| v.to_string())
                .collect(),
        },
        // Lime3DS
        TemplateBuilder {
            id: TemplateId::parse("fe82be74-22b9-4135-b7a0-cb6d8f51aecd"),
            platform: PipelineActionId::new("core:lime3ds:platform"),
            tags: vec!["Lime3DS", "3DS", "N3DS", "Nintendo 3DS"]
                .into_iter()
                .map(|v| v.to_string())
                .collect(),
        },
        // Citra
        TemplateBuilder {
            id: TemplateId::parse("fe82be74-22b9-4135-b7a0-cb6d8f51aecd"),
            platform: PipelineActionId::new("core:citra:platform"),
            tags: vec!["Citra", "3DS", "N3DS", "Nintendo 3DS"]
                .into_iter()
                .map(|v| v.to_string())
                .collect(),
        },
        // Cemu
        TemplateBuilder {
            id: TemplateId::parse("33c863e5-2739-4bc3-b9bc-4798bac8682d"),
            platform: PipelineActionId::new("core:cemu:platform"),
            tags: vec!["Cemu", "WiiU"]
                .into_iter()
                .map(|v| v.to_string())
                .collect(),
        },
        // Cemu (Proton)
        TemplateBuilder {
            id: TemplateId::parse("2bb19c62-a3ee-4602-9707-59258f9b21b9"),
            platform: PipelineActionId::new("core:cemu_proton:platform"),
            tags: vec![
                "Cemu (Proton)",
                "Cemu - Proton",
                "WiiU (Proton)",
                "WiiU - Proton",
            ]
            .into_iter()
            .map(|v| v.to_string())
            .collect(),
        },
    ];

    templates.into_iter().map(|t| t.build(&registrar)).collect()
}
