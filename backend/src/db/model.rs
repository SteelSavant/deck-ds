use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

use std::path::PathBuf;

use crate::{
    native_model_serde_json::NativeModelJSON,
    pipeline::{
        action::{
            cemu_layout::{CemuLayout, CemuLayoutState},
            citra_layout::{CitraLayout, CitraLayoutOption, CitraLayoutState},
            display_config::DisplayConfig,
            melonds_layout::{MelonDSLayout, MelonDSLayoutOption, MelonDSSizingOption},
            session_handler::DesktopSessionHandler,
            ActionId,
        },
        data::generic,
    },
    settings::{AppId, ProfileId},
};

// Core

pub type DbCategoryProfile = v1::DbCategoryProfile;
pub type DbPipelineDefinition = v1::DbPipelineDefinition;
pub type DbSelection<T> = v1::DbSelection<T>;
pub type DbAction = v1::DbAction;
pub type DbPipelineActionLookup = v1::DbPipelineActionLookup;
pub type DbPipelineActionSettings = v1::DbPipelineActionSettings;

// Action

pub type DbCemuLayout = v1::DbCemuLayout;
pub type DbCitraLayout = v1::DbCitraLayout;
pub type DbMelonDSLayout = v1::DbMelonDSLayout;
pub type DbDesktopSessionHandler = v1::DbDesktopSessionHandler;
pub type DbMultiWindow = v1::DbMultiWindow;
pub type DbSourceFile = v1::DbSourceFile;
pub type DbVirtualScreen = v1::DbVirtualScreen;
pub type DbDesktopConfig = v1::DbDisplayConfig;

pub mod v1 {
    // Core

    use crate::{
        pipeline::action::{
            multi_window::MultiWindow,
            session_handler::{ExternalDisplaySettings, RelativeLocation},
            source_file::{
                AppImageSource, CustomFileOptions, EmuDeckSource, FileSource, FlatpakSource,
                SourceFile,
            },
            virtual_screen::VirtualScreen,
        },
        sys::x_display::{AspectRatioOption, ModeOption, ModePreference, Resolution},
    };

    use super::*;

    #[derive(Serialize, Deserialize)]
    #[native_db]
    #[native_model(id = 1, version = 1, with = NativeModelJSON)]
    pub struct DbCategoryProfile {
        #[primary_key]
        pub id: ProfileId,
        pub tags: Vec<String>,
        pub pipeline: DbPipelineDefinition,
    }

    #[derive(Serialize, Deserialize)]
    #[native_db]
    #[native_model(id = 2, version = 1, with = NativeModelJSON)]
    pub struct DbAppOverride {
        #[primary_key]
        pub id: (AppId, ProfileId),
        pub pipeline: DbPipelineDefinition,
    }

    #[derive(Serialize, Deserialize)]
    #[native_db]
    #[native_model(id = 3, version = 1, with = NativeModelJSON)]
    pub struct DbAppSettings {
        #[primary_key]
        pub app_id: AppId,
        pub default_profile: Option<ProfileId>,
    }

    pub type DbPipelineDefinition = generic::PipelineDefinition<DbAction>;
    pub type DbSelection<T> = generic::Selection<DbAction, T>;
    pub type DbPipelineActionSettings = generic::PipelineActionSettings<DbAction>;
    pub type DbPipelineActionLookup = generic::PipelineActionLookup<DbAction>;

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub enum DbAction {
        DesktopSessionHandler(ActionId),
        DisplayConfig(ActionId),
        VirtualScreen(ActionId),
        MultiWindow(ActionId),
        CitraLayout(ActionId),
        CemuLayout(ActionId),
        MelonDSLayout(ActionId),
        SourceFile(ActionId),
    }

    // Actions

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[native_db]
    #[native_model(id = 4, version = 1, with = NativeModelJSON)]
    pub struct DbCemuLayout {
        #[primary_key]
        pub id: ActionId,
        pub separate_gamepad_view: bool,
        pub fullscreen: bool,
    }

