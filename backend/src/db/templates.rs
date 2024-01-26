use std::{collections::HashMap, fmt::Debug};

newtype_uuid!(ProfileId);
newtype_strid!("", AppId);

use crate::{
    macros::{newtype_strid, newtype_uuid},
    pipeline::{
        action_registar::PipelineActionRegistrar,
        data::{
            PipelineActionId, PipelineDefinition, PipelineTarget, Selection, Template, TemplateId,
        },
    },
};

pub fn build_templates(registrar: PipelineActionRegistrar) -> Vec<Template> {
    struct TemplateBuilder {
        id: TemplateId,
        name: String,
        description: String,
        targets: HashMap<PipelineTarget, Selection<PipelineActionId>>,
    }

    impl TemplateBuilder {
        fn build(self, registrar: &PipelineActionRegistrar) -> Template {
            let actions = registrar.make_lookup(&self.targets);
            Template {
                id: self.id,
                pipeline: PipelineDefinition {
                    name: self.name,
                    description: self.description,
                    targets: self.targets,
                    register_exit_hooks: true, // For now, default exit hooks to true, since no configs won't use them.
                    actions,
                },
            }
        }
    }

    let templates = vec![
        // melonDS
        TemplateBuilder {
            id: TemplateId::parse("c6430131-50e0-435e-a917-5ae3cfa46e3c"),
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
            name: "Citra".to_string(),
            description: "Maps primary and secondary windows to different screens for Citra. Allows optional Citra layout configuration.".to_string(),
            targets: HashMap::from_iter([
                (PipelineTarget::Desktop, Selection::AllOf(vec![
                    PipelineActionId::new("core:citra:config"),
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
            name: "Cemu".to_string(),
            description: "Maps primary and secondary windows to different screens for Cemu.".to_string(),
            targets: HashMap::from_iter([
                (PipelineTarget::Desktop,
                    Selection::AllOf(vec![
                        PipelineActionId::new("core:cemu:config"),
                        PipelineActionId::new("core:display:multi_window"),
                ])),
                (PipelineTarget::Gamemode,
                    Selection::AllOf(vec![
                        PipelineActionId::new("core:cemu:config")
                ]))
            ]),
        }
    ];

    templates.into_iter().map(|t| t.build(&registrar)).collect()
}
