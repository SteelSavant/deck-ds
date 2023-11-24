use std::collections::HashMap;

use super::{
    action::{
        cemu_config::{CemuConfig, CemuXmlSource},
        citra_config::{CitraConfig, CitraIniSource, CitraLayoutOption},
        display_config::{DisplayConfig, RelativeLocation, TeardownExternalSettings},
        multi_window::MultiWindow,
        virtual_screen::VirtualScreen,
    },
    config::{PipelineActionDefinition, PipelineActionDefinitionId, PipelineTarget},
};

use self::internal::{PipelineActionRegistarBuilder, PluginScopeBuilder};

#[derive(Debug)]
pub struct PipelineActionRegistar {
    actions: HashMap<PipelineActionDefinitionId, PipelineActionDefinition>,
}

impl PipelineActionRegistar {
    pub fn builder() -> internal::PipelineActionRegistarBuilder {
        PipelineActionRegistarBuilder::default()
    }
}

mod internal {
    use crate::pipeline::config::PipelineTarget;

    use super::*;

    #[derive(Debug, Default)]
    pub struct PipelineActionRegistarBuilder {
        pub(super) scopes:
            HashMap<String, HashMap<String, HashMap<String, PipelineActionDefinition>>>,
    }

    impl PipelineActionRegistarBuilder {
        pub fn with_plugin<F>(mut self, name: &str, f: F) -> Self
        where
            F: FnOnce(PluginScopeBuilder) -> PluginScopeBuilder,
        {
            self.scopes[name] = f(PluginScopeBuilder::default()).build();
            self
        }
    }

    #[derive(Debug, Default)]
    pub struct PluginScopeBuilder {
        groups: HashMap<String, HashMap<String, PipelineActionDefinition>>,
    }

    impl PluginScopeBuilder {
        pub fn with_group<F>(mut self, name: &str, f: F) -> Self
        where
            F: FnOnce(GroupScopeBuilder) -> GroupScopeBuilder,
        {
            self.groups[name] = f(GroupScopeBuilder::default()).build();
            self
        }

        pub(super) fn build(self) -> HashMap<String, HashMap<String, PipelineActionDefinition>> {
            self.groups
        }
    }

    #[derive(Debug, Default)]
    pub struct GroupScopeBuilder {
        actions: HashMap<String, (Option<PipelineTarget>, PipelineActionDefinition)>,
    }

    impl GroupScopeBuilder {
        pub fn with_action(
            mut self,
            name: &str,
            target: Option<PipelineTarget>,
            action: PipelineActionDefinition,
        ) -> Self {
            self.actions[name] = (target, action);
            self
        }

        fn build(self) -> HashMap<String, PipelineActionDefinition> {
            self.actions
                .into_iter()
                .map(|(k, (t, v))| {
                    (
                        match t {
                            Some(PipelineTarget::Desktop) => format!("{k}:desktop"),
                            Some(PipelineTarget::Gamemode) => format!("{k}:gamemode"),
                            None => k,
                        },
                        v,
                    )
                })
                .collect()
        }
    }
}

impl PipelineActionRegistarBuilder {
    pub fn with_scope<F>(mut self, name: &str, f: F) -> Self
    where
        F: FnOnce(PluginScopeBuilder) -> PluginScopeBuilder,
    {
        self.scopes[name] = f(PluginScopeBuilder::default()).build();
        self
    }

    pub fn build(self) -> PipelineActionRegistar {
        PipelineActionRegistar {
            actions: self
                .scopes
                .into_iter()
                .flat_map(|(scope_id, scope)| {
                    scope.into_iter().flat_map(|(group_id, group)| {
                        group.into_iter().map(|(action_id, action)| {
                            (
                                PipelineActionDefinitionId::new(&format!(
                                    "{scope_id}:{group_id}:{action_id}"
                                )),
                                action,
                            )
                        })
                    })
                })
                .collect(),
        }
    }

    pub fn with_core(mut self) -> Self {
        self.with_scope("core", |scope| {
            scope
                .with_group("display", |group| {
                    group.with_action(
                        "display_config",
                        Some(PipelineTarget::Desktop),
                        PipelineActionDefinition {
                            name: "Display Configuration".to_string(),
                            description: Some("Ensures the display resolution and layout are correctly configured before and after executing pipeline actions.".into()),
                            selection: DisplayConfig {
                                teardown_external_settings: TeardownExternalSettings::Previous,
                                teardown_deck_location: RelativeLocation::Below,
                            } .into(),
                            exported: true,
                        },
                    ).with_action("virtual_screen",      
                                       Some(PipelineTarget::Desktop),
                    PipelineActionDefinition {
                        name: "Virtual Screen".to_string(),
                        description: Some("Maps the internal and external monitor to a single virtual screen, for applications that do not support multiple windows.".into()),
                         selection: VirtualScreen.into(),
                         exported: true,

                    },).with_action("multi_window",    Some(PipelineTarget::Desktop), PipelineActionDefinition {
                        name: "Multi-Window Emulation".to_string(),
                        description: Some("Manages windows for known emulators configurations with multiple display windows.".into()),
                        selection: MultiWindow.into(),
                        exported: true,

                    })
                })
                .with_group("citra", |group| {
                    group.with_action("layout",    Some(PipelineTarget::Desktop),   PipelineActionDefinition {
                        name: "Citra Layout".to_string(),
                        description: Some("Edits Citra ini file to desired layout settings".to_string()),
                        selection: CitraConfig {
                            ini_source: CitraIniSource::Flatpak,
                            layout_option: CitraLayoutOption::Default,
                        }.into(),
                        exported: true,
                    },
                ).with_action("layout",    Some(PipelineTarget::Gamemode),   PipelineActionDefinition {
                    name: "Citra Layout".to_string(),
                    description: Some("Edits Citra ini file to desired layout settings".to_string()),
                    selection: CitraConfig {
                        ini_source: CitraIniSource::Flatpak,
                        layout_option: CitraLayoutOption::HybridScreen,
                    }.into(),
                    exported: true,
                },
            )
                })
                .with_group("cemu", |group| {
                    group.with_action("layout", Some(PipelineTarget::Desktop),     PipelineActionDefinition {
                        name: "Cemu Layout".to_string(),
                        description: Some("Edits Cemu settings.xml file to desired settings".to_string()),
                        selection: CemuConfig {
                            xml_source: CemuXmlSource::Flatpak,
                            separate_gamepad_view: true,
                        }.into(),
                        exported: true
                    },).with_action("layout",  Some(PipelineTarget::Gamemode),    PipelineActionDefinition {
                        name: "Cemu Layout".to_string(),
                        description: Some("Edits Cemu settings.xml file to desired settings".to_string()),
                        selection: CemuConfig {
                            xml_source: CemuXmlSource::Flatpak,
                            separate_gamepad_view: false,
                        }.into(),
                        exported: true
                    },)
                })
                .with_group("melonds", |group| {
                    group
                })
        })
    }
}
