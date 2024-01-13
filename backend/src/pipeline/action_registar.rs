use anyhow::Context;

use crate::settings::ProfileId;

use super::{
    action::{
        cemu_layout::{CemuLayout, CemuLayoutState},
        citra_layout::{CitraLayout, CitraLayoutOption, CitraLayoutState},
        display_restoration::{DisplayRestoration, RelativeLocation, TeardownExternalSettings},
        melonds_layout::{MelonDSLayout, MelonDSLayoutOption, MelonDSSizingOption},
        multi_window::MultiWindow,
        source_file::{CustomFileOptions, EmuDeckSource, FileSource, FlatpakSource, SourceFile},
        virtual_screen::VirtualScreen,
        ActionId,
    },
    data::{
        PipelineActionDefinition, PipelineActionId, PipelineActionLookup, PipelineActionSettings,
        PipelineTarget, Selection,
    },
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

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
        let variant = id.variant(target);
        self.actions.get(&variant).or_else(|| self.actions.get(id))
    }

    pub fn all(&self) -> Arc<HashMap<PipelineActionId, PipelineActionDefinition>> {
        self.actions.clone()
    }

    pub fn make_lookup(
        &self,
        targets: &HashMap<PipelineTarget, Selection<PipelineActionId>>,
    ) -> PipelineActionLookup {
        fn get_ids(
            registrar: &PipelineActionRegistrar,
            selection: &Selection<PipelineActionId>,
            target: PipelineTarget,
        ) -> HashSet<(PipelineActionId, PipelineTarget)> {
            match selection {
                Selection::Action(_) => HashSet::new(),
                Selection::OneOf { actions, .. } | Selection::AllOf(actions) => {
                    let mut ids: HashSet<_> = actions
                        .iter()
                        .map(|id| {
                            registrar
                                .get(id, target)
                                .with_context(|| format!("action {id:?} should exist"))
                                .unwrap()
                        })
                        .flat_map(|def| get_ids(registrar, &def.settings.selection, target))
                        .collect();

                    for a in actions {
                        ids.insert((a.clone(), target));
                    }

                    ids
                }
            }
        }

        let set: HashSet<_> = targets
            .iter()
            .flat_map(|(t, s)| get_ids(self, s, *t))
            .collect();

        let mut actions = HashMap::new();

        for (id, target) in set {
            if let Some(action) = self.get(&id, target) {
                actions.insert(action.id.clone(), action.settings.clone());
            }
        }

        PipelineActionLookup { actions }
    }
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
        actions: HashMap<(String, Option<PipelineTarget>), PipelineActionDefinitionBuilder>,
    }

    impl GroupScopeBuilder {
        pub fn with_action(
            mut self,
            name: &str,
            target: Option<PipelineTarget>,
            action: PipelineActionDefinitionBuilder,
        ) -> Self {
            self.actions.insert((name.to_string(), target), action);
            self
        }

        fn build(self) -> HashMap<String, PipelineActionDefinition> {
            self.actions
                .into_iter()
                .map(|((k, t), v)| {
                    let id = match t {
                        Some(t) => PipelineActionId::new(&k).variant(t).raw().to_string(),
                        None => k,
                    };
                    let action = v.build(PipelineActionId::new(&id));
                    (id, action)
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
        // All actions in the registrar have nil ActionIds, since they're not stored in the DB.
        self.with_scope("core", |scope| {
            scope
                .with_group("display", |group| {
                    group.with_action(
                        "display_config",
                        Some(PipelineTarget::Desktop),
                        PipelineActionDefinitionBuilder {
                            name: "Display Restoration".to_string(),
                            description: Some("Ensures the display resolution and layout are correctly configured before and after executing pipeline actions.".into()),
                            enabled: None,
                            profile_override: None,
                            selection: DisplayRestoration {
                                id: ActionId::nil(),
                                teardown_external_settings: TeardownExternalSettings::Previous,
                                teardown_deck_location: RelativeLocation::Below,
                            } .into(),
                        },
                    ).with_action("virtual_screen",      
                    Some(PipelineTarget::Desktop),
                    PipelineActionDefinitionBuilder {
                        name: "Virtual Screen".to_string(),
                        description: Some("Maps the internal and external monitor to a single virtual screen, for applications that do not support multiple windows.".into()),
                        enabled: None,
                        profile_override: None,
                        selection: VirtualScreen {
                            id: ActionId::nil(),
                        }.into(),
                    },).with_action("multi_window",    Some(PipelineTarget::Desktop), PipelineActionDefinitionBuilder {
                        name: "Multi-Window Emulation".to_string(),
                        description: Some("Manages windows for known emulators configurations with multiple display windows.".into()),
                        enabled: None,
                        profile_override: None,
                        selection: MultiWindow {
                            id: ActionId::nil(),
                        } .into(),
                    })
                })
                .with_group("citra", |group| {
                    group.with_action("config", None, PipelineActionDefinitionBuilder {
                        name: "Citra Configuration".to_string(),
                        description: None,
                        enabled: None,
                        profile_override: None,
                        selection: Selection::AllOf(vec![
                            PipelineActionId::new("core:citra:source"),
                            PipelineActionId::new("core:citra:layout")
                        ]),
                    })
                    .with_action("source", None, PipelineActionDefinitionBuilder {
                        name: "Citra Settings Source".to_string(),
                        description: Some("Source file to use when editing Citra settings.".to_string()),
                        enabled: None,
                        profile_override: None,
                        selection:  Selection::OneOf {selection: PipelineActionId::new("core:citra:flatpak_source"), actions: vec![
                            PipelineActionId::new("core:citra:flatpak_source"),
                            PipelineActionId::new("core:citra:custom_source")
                        ]},
                    })
                    .with_action("flatpak_source", None, PipelineActionDefinitionBuilder {
                        name: "Flatpak".to_string(),
                        description: Some("Sets the settings INI file location to the default Flatpak location.".to_string()),
                        enabled: None,
                        profile_override: None,
                        selection:SourceFile {
                            id: ActionId::nil(),
                            source: FileSource::Flatpak(FlatpakSource::Citra),
                        }.into()
                    })
                    .with_action("custom_source", None, PipelineActionDefinitionBuilder {
                        name: "Custom".to_string(),
                        description: Some("Sets the settings INI file location to a custom location.".to_string()),
                        enabled: None,
                        profile_override: None,
                        selection: SourceFile {
                            id: ActionId::nil(),
                            source: FileSource::Custom(CustomFileOptions {path: None, valid_ext: vec!["ini".to_string()]})
                        }.into(),
                    })
                    .with_action("layout",    Some(PipelineTarget::Desktop),   PipelineActionDefinitionBuilder {
                        name: "Citra Layout".to_string(),
                        description: Some("Edits Citra ini file to desired layout settings.".to_string()),
                        enabled: Some(true),
                        profile_override: None,
                        selection: CitraLayout {
                            id: ActionId::nil(),
                            layout: CitraLayoutState {
                            layout_option: CitraLayoutOption::SeparateWindows,
                            swap_screens: false,
                            fullscreen: true,
                            }
                        }.into(),
                    }).with_action("layout",    Some(PipelineTarget::Gamemode),PipelineActionDefinitionBuilder {
                        name: "Citra Layout".to_string(),
                        description: Some("Edits Citra ini file to desired layout settings.".to_string()),
                        enabled: Some(true),
                        profile_override: None,
                        selection: CitraLayout {
                            id: ActionId::nil(),
                            layout: CitraLayoutState {
                                layout_option: CitraLayoutOption::HybridScreen,
                                swap_screens: false,
                                fullscreen: true,
                            }
                        }.into(),
                    })
                })
                .with_group("cemu", |group| {
                    group.with_action("config", None, PipelineActionDefinitionBuilder {
                        name: "Cemu Configuration".to_string(),
                        description: None,
                        enabled: None,
                        profile_override: None,
                        selection: Selection::AllOf(vec![
                            PipelineActionId::new("core:cemu:source"),
                            PipelineActionId::new("core:cemu:layout")
                        ]),
                    })
                    .with_action("source", None, PipelineActionDefinitionBuilder {
                        name: "Cemu Settings Source".to_string(),
                        description: Some("Source file to use when editing Cemu settings.".to_string()),
                        enabled: None,
                        profile_override: None,
                        selection:  Selection::OneOf {selection: PipelineActionId::new("core:cemu:flatpak_source"), actions: vec![
                            PipelineActionId::new("core:cemu:flatpak_source"),
                            PipelineActionId::new("core:cemu:emudeck_proton_source"),
                            PipelineActionId::new("core:cemu:custom_source")
                        ]},
                    })
                    .with_action("flatpak_source", None, PipelineActionDefinitionBuilder {
                        name: "Flatpak".to_string(),
                        description: Some("Sets the settings INI file location to the default Flatpak location.".to_string()),
                        enabled: None,
                        profile_override: None,
                        selection:SourceFile {
                            id: ActionId::nil(),
                            source: FileSource::Flatpak(FlatpakSource::Cemu)
                        }.into(),
                    })
                    .with_action("emudeck_proton_source", None, PipelineActionDefinitionBuilder {
                        name: "EmuDeck (Proton)".to_string(),
                        description: Some("Sets the settings INI file location to the default EmuDeck (Proton) location.".to_string()),
                        enabled: None,
                        profile_override: None,
                        selection:SourceFile {
                            id: ActionId::nil(),
                            source: FileSource::EmuDeck(EmuDeckSource::CemuProton)
                        }.into(),
                    })
                    .with_action("custom_source", None, PipelineActionDefinitionBuilder {
                        name: "Custom".to_string(),
                        description: Some("Sets the settings XML file location to a custom location.".to_string()),
                        enabled: None,
                        profile_override: None,
                        selection: SourceFile {
                            id: ActionId::nil(),
                            source: FileSource::Custom(CustomFileOptions {path: None, valid_ext: vec!["xml".to_string()]}),
                        }.into()
                    }).with_action("layout", Some(PipelineTarget::Desktop),     PipelineActionDefinitionBuilder {
                        name: "Cemu Layout".to_string(),
                        description: Some("Edits Cemu settings.xml file to desired settings.".to_string()),
                        enabled: Some(true),
                        profile_override: None,
                        selection: CemuLayout {
                            id: ActionId::nil(),
                            layout: CemuLayoutState {
                                separate_gamepad_view: true,
                                fullscreen: true
                            }
                        }.into(),
                    }).with_action("layout",  Some(PipelineTarget::Gamemode),    PipelineActionDefinitionBuilder {
                        name: "Cemu Layout".to_string(),
                        description: Some("Edits Cemu settings.xml file to desired settings.".to_string()),
                        enabled: Some(true),
                        profile_override: None,
                        selection: CemuLayout {
                            id: ActionId::nil(),
                            layout: CemuLayoutState {
                                separate_gamepad_view: false,
                                fullscreen: true
                            }
                        }.into(),
                    })
                })
                .with_group("melonds", |group| {
                    group.with_action("config", None, PipelineActionDefinitionBuilder {
                        name: "melonDS Configuration".to_string(),
                        description: None,
                        enabled: None,
                        profile_override: None,
                        selection: Selection::AllOf(vec![
                            PipelineActionId::new("core:melonds:source"),
                            PipelineActionId::new("core:melonds:layout")
                        ]),
                    })
                    .with_action("source", None, PipelineActionDefinitionBuilder {
                        name: "melonDS Settings Source".to_string(),
                        description: Some("Source file to use when editing melonDS settings.".to_string()),
                        enabled: None,
                        profile_override: None,
                        selection:  Selection::OneOf {selection: PipelineActionId::new("core:melonds:flatpak_source"), actions: vec![
                            PipelineActionId::new("core:melonds:flatpak_source"),
                            PipelineActionId::new("core:melonds:custom_source")
                        ]},
                    })
                    .with_action("flatpak_source", None, PipelineActionDefinitionBuilder {
                        name: "Flatpak".to_string(),
                        description: Some("Sets the settings INI file location to the default Flatpak location.".to_string()),
                        enabled: None,
                        profile_override: None,
                        selection: SourceFile {
                            id: ActionId::nil(),
                            source: FileSource::Flatpak(FlatpakSource::MelonDS)
                        }.into(),
                    })
                    .with_action("custom_source", None, PipelineActionDefinitionBuilder {
                        name: "Custom".to_string(),
                        description: Some("Sets the settings INI file location to a custom location.".to_string()),
                        enabled: None,
                        profile_override: None,
                        selection: SourceFile {
                            id: ActionId::nil(),
                            source: FileSource::Custom(CustomFileOptions {path: None, valid_ext: vec!["ini".to_string()]}),
                        }.into()
                    })
                    .with_action("layout", Some(PipelineTarget::Desktop),     PipelineActionDefinitionBuilder {
                        name: "melonDS Layout".to_string(),
                        description: Some("Edits melonDS ini file to desired layout settings.".to_string()),
                        enabled: Some(true),
                        profile_override: None,
                        selection: MelonDSLayout {
                            id: ActionId::nil(),
                            layout_option: MelonDSLayoutOption::Vertical,
                            sizing_option: MelonDSSizingOption::Even,
                            book_mode: false,
                            swap_screens: false,
                        }.into(),
                    }).with_action("layout", Some(PipelineTarget::Gamemode),    PipelineActionDefinitionBuilder {
                        name: "melonDS Layout".to_string(),
                        description: Some("Edits melonDS ini file to desired settings.".to_string()),
                        enabled: Some(true),
                        profile_override: None,
                        selection: MelonDSLayout {
                            id: ActionId::nil(),
                            layout_option: MelonDSLayoutOption::Hybrid,
                            sizing_option: MelonDSSizingOption::Even,
                            book_mode: false,
                            swap_screens: false,
                        }.into(),
                    })
                })
        })
    }
}

#[derive(Debug)]
pub struct PipelineActionDefinitionBuilder {
    pub name: String,
    pub description: Option<String>,
    /// Flags whether the selection is enabled. If None, not optional. If Some(true), optional and enabled, else disabled.
    pub enabled: Option<bool>,
    /// Flags whether the selection is overridden by the setting from a different profile.
    pub profile_override: Option<ProfileId>,
    /// The value of the pipeline action
    pub selection: Selection<PipelineActionId>,
}

impl PipelineActionDefinitionBuilder {
    pub fn build(self, id: PipelineActionId) -> PipelineActionDefinition {
        PipelineActionDefinition {
            name: self.name,
            description: self.description,
            id,
            settings: PipelineActionSettings {
                enabled: self.enabled,
                profile_override: self.profile_override,
                selection: self.selection,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::hash::RandomState;

    use super::*;

    #[test]
    fn test_action_count() {
        assert_eq!(
            PipelineActionRegistrar::builder()
                .with_core()
                .build()
                .actions
                .len(),
            22
        );
    }

    #[test]
    fn test_make_cemu_lookup() {
        let registrar = PipelineActionRegistrar::builder().with_core().build();

        let targets = HashMap::from_iter([
            (
                PipelineTarget::Desktop,
                Selection::AllOf(vec![
                    PipelineActionId::new("core:cemu:config"),
                    PipelineActionId::new("core:display:multi_window"),
                ]),
            ),
            (
                PipelineTarget::Gamemode,
                Selection::AllOf(vec![PipelineActionId::new("core:cemu:config")]),
            ),
        ]);

        let lookup = registrar.make_lookup(&targets);
        let expected_keys: HashSet<PipelineActionId, RandomState> = HashSet::from_iter(
            [
                "core:display:multi_window:desktop",
                "core:cemu:config",
                "core:cemu:source",
                "core:cemu:flatpak_source",
                "core:cemu:emudeck_proton_source",
                "core:cemu:custom_source",
                "core:cemu:layout:desktop",
                "core:cemu:layout:gamemode",
            ]
            .map(|v| PipelineActionId::new(v)),
        );
        let actual_keys = lookup
            .actions
            .into_keys()
            .collect::<HashSet<PipelineActionId>>();

        let intersection = expected_keys
            .intersection(&actual_keys)
            .into_iter()
            .map(|a| a.clone())
            .collect::<HashSet<_>>();
        let difference = expected_keys
            .difference(&actual_keys)
            .into_iter()
            .map(|a| a.clone())
            .collect::<HashSet<_>>();

        assert_eq!(difference.len(), 0);
        assert_eq!(intersection.len(), expected_keys.len());
    }
}