    impl From<CemuLayout> for DbCemuLayout {
        fn from(value: CemuLayout) -> Self {
            Self {
                id: value.id,
                separate_gamepad_view: value.layout.separate_gamepad_view,
                fullscreen: value.layout.fullscreen,
            }
        }
    }

    impl From<DbCemuLayout> for CemuLayout {
        fn from(value: DbCemuLayout) -> Self {
            Self {
                id: value.id,
                layout: CemuLayoutState {
                    separate_gamepad_view: value.separate_gamepad_view,
                    fullscreen: value.fullscreen,
                },
            }
        }
    }

    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
    #[native_db]
    #[native_model(id = 5, version = 1, with = NativeModelJSON)]
    pub struct DbCitraLayout {
        #[primary_key]
        pub id: ActionId,
        pub layout_option: DbCitraLayoutOption,
        pub swap_screens: bool,
        pub fullscreen: bool,
    }

    impl From<CitraLayout> for DbCitraLayout {
        fn from(value: CitraLayout) -> Self {
            Self {
                id: value.id,
                layout_option: match value.layout.layout_option {
                    CitraLayoutOption::Default => DbCitraLayoutOption::Default,
                    CitraLayoutOption::SingleScreen => DbCitraLayoutOption::SingleScreen,
                    CitraLayoutOption::LargeScreen => DbCitraLayoutOption::LargeScreen,
                    CitraLayoutOption::SideBySide => DbCitraLayoutOption::SideBySide,
                    CitraLayoutOption::SeparateWindows => DbCitraLayoutOption::SeparateWindows,
                    CitraLayoutOption::HybridScreen => DbCitraLayoutOption::HybridScreen,
                    CitraLayoutOption::Unknown(v) => DbCitraLayoutOption::Unknown(v),
                },
                swap_screens: value.layout.swap_screens,
                fullscreen: value.layout.fullscreen,
            }
        }
    }

    impl From<DbCitraLayout> for CitraLayout {
        fn from(value: DbCitraLayout) -> Self {
            Self {
                id: value.id,
                layout: CitraLayoutState {
                    layout_option: match value.layout_option {
                        DbCitraLayoutOption::Default => CitraLayoutOption::Default,
                        DbCitraLayoutOption::SingleScreen => CitraLayoutOption::SingleScreen,
                        DbCitraLayoutOption::LargeScreen => CitraLayoutOption::LargeScreen,
                        DbCitraLayoutOption::SideBySide => CitraLayoutOption::SideBySide,
                        DbCitraLayoutOption::SeparateWindows => CitraLayoutOption::SeparateWindows,
                        DbCitraLayoutOption::HybridScreen => CitraLayoutOption::HybridScreen,
                        DbCitraLayoutOption::Unknown(v) => CitraLayoutOption::Unknown(v),
                    },
                    swap_screens: value.swap_screens,
                    fullscreen: value.fullscreen,
                },
            }
        }
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(tag = "type", content = "value")]
    pub enum DbCitraLayoutOption {
        Default,         // 0
        SingleScreen,    // 1
        LargeScreen,     // 2
        SideBySide,      // 3
        SeparateWindows, // 4
        HybridScreen,    // 5
        Unknown(u64),
    }

    #[derive(Debug, Copy, Clone, Serialize, Deserialize)]
    #[native_db]
    #[native_model(id = 6, version = 1, with = NativeModelJSON)]
    pub struct DbMelonDSLayout {
        #[primary_key]
        pub id: ActionId,
        pub layout_option: DbMelonDSLayoutOption,
        pub sizing_option: DbMelonDSSizingOption,
        pub book_mode: bool, // if in book mode, set rotation to 270,
        pub swap_screens: bool,
    }

