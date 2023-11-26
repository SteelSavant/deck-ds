use super::{
    action::{
        cemu_config::{CemuConfig, CemuXmlSource},
        citra_config::{CitraConfig, CitraIniSource, CitraLayoutOption},
        display_config::{DisplayConfig, RelativeLocation, TeardownExternalSettings},
        multi_window::MultiWindow,
        virtual_screen::VirtualScreen,
    },
    data::{PipelineActionDefinition, PipelineActionId, PipelineTarget},
};
use std::{collections::HashMap, sync::Arc};

use self::internal::{PipelineActionRegistarBuilder, PluginScopeBuilder};

#[derive(Debug, Clone)]
pub struct PipelineActionRegistrar {
    actions: Arc<HashMap<PipelineActionId, PipelineActionDefinition>>,
}

impl PipelineActionRegistrar {
    pub fn builder() -> internal::PipelineActionRegistarBuilder {
        PipelineActionRegistarBuilder::default()
    }

    pub fn get(
        &self,
        id: &PipelineActionId,
        target: PipelineTarget,
    ) -> Option<&PipelineActionDefinition> {
        self.actions
            .get(&format_variant(id.raw(), target))
            .or_else(|| self.actions.get(id))
    }

    pub fn all(&self) -> Arc<HashMap<PipelineActionId, PipelineActionDefinition>> {
        self.actions.clone()
    }
}

fn format_variant(id: &str, target: PipelineTarget) -> PipelineActionId {
    let variant = match target {
        PipelineTarget::Desktop => "desktop",
        PipelineTarget::Gamemode => "gamemode",
    };

    PipelineActionId::new(&format!("{id}:{variant}"))
}

mod internal {
    use crate::pipeline::data::PipelineTarget;

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
            self.scopes
                .insert(name.to_string(), f(PluginScopeBuilder::default()).build());
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
            self.groups
                .insert(name.to_string(), f(GroupScopeBuilder::default()).build());
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
            self.actions.insert(name.to_string(), (target, action));
            self
        }

        fn build(self) -> HashMap<String, PipelineActionDefinition> {
            self.actions
                .into_iter()
                .map(|(k, (t, v))| {
                    let id = match t {
                        Some(t) => format_variant(&k, t).raw().to_string(),
                        None => k,
                    };
                    (id, v)
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
        self.scopes
            .insert(name.to_string(), f(PluginScopeBuilder::default()).build());
        self
    }

    pub fn build(self) -> PipelineActionRegistrar {
        let actions = self
            .scopes
            .into_iter()
            .flat_map(|(ref scope_id, scope)| {
                scope
                    .into_iter()
                    .flat_map(|(ref group_id, group)| {
                        group
                            .into_iter()
                            .map(move |(ref action_id, action)| {
                                let id = PipelineActionId::new(&format!(
                                    "{scope_id}:{group_id}:{action_id}"
                                ));
                                (id.clone(), PipelineActionDefinition { id, ..action })
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        PipelineActionRegistrar {
            actions: Arc::new(actions),
        }
    }

    pub fn with_core(self) -> Self {
        self.with_scope("core", |scope| {
            scope
                .with_group("display", |group| {
                    group.with_action(
                        "display_config",
                        Some(PipelineTarget::Desktop),
                        PipelineActionDefinition {
                            id: PipelineActionId::new(""),
                            name: "Display Configuration".to_string(),
                            description: Some("Ensures the display resolution and layout are correctly configured before and after executing pipeline actions.".into()),
                            selection: DisplayConfig {
                                teardown_external_settings: TeardownExternalSettings::Previous,
                                teardown_deck_location: RelativeLocation::Below,
                            } .into(),
                        },
                    ).with_action("virtual_screen",      
                                       Some(PipelineTarget::Desktop),
                    PipelineActionDefinition {
                        id: PipelineActionId::new(""),
                        name: "Virtual Screen".to_string(),
                        description: Some("Maps the internal and external monitor to a single virtual screen, for applications that do not support multiple windows.".into()),
                        selection: VirtualScreen.into(),
                    },).with_action("multi_window",    Some(PipelineTarget::Desktop), PipelineActionDefinition {
                        id: PipelineActionId::new(""),
                        name: "Multi-Window Emulation".to_string(),
                        description: Some("Manages windows for known emulators configurations with multiple display windows.".into()),
                        selection: MultiWindow.into(),

                    })
                })
                .with_group("citra", |group| {
                    group.with_action("layout",    Some(PipelineTarget::Desktop),   PipelineActionDefinition {
                        id: PipelineActionId::new(""),
                        name: "Citra Layout".to_string(),
                        description: Some("Edits Citra ini file to desired layout settings".to_string()),
                        selection: CitraConfig {
                            ini_source: CitraIniSource::Flatpak,
                            layout_option: CitraLayoutOption::Default,
                        }.into(),
                    },
                ).with_action("layout",    Some(PipelineTarget::Gamemode),PipelineActionDefinition {
                    id: PipelineActionId::new(""),
                    name: "Citra Layout".to_string(),
                    description: Some("Edits Citra ini file to desired layout settings".to_string()),
                    selection: CitraConfig {
                        ini_source: CitraIniSource::Flatpak,
                        layout_option: CitraLayoutOption::HybridScreen,
                    }.into(),
                },
            )
                })
                .with_group("cemu", |group| {
                    group.with_action("layout", Some(PipelineTarget::Desktop),     PipelineActionDefinition {
                        id: PipelineActionId::new(""),
                        name: "Cemu Layout".to_string(),
                        description: Some("Edits Cemu settings.xml file to desired settings".to_string()),
                        selection: CemuConfig {
                            xml_source: CemuXmlSource::Flatpak,
                            separate_gamepad_view: true,
                        }.into(),
                    },).with_action("layout",  Some(PipelineTarget::Gamemode),    PipelineActionDefinition {
                        id: PipelineActionId::new(""),
                        name: "Cemu Layout".to_string(),
                        description: Some("Edits Cemu settings.xml file to desired settings".to_string()),
                        selection: CemuConfig {
                            xml_source: CemuXmlSource::Flatpak,
                            separate_gamepad_view: false,
                        }.into(),
                    },)
                })
                .with_group("melonds", |group| {
                    group
                })
        })
    }
}
