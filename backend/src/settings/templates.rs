use std::{collections::HashMap, fmt::Debug};

use anyhow::Result;
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
            description: "Maps primary and secondary windows to different screens for Citra. Allows optional Citra layout configuration".to_string(),
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

#[cfg(test)]
mod tests {

    use std::path::Path;

    use crate::{consts::PACKAGE_NAME, settings::Settings};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_desktop_contents_correct() {
        let settings = Settings::new(
            Path::new("test/out/homebrew/plugins")
                .join(PACKAGE_NAME)
                .join("bin/backend"),
            Path::new("test/out/.config").join(PACKAGE_NAME),
            Path::new("test/out/.config/autostart").to_path_buf(),
            PipelineActionRegistrar::builder().with_core().build(),
        );

        let actual = settings.create_desktop_contents();
        let expected = r"[Desktop Entry]
Comment=Runs DeckDS plugin autostart script for dual screen applications.
Exec=test/out/homebrew/plugins/DeckDS/bin/backend autostart
Path=test/out/homebrew/plugins/DeckDS/bin
Name=DeckDS
Type=Application";

        assert_eq!(expected, actual);
    }
}