    impl From<MelonDSLayout> for DbMelonDSLayout {
        fn from(value: MelonDSLayout) -> Self {
            Self {
                id: value.id,
                layout_option: match value.layout_option {
                    MelonDSLayoutOption::Natural => DbMelonDSLayoutOption::Natural,
                    MelonDSLayoutOption::Vertical => DbMelonDSLayoutOption::Vertical,
                    MelonDSLayoutOption::Horizontal => DbMelonDSLayoutOption::Horizontal,
                    MelonDSLayoutOption::Hybrid => DbMelonDSLayoutOption::Hybrid,
                    MelonDSLayoutOption::Single => DbMelonDSLayoutOption::Single,
                },
                sizing_option: match value.sizing_option {
                    MelonDSSizingOption::Even => DbMelonDSSizingOption::Even,
                    MelonDSSizingOption::EmphasizeTop => DbMelonDSSizingOption::EmphasizeTop,
                    MelonDSSizingOption::EmphasizeBottom => DbMelonDSSizingOption::EmphasizeBottom,
                    MelonDSSizingOption::Auto => DbMelonDSSizingOption::Auto,
                },
                book_mode: value.book_mode,
                swap_screens: value.swap_screens,
            }
        }
    }

    impl From<DbMelonDSLayout> for MelonDSLayout {
        fn from(value: DbMelonDSLayout) -> Self {
            Self {
                id: value.id,
                layout_option: match value.layout_option {
                    DbMelonDSLayoutOption::Natural => MelonDSLayoutOption::Natural,
                    DbMelonDSLayoutOption::Vertical => MelonDSLayoutOption::Vertical,
                    DbMelonDSLayoutOption::Horizontal => MelonDSLayoutOption::Horizontal,
                    DbMelonDSLayoutOption::Hybrid => MelonDSLayoutOption::Hybrid,
                    DbMelonDSLayoutOption::Single => MelonDSLayoutOption::Single,
                },
                sizing_option: match value.sizing_option {
                    DbMelonDSSizingOption::Even => MelonDSSizingOption::Even,
                    DbMelonDSSizingOption::EmphasizeTop => MelonDSSizingOption::EmphasizeTop,
                    DbMelonDSSizingOption::EmphasizeBottom => MelonDSSizingOption::EmphasizeBottom,
                    DbMelonDSSizingOption::Auto => MelonDSSizingOption::Auto,
                },
                book_mode: value.book_mode,
                swap_screens: value.swap_screens,
            }
        }
    }

    #[derive(Debug, Copy, Clone, Serialize, Deserialize)]
    pub enum DbMelonDSLayoutOption {
        Natural,    // Puts screens vertical normally, horizonal in book mode.
        Vertical,   // Puts screens vertical always,
        Horizontal, // Puts screens horizonal always,
        Hybrid, // Puts main screen large, with both screens adjacent. Overrides sizing settings.
        Single, // Displays only one screen,
    }

    #[derive(Debug, Copy, Clone, Serialize, Deserialize)]
    pub enum DbMelonDSSizingOption {
        Even,
        EmphasizeTop,
        EmphasizeBottom,
        Auto,
    }

    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    #[native_db]
    #[native_model(id = 7, version = 1, with = NativeModelJSON)]
    pub struct DbDesktopSessionHandler {
        #[primary_key]
        pub id: ActionId,
        pub teardown_external_settings: DbExternalDisplaySettings,
        pub teardown_deck_location: Option<DbRelativeLocation>,
    }

    impl From<DesktopSessionHandler> for DbDesktopSessionHandler {
        fn from(value: DesktopSessionHandler) -> Self {
            Self {
                id: value.id,
                teardown_external_settings: value.teardown_external_settings.into(),
                teardown_deck_location: value.teardown_deck_location.map(|v| match v {
                    RelativeLocation::Above => DbRelativeLocation::Above,
                    RelativeLocation::Below => DbRelativeLocation::Below,
                    RelativeLocation::LeftOf => DbRelativeLocation::LeftOf,
                    RelativeLocation::RightOf => DbRelativeLocation::RightOf,
                    RelativeLocation::SameAs => DbRelativeLocation::SameAs,
                }),
            }
        }
    }

