use std::{collections::HashMap, fmt::Debug};

newtype_uuid!(ProfileId);
newtype_strid!("", AppId);

use crate::{
    macros::{newtype_strid, newtype_uuid},
    pipeline::{
        action_registar::PipelineActionRegistrar,
        data::{
            PipelineActionId, PipelineDefinition, PipelineTarget, Selection, Template, TemplateId,
            TemplateInfo,
        },
    },
};

pub fn build_templates(registrar: PipelineActionRegistrar) -> Vec<Template> {
    struct TemplateBuilder {
        id: TemplateId,
        version: u32,
        name: String,
        description: String,
        targets: HashMap<PipelineTarget, Selection<PipelineActionId>>,
    }

    impl TemplateBuilder {
        fn build(self, registrar: &PipelineActionRegistrar) -> Template {
            let actions = registrar.make_lookup(&self.targets);
            Template {
                id: self.id,
                version: self.version,
                pipeline: PipelineDefinition {
                    source_template: TemplateInfo {
                        id: self.id,
                        version: self.version,
                    },
                    name: self.name,
                    description: self.description,
                    targets: self.targets,
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
            version: 0,
            name: "melonDS".to_string(),
            description: "Maps the internal and external monitor to a single virtual screen, as melonDS does not currently support multiple windows. Allows optional melonDS layout configuration.".to_string(),
            targets: HashMap::from_iter([
                (PipelineTarget::Desktop, Selection::AllOf(vec![
                    PipelineActionId::new("core:melonds:config"),
                    PipelineActionId::new("core:display:virtual_screen"),
                ])),
                (PipelineTarget::Gamemode, Selection::AllOf(vec![
                    PipelineActionId::new("core:melonds:config"),
                ]))
            ]),
        },

        // Citra
        TemplateBuilder {
            id: TemplateId::parse("fe82be74-22b9-4135-b7a0-cb6d8f51aecd"),
            version: 0,
            name: "Citra".to_string(),
            description: "Maps primary and secondary windows to different screens for Citra. Allows optional Citra layout configuration.".to_string(),
            targets: HashMap::from_iter([
                (PipelineTarget::Desktop, Selection::AllOf(vec![
                    PipelineActionId::new("core:citra:config"),
                    PipelineActionId::new("core:display:display_config"),
                    PipelineActionId::new("core:display:multi_window"),
                ])),
                (PipelineTarget::Gamemode, Selection::AllOf(vec![
                    PipelineActionId::new("core:citra:config"),
                ]))
            ]),
        },

        // Cemu
        TemplateBuilder {
            id: TemplateId::parse("33c863e5-2739-4bc3-b9bc-4798bac8682d"),
            version: 0,
            name: "Cemu".to_string(),
            description: "Maps primary and secondary windows to different screens for Cemu.".to_string(),
            targets: HashMap::from_iter([
                (PipelineTarget::Desktop,
                    Selection::AllOf(vec![
                        PipelineActionId::new("core:cemu:config"),
                        PipelineActionId::new("core:display:display_config"),
                        PipelineActionId::new("core:display:multi_window"),
                ])),
                (PipelineTarget::Gamemode,
                    Selection::AllOf(vec![
                        PipelineActionId::new("core:cemu:config")
                ]))
            ]),
        },

        // Simple Desktop
        TemplateBuilder {
            id: TemplateId::parse("84f870e9-9491-41a9-8837-d5a6f591f687"),
            version: 0,
            name: "Simple Desktop".to_string(),
            description: "Launches an application in desktop mode.".to_string(),
            targets: HashMap::from_iter([
                (PipelineTarget::Desktop,
                    Selection::AllOf(vec![
                        PipelineActionId::new("core:display:display_config"),
                    ])
                )
            ]),
        }
    ];

    templates.into_iter().map(|t| t.build(&registrar)).collect()
}
