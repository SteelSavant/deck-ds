use strum::IntoEnumIterator;

use crate::settings::ProfileId;

use super::{
    action::{
        cemu_layout::{CemuLayout, CemuLayoutState},
        citra_layout::{CitraLayout, CitraLayoutOption, CitraLayoutState},
        display_config::DisplayConfig,
        melonds_layout::{MelonDSLayout, MelonDSLayoutOption, MelonDSSizingOption},
        multi_window::{CemuWindowOptions, CitraWindowOptions, GeneralOptions, MultiWindow},
        session_handler::{DesktopSessionHandler, ExternalDisplaySettings, RelativeLocation},
        source_file::{
            AppImageSource, CustomFileOptions, EmuDeckSource, FileSource, FlatpakSource, SourceFile,
        },
        virtual_screen::VirtualScreen,
        ActionId,
    },
    data::{
        DefinitionSelection, PipelineActionDefinition, PipelineActionId, PipelineActionLookup,
        PipelineActionSettings, PipelineTarget,
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

    pub fn make_lookup(&self, root: &PipelineActionId) -> PipelineActionLookup {
        fn get_ids(
            registrar: &PipelineActionRegistrar,
            selection: &DefinitionSelection,
            target: PipelineTarget,
        ) -> HashSet<(PipelineActionId, PipelineTarget)> {
            match selection {
                DefinitionSelection::Action(_) | DefinitionSelection::UserDefined => HashSet::new(),
                DefinitionSelection::OneOf { actions, .. }
                | DefinitionSelection::AllOf(actions) => {
                    let mut ids: HashSet<_> = actions
                        .iter()
                        .filter_map(|id| registrar.get(id, target))
                        .flat_map(|def| get_ids(registrar, &def.settings.selection, target))
                        .collect();

                    for a in actions {
                        ids.insert((a.clone(), target));
                    }

                    ids
                }
            }
        }

        let set: HashSet<_> = PipelineTarget::iter()
            .flat_map(|t| get_ids(self, &DefinitionSelection::AllOf(vec![root.clone()]), t))
            .collect();

        let mut actions = HashMap::new();

        for (id, target) in set {
            if let Some(action) = self.get(&id, target) {
                actions.insert(action.id.clone(), action.settings.clone().into());
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
            let multi_window_name = "Multi-Window Emulation".to_string();
            let multi_window_description = Some("Manages windows for known emulators configurations with multiple display windows.".to_string());

            scope
                .with_group("display", |group| {
                    group.with_action(
                        "desktop_session",
                        Some(PipelineTarget::Desktop),
                        PipelineActionDefinitionBuilder {
                            name: "Desktop Session".to_string(),
                            description: Some("Ensures the display resolution and layout are correctly configured before and after executing pipeline actions.".into()),
                            enabled: None,
                            is_visible_on_qam: false,
                            profile_override: None,
                            selection: DesktopSessionHandler {
                                id: ActionId::nil(),
                                teardown_external_settings: ExternalDisplaySettings::Previous,
                                teardown_deck_location: Some(RelativeLocation::Below),
                                deck_is_primary_display: true,
                            } .into(),
                        },
                    )
                    .with_action("display_config", Some(PipelineTarget::Desktop), 
                PipelineActionDefinitionBuilder {
                            name: "Display Config".to_string(),
                            description: Some("Configures displays in desktop mode.".to_string()),
                            enabled: None,
                            is_visible_on_qam: false,
                            profile_override: None,
                            selection: DisplayConfig {
                                id: ActionId::nil(),
                                external_display_settings: ExternalDisplaySettings::Previous,
                                deck_location: Some(RelativeLocation::Below),
                                deck_is_primary_display: true,
                            }.into()
                        })
                    .with_action("virtual_screen",      
                    Some(PipelineTarget::Desktop),
                    PipelineActionDefinitionBuilder {
                        name: "Virtual Screen".to_string(),
                        description: Some("Maps the internal and external monitor to a single virtual screen, for applications that do not support multiple windows.".into()),
                        enabled: None,
                        is_visible_on_qam: true,
                        profile_override: None,
                        selection: VirtualScreen {
                            id: ActionId::nil(),
                        }.into(),
                    })
                })
                .with_group("citra", |group| {
                    let citra_name = "Citra".to_string();
                    let citra_description = Some("Maps primary and secondary windows to different screens for Citra. Allows optional Citra layout configuration.".to_string());

                    let citra_layout_name = "Layout".to_string();
                    let citra_layout_description = Some("Edits Citra ini file to desired layout settings.".to_string());

                    group.with_action("root", None, PipelineActionDefinitionBuilder {
                        name: citra_name.clone(),
                        description: citra_description.clone(),
                        enabled: None,
                        profile_override: None,
                        selection: DefinitionSelection::AllOf(vec![
                            PipelineActionId::new("core:citra:source"),
                            PipelineActionId::new("core:citra:layout"),
                            PipelineActionId::new("core:citra:multi_window"),
                            PipelineActionId::new("core:display:display_config"),
                        ]),
                        is_visible_on_qam: true,
                    })
                    .with_action("source", None, PipelineActionDefinitionBuilder {
                        name: "Citra Settings Source".to_string(),
                        description: Some("Source file to use when editing Citra settings.".to_string()),
                        enabled: None,
                        is_visible_on_qam: false,
                        profile_override: None,
                        selection:  DefinitionSelection::OneOf {selection: PipelineActionId::new("core:citra:flatpak_source"), actions: vec![
                            PipelineActionId::new("core:citra:flatpak_source"),
                            PipelineActionId::new("core:citra:custom_source")
                        ]},
                    })
                    .with_action("flatpak_source", None, PipelineActionDefinitionBuilder {
                        name: "Flatpak".to_string(),
                        description: Some("Sets the settings INI file location to the default Flatpak location.".to_string()),
                        enabled: None,
                        is_visible_on_qam: true,
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
                        is_visible_on_qam: true,
                        profile_override: None,
                        selection: SourceFile {
                            id: ActionId::nil(),
                            source: FileSource::Custom(CustomFileOptions {path: None, valid_ext: vec!["ini".to_string()]})
                        }.into(),
                    })
                    .with_action("layout", Some(PipelineTarget::Desktop),   PipelineActionDefinitionBuilder {
                        name: citra_layout_name.clone(),
                        description: citra_layout_description.clone(),
                        enabled: Some(true),
                        is_visible_on_qam: true,
                        profile_override: None,
                        selection: CitraLayout {
                            id: ActionId::nil(),
                            layout: CitraLayoutState {
                            layout_option: CitraLayoutOption::SeparateWindows,
                            swap_screens: false,
                            fullscreen: true,
                            }
                        }.into(),
                    }).with_action("layout", Some(PipelineTarget::Gamemode),PipelineActionDefinitionBuilder {
                        name: citra_layout_name.clone(),
                        description: citra_layout_description.clone(),
                        enabled: Some(true),
                        is_visible_on_qam: true,
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
                    .with_action("multi_window",Some(PipelineTarget::Desktop), PipelineActionDefinitionBuilder {
                        name: multi_window_name.clone(),
                        description: multi_window_description.clone(),
                        enabled: None,
                        is_visible_on_qam: false,
                        profile_override: None,
                        selection: MultiWindow {
                            id: ActionId::nil(),
                            general: GeneralOptions::default(),
                            citra: Some(CitraWindowOptions::default()),
                            cemu: None,
                            dolphin: None,
                            custom: None,
                        }.into(),
                    })
                })
                .with_group("cemu", |group| {
                    let cemu_name = "Cemu".to_string();
                    let cemu_description = Some("Maps primary and secondary windows to different screens for Cemu.".to_string());
                    let cemu_layout_name = "Layout".to_string();
                    let cemu_layout_description = Some("Edits Cemu settings.xml file to desired settings.".to_string());

                    group.with_action("root", None, PipelineActionDefinitionBuilder {
                        name: cemu_name.clone(),
                        description: cemu_description.clone(),
                        enabled: None,
                        profile_override: None,
                        selection: DefinitionSelection::AllOf(vec![
                            PipelineActionId::new("core:cemu:source"),
                            PipelineActionId::new("core:cemu:layout"),
                            PipelineActionId::new("core:cemu:multi_window"),
                            PipelineActionId::new("core:display:display_config"),
                        ]),
                        is_visible_on_qam: true,
                    })
                    .with_action("source", None, PipelineActionDefinitionBuilder {
                        name: "Cemu Settings Source".to_string(),
                        description: Some("Source file to use when editing Cemu settings.".to_string()),
                        enabled: None,
                        is_visible_on_qam: false,
                        profile_override: None,
                        selection:  DefinitionSelection::OneOf {selection: PipelineActionId::new("core:cemu:flatpak_source"), actions: vec![
                            PipelineActionId::new("core:cemu:flatpak_source"),
                            PipelineActionId::new("core:cemu:appimage_source"),
                            PipelineActionId::new("core:cemu:emudeck_proton_source"),
                            PipelineActionId::new("core:cemu:custom_source")
                        ]},
                    })
                    .with_action("flatpak_source", None, PipelineActionDefinitionBuilder {
                        name: "Flatpak".to_string(),
                        description: Some("Sets the settings XML file location to the default Flatpak location.".to_string()),
                        enabled: None,
                        is_visible_on_qam: true,
                        profile_override: None,
                        selection:SourceFile {
                            id: ActionId::nil(),
                            source: FileSource::Flatpak(FlatpakSource::Cemu)
                        }.into(),
                    })
                    .with_action("appimage_source", None,PipelineActionDefinitionBuilder {
                        name: "AppImage".to_string(),
                        description: Some("Sets the settings XML file location to the default AppImage location.".to_string()),
                        enabled: None,
                        is_visible_on_qam: true,
                        profile_override: None,
                        selection: SourceFile {
                            id: ActionId::nil(),
                            source: FileSource::AppImage(AppImageSource::Cemu)
                        }.into(),
                    })
                    .with_action("emudeck_proton_source", None, PipelineActionDefinitionBuilder {
                        name: "EmuDeck (Proton)".to_string(),
                        description: Some("Sets the settings XML file location to the location of EmuDeck's Cemu (Proton).".to_string()),
                        enabled: None,
                        is_visible_on_qam: true,
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
                        is_visible_on_qam: true,
                        profile_override: None,
                        selection: SourceFile {
                            id: ActionId::nil(),
                            source: FileSource::Custom(CustomFileOptions {path: None, valid_ext: vec!["xml".to_string()]}),
                        }.into()
                    }).with_action("layout", Some(PipelineTarget::Desktop),     PipelineActionDefinitionBuilder {
                        name: cemu_layout_name.clone(),
                        description: cemu_layout_description.clone(),
                        enabled: Some(true),
                        is_visible_on_qam: true,
                        profile_override: None,
                        selection: CemuLayout {
                            id: ActionId::nil(),
                            layout: CemuLayoutState {
                                separate_gamepad_view: true,
                                fullscreen: true
                            }
                        }.into(),
                    }).with_action("layout",  Some(PipelineTarget::Gamemode),    PipelineActionDefinitionBuilder {
                        name: cemu_layout_name.to_string(),
                        description: cemu_description.clone(),
                        enabled: Some(true),
                        is_visible_on_qam: true,
                        profile_override: None,
                        selection: CemuLayout {
                            id: ActionId::nil(),
                            layout: CemuLayoutState {
                                separate_gamepad_view: false,
                                fullscreen: true
                            }
                        }.into(),
                    })
                    .with_action("multi_window",Some(PipelineTarget::Desktop), PipelineActionDefinitionBuilder {
                        name: multi_window_name.clone(),
                        description: multi_window_description.clone(),
                        enabled: None,
                        is_visible_on_qam: false,
                        profile_override: None,
                        selection: MultiWindow {
                            id: ActionId::nil(),
                            general: GeneralOptions::default(),
                            cemu: Some(CemuWindowOptions::default()),
                            citra: None,
                            dolphin: None,
                            custom: None,
                        }.into(),
                    })
                })
                .with_group("melonds", |group| {
                    let melonds_name = "melonDS".to_string();
                    let melonds_description = Some("Maps the internal and external monitor to a single virtual screen, as melonDS does not currently support multiple windows. Allows optional melonDS layout configuration.".to_string());
                    let melonds_layout_name = "Layout".to_string();
                    let melonds_layout_description = Some("Edits melonDS ini file to desired layout settings.".to_string());

                    group.with_action("root", None, PipelineActionDefinitionBuilder {
                        name: melonds_name.clone(),
                        description: melonds_description.clone(),
                        enabled: None,
                        profile_override: None,
                        selection: DefinitionSelection::AllOf(vec![
                            PipelineActionId::new("core:melonds:source"),
                            PipelineActionId::new("core:melonds:layout"),
                            PipelineActionId::new("core:display:virtual_screen"),
                        ]),
                        is_visible_on_qam: true,
                    })
                    .with_action("source", None, PipelineActionDefinitionBuilder {
                        name: "melonDS Settings Source".to_string(),
                        description: Some("Source file to use when editing melonDS settings.".to_string()),
                        enabled: None,
                        is_visible_on_qam: false,
                        profile_override: None,
                        selection:  DefinitionSelection::OneOf {selection: PipelineActionId::new("core:melonds:flatpak_source"), actions: vec![
                            PipelineActionId::new("core:melonds:flatpak_source"),
                            PipelineActionId::new("core:melonds:custom_source")
                        ]},
                    })
                    .with_action("flatpak_source", None, PipelineActionDefinitionBuilder {
                        name: "Flatpak".to_string(),
                        description: Some("Sets the settings INI file location to the default Flatpak location.".to_string()),
                        enabled: None,
                        is_visible_on_qam: true,
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
                        is_visible_on_qam: true,
                        profile_override: None,
                        selection: SourceFile {
                            id: ActionId::nil(),
                            source: FileSource::Custom(CustomFileOptions {path: None, valid_ext: vec!["ini".to_string()]}),
                        }.into()
                    })
                    .with_action("layout", Some(PipelineTarget::Desktop),     PipelineActionDefinitionBuilder {
                        name: melonds_layout_name.clone(),
                        description: melonds_layout_description.clone(),
                        enabled: Some(true),
                        is_visible_on_qam: true,
                        profile_override: None,
                        selection: MelonDSLayout {
                            id: ActionId::nil(),
                            layout_option: MelonDSLayoutOption::Vertical,
                            sizing_option: MelonDSSizingOption::Even,
                            book_mode: false,
                            swap_screens: false,
                        }.into(),
                    }).with_action("layout", Some(PipelineTarget::Gamemode),    PipelineActionDefinitionBuilder {
                        name: melonds_layout_name.clone(),
                        description: melonds_layout_description.clone(),
                        enabled: Some(true),
                        is_visible_on_qam: true,
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
                .with_group("app", |group| {
                    let app_name =  "App".to_string();
                    let app_description = Some("Launches an application in desktop mode.".to_string());

                    group.with_action("root", Some(PipelineTarget::Desktop), PipelineActionDefinitionBuilder {
                        name: app_name.clone(),
                        description: app_description.clone(),
                        enabled: None,
                        profile_override: None,
                        is_visible_on_qam: true,
                        selection: DefinitionSelection::AllOf(vec![
                            PipelineActionId::new("core:app:multi_window"),
                            PipelineActionId::new("core:display:display_config"),
                        ]),
                    })
                })
        })
        // .with_scope("secondary", |scope| {
        //     todo!("secondary actions; for now, browser flatpaks (or custom) sites")
        // })
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

    /// If true, the action is visible to be configured on the quick-access menu.
    pub is_visible_on_qam: bool,
    /// The value of the pipeline action.
    pub selection: DefinitionSelection,
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
                is_visible_on_qam: self.is_visible_on_qam,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::hash::RandomState;

    use super::*;

    #[test]
    fn test_make_cemu_lookup() {
        let registrar = PipelineActionRegistrar::builder().with_core().build();

        let root = PipelineActionId::new("core:cemu:root");

        let lookup = registrar.make_lookup(&root);
        let expected_keys: HashSet<PipelineActionId, RandomState> = HashSet::from_iter(
            [
                "core:cemu:root",
                "core:cemu:multi_window:desktop",
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