    impl From<ExternalDisplaySettings> for DbExternalDisplaySettings {
        fn from(value: ExternalDisplaySettings) -> Self {
            match value {
                ExternalDisplaySettings::Previous => DbExternalDisplaySettings::Previous,
                ExternalDisplaySettings::Native => DbExternalDisplaySettings::Native,
                ExternalDisplaySettings::Preference(v) => {
                    DbExternalDisplaySettings::Preference(DbModePreference {
                        resolution: v.resolution.into(),
                        aspect_ratio: v.aspect_ratio.into(),
                        refresh: v.refresh.into(),
                    })
                }
            }
        }
    }

    impl From<DbExternalDisplaySettings> for ExternalDisplaySettings {
        fn from(value: DbExternalDisplaySettings) -> Self {
            match value {
                DbExternalDisplaySettings::Previous => ExternalDisplaySettings::Previous,
                DbExternalDisplaySettings::Native => ExternalDisplaySettings::Native,
                DbExternalDisplaySettings::Preference(v) => {
                    ExternalDisplaySettings::Preference(ModePreference {
                        resolution: v.resolution.into(),
                        aspect_ratio: v.aspect_ratio.into(),
                        refresh: v.refresh.into(),
                    })
                }
            }
        }
    }

    impl From<RelativeLocation> for DbRelativeLocation {
        fn from(value: RelativeLocation) -> Self {
            match value {
                RelativeLocation::Above => DbRelativeLocation::Above,
                RelativeLocation::Below => DbRelativeLocation::Below,
                RelativeLocation::LeftOf => DbRelativeLocation::LeftOf,
                RelativeLocation::RightOf => DbRelativeLocation::RightOf,
                RelativeLocation::SameAs => DbRelativeLocation::SameAs,
            }
        }
    }

    impl From<DbRelativeLocation> for RelativeLocation {
        fn from(value: DbRelativeLocation) -> Self {
            match value {
                DbRelativeLocation::Above => RelativeLocation::Above,
                DbRelativeLocation::Below => RelativeLocation::Below,
                DbRelativeLocation::LeftOf => RelativeLocation::LeftOf,
                DbRelativeLocation::RightOf => RelativeLocation::RightOf,
                DbRelativeLocation::SameAs => RelativeLocation::SameAs,
            }
        }
    }

    impl<T, R> From<ModeOption<T>> for DbModeOption<R>
    where
        R: From<T>,
    {
        fn from(value: ModeOption<T>) -> Self {
            match value {
                ModeOption::Exact(v) => DbModeOption::Exact(v.into()),
                ModeOption::AtLeast(v) => DbModeOption::AtLeast(v.into()),
                ModeOption::AtMost(v) => DbModeOption::AtMost(v.into()),
            }
        }
    }

    impl From<Resolution> for DbResolution {
        fn from(value: Resolution) -> Self {
            Self {
                w: value.w,
                h: value.h,
            }
        }
    }

    impl From<AspectRatioOption> for DbAspectRatioOption {
        fn from(value: AspectRatioOption) -> Self {
            match value {
                AspectRatioOption::Any => DbAspectRatioOption::Any,
                AspectRatioOption::Native => DbAspectRatioOption::Native,
                AspectRatioOption::Exact(v) => DbAspectRatioOption::Exact(v),
            }
        }
    }

    impl From<DbDesktopSessionHandler> for DesktopSessionHandler {
        fn from(value: DbDesktopSessionHandler) -> Self {
            Self {
                id: value.id,
                teardown_external_settings: match value.teardown_external_settings {
                    DbExternalDisplaySettings::Previous => ExternalDisplaySettings::Previous,
                    DbExternalDisplaySettings::Native => ExternalDisplaySettings::Native,
                    DbExternalDisplaySettings::Preference(v) => {
                        ExternalDisplaySettings::Preference(ModePreference {
                            resolution: v.resolution.into(),
                            aspect_ratio: v.aspect_ratio.into(),
                            refresh: v.refresh.into(),
                        })
                    }
                },
                teardown_deck_location: value.teardown_deck_location.map(|v| match v {
                    DbRelativeLocation::Above => RelativeLocation::Above,
                    DbRelativeLocation::Below => RelativeLocation::Below,
                    DbRelativeLocation::LeftOf => RelativeLocation::LeftOf,
                    DbRelativeLocation::RightOf => RelativeLocation::RightOf,
                    DbRelativeLocation::SameAs => RelativeLocation::SameAs,
                }),
            }
        }
    }

    impl<T, R> From<DbModeOption<T>> for ModeOption<R>
    where
        R: From<T>,
    {
        fn from(value: DbModeOption<T>) -> Self {
            match value {
                DbModeOption::Exact(v) => ModeOption::Exact(v.into()),
                DbModeOption::AtLeast(v) => ModeOption::AtLeast(v.into()),
                DbModeOption::AtMost(v) => ModeOption::AtMost(v.into()),
            }
        }
    }

    impl From<DbResolution> for Resolution {
        fn from(value: DbResolution) -> Self {
            Self {
                w: value.w,
                h: value.h,
            }
        }
    }

    impl From<DbAspectRatioOption> for AspectRatioOption {
        fn from(value: DbAspectRatioOption) -> Self {
            match value {
                DbAspectRatioOption::Any => AspectRatioOption::Any,
                DbAspectRatioOption::Native => AspectRatioOption::Native,
                DbAspectRatioOption::Exact(v) => AspectRatioOption::Exact(v),
            }
        }
    }

    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    pub enum DbRelativeLocation {
        Above,
        #[default]
        Below,
        LeftOf,
        RightOf,
        SameAs,
    }

    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    #[serde(tag = "type", content = "value")]
    pub enum DbExternalDisplaySettings {
        /// Previous resolution, before setup
        #[default]
        Previous,
        /// Native resolution
        Native,
        /// Resolution based on specific settings
        Preference(DbModePreference),
    }

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct DbModePreference {
        pub resolution: DbModeOption<DbResolution>,
        pub aspect_ratio: DbAspectRatioOption,
        pub refresh: DbModeOption<f64>,
    }

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub enum DbModeOption<T> {
        Exact(T),
        AtLeast(T),
        AtMost(T),
    }

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct DbResolution {
        pub w: u32, // TODO::enforce w is multiple of 8 for CVT
        pub h: u32,
    }

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub enum DbAspectRatioOption {
        Any,
        Native,
        Exact(f32),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[native_db]
    #[native_model(id = 8, version = 1, with = NativeModelJSON)]
    pub struct DbMultiWindow {
        #[primary_key]
        pub id: ActionId,
    }

    impl From<MultiWindow> for DbMultiWindow {
        fn from(value: MultiWindow) -> Self {
            Self { id: value.id }
        }
    }

    impl From<DbMultiWindow> for MultiWindow {
        fn from(value: DbMultiWindow) -> Self {
            Self { id: value.id }
        }
    }

    #[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize)]
    #[native_db]
    #[native_model(id = 9, version = 1, with = NativeModelJSON)]
    pub struct DbSourceFile {
        #[primary_key]
        pub id: ActionId,
        pub source: DbFileSource,
    }

    impl From<SourceFile> for DbSourceFile {
        fn from(value: SourceFile) -> Self {
            Self {
                id: value.id,
                source: match value.source {
                    FileSource::Flatpak(v) => DbFileSource::Flatpak(match v {
                        FlatpakSource::Cemu => DbFlatpakSource::Cemu,
                        FlatpakSource::Citra => DbFlatpakSource::Citra,
                        FlatpakSource::MelonDS => DbFlatpakSource::MelonDS,
                    }),
                    FileSource::AppImage(v) => DbFileSource::AppImage(match v {
                        AppImageSource::Cemu => DbAppImageSource::Cemu,
                    }),
                    FileSource::EmuDeck(v) => DbFileSource::EmuDeck(match v {
                        EmuDeckSource::CemuProton => DbEmuDeckSource::CemuProton,
                    }),
                    FileSource::Custom(v) => DbFileSource::Custom(DbCustomFileOptions {
                        valid_ext: v.valid_ext,
                        path: v.path,
                    }),
                },
            }
        }
    }

    impl From<DbSourceFile> for SourceFile {
        fn from(value: DbSourceFile) -> Self {
            Self {
                id: value.id,
                source: match value.source {
                    DbFileSource::Flatpak(v) => FileSource::Flatpak(match v {
                        DbFlatpakSource::Cemu => FlatpakSource::Cemu,
                        DbFlatpakSource::Citra => FlatpakSource::Citra,
                        DbFlatpakSource::MelonDS => FlatpakSource::MelonDS,
                    }),
                    DbFileSource::AppImage(v) => FileSource::AppImage(match v {
                        DbAppImageSource::Cemu => AppImageSource::Cemu,
                    }),
                    DbFileSource::EmuDeck(v) => FileSource::EmuDeck(match v {
                        DbEmuDeckSource::CemuProton => EmuDeckSource::CemuProton,
                    }),
                    DbFileSource::Custom(v) => FileSource::Custom(CustomFileOptions {
                        valid_ext: v.valid_ext,
                        path: v.path,
                    }),
                },
            }
        }
    }

    #[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize)]
    #[serde(tag = "type", content = "value")]
    pub enum DbFileSource {
        Flatpak(DbFlatpakSource),
        AppImage(DbAppImageSource),
        EmuDeck(DbEmuDeckSource),
        Custom(DbCustomFileOptions),
    }

    #[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize)]
    pub struct DbCustomFileOptions {
        /// valid file extensions for source file
        pub valid_ext: Vec<String>,
        /// user defined custom path
        pub path: Option<PathBuf>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum DbFlatpakSource {
        Cemu,
        Citra,
        MelonDS,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum DbEmuDeckSource {
        CemuProton,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum DbAppImageSource {
        Cemu,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[native_db]
    #[native_model(id = 10, version = 1, with = NativeModelJSON)]
    pub struct DbVirtualScreen {
        #[primary_key]
        pub id: ActionId,
    }

    impl From<VirtualScreen> for DbVirtualScreen {
        fn from(value: VirtualScreen) -> Self {
            Self { id: value.id }
        }
    }

    impl From<DbVirtualScreen> for VirtualScreen {
        fn from(value: DbVirtualScreen) -> Self {
            Self { id: value.id }
        }
    }

    #[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
    #[native_db]
    #[native_model(id = 11, version = 1, with = NativeModelJSON)]
    pub struct DbDisplayConfig {
        #[primary_key]
        pub id: ActionId,
        pub external_display_settings: DbExternalDisplaySettings,
        pub deck_location: Option<DbRelativeLocation>,
        pub disable_splash: bool,
    }

    impl From<DisplayConfig> for DbDisplayConfig {
        fn from(value: DisplayConfig) -> Self {
            Self {
                id: value.id,
                external_display_settings: value.external_display_settings.into(),
                deck_location: value.deck_location.map(|v| v.into()),
                disable_splash: value.disable_splash,
            }
        }
    }

    impl From<DbDisplayConfig> for DisplayConfig {
        fn from(value: DbDisplayConfig) -> Self {
            Self {
                id: value.id,
                external_display_settings: value.external_display_settings.into(),
                deck_location: value.deck_location.map(|v| v.into()),
                disable_splash: value.disable_splash,
            }
        }
    }
}

impl DbAction {
    pub fn get_id(&self) -> ActionId {
        match *self {
            v1::DbAction::DesktopSessionHandler(id) => id,
            v1::DbAction::DisplayConfig(id) => id,
            v1::DbAction::VirtualScreen(id) => id,
            v1::DbAction::MultiWindow(id) => id,
            v1::DbAction::CitraLayout(id) => id,
            v1::DbAction::CemuLayout(id) => id,
            v1::DbAction::MelonDSLayout(id) => id,
            v1::DbAction::SourceFile(id) => id,
        }
    }
}
