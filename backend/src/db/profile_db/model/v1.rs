use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

use std::path::PathBuf;

use crate::db::common::codec::rmp_serde_1_3::{RmpSerde, RmpSerdeNamed};

use crate::db::common::model::DbExternalDisplaySettings;
use crate::{
    config::{AppId, ProfileId},
    pipeline::{
        action::{
            cemu_audio::{CemuAudio, CemuAudioChannels, CemuAudioSetting, CemuAudioState},
            cemu_layout::{CemuLayout, CemuLayoutState},
            citra_layout::{CitraLayout, CitraLayoutOption, CitraLayoutState},
            desktop_controller_layout_hack::DesktopControllerLayoutHack,
            display_config::DisplayConfig,
            lime_3ds_layout::Lime3dsLayout,
            melonds_layout::{MelonDSLayout, MelonDSLayoutOption, MelonDSSizingOption},
            multi_window::{
                main_app_automatic_windowing::{
                    GamescopeFilter, GamescopeFullscreenOption, GamescopeOptions, GamescopeScaler,
                    MainAppAutomaticWindowing,
                },
                primary_windowing::{
                    CemuWindowOptions, CitraWindowOptions, CustomWindowOptions,
                    DolphinWindowOptions, GeneralOptions,
                },
                secondary_app::{
                    LaunchSecondaryAppPreset, LaunchSecondaryFlatpakApp,
                    SecondaryAppScreenPreference, SecondaryAppWindowingBehavior,
                },
            },
            session_handler::DesktopSessionHandler,
            touch_config::TouchConfig,
            ActionId,
        },
        data::{PipelineActionId, PipelineDefinitionId, PipelineTarget, TopLevelId},
    },
    secondary_app::{FlatpakApp, SecondaryApp, SecondaryAppPresetId},
    sys::x_display::x_touch::TouchSelectionMode,
};

// Core

use crate::{
    pipeline::action::{
        emu_source::{
            AppImageSource, CustomEmuSource, EmuDeckSource, EmuSettingsSource,
            EmuSettingsSourceConfig, FlatpakSource,
        },
        multi_window::primary_windowing::{
            LimitedMultiWindowLayout, MultiWindow, MultiWindowLayout,
        },
        virtual_screen::VirtualScreen,
    },
    sys::x_display::Resolution,
};

#[derive(Serialize, Deserialize)]
#[native_db]
#[native_model(id = 1, version = 1, with = RmpSerde)]
pub struct DbCategoryProfile {
    #[primary_key]
    pub id: ProfileId,
    pub tags: Vec<String>,
    pub pipeline: DbPipelineDefinition,
}

#[derive(Serialize, Deserialize)]
#[native_db]
#[native_model(id = 2, version = 1, with = RmpSerde)]
pub struct DbAppSettings {
    #[primary_key]
    pub app_id: AppId,
    pub default_profile: Option<ProfileId>,
}

#[derive(Serialize, Deserialize)]
#[native_db]
#[native_model(id = 3, version = 1, with = RmpSerde)]
pub struct DbAppOverride {
    #[primary_key]
    pub id: (AppId, ProfileId),
    pub pipeline: DbPipelineDefinition,
}

#[derive(Serialize, Deserialize)]
#[native_db]
#[native_model(id = 4, version = 1, with = RmpSerde)]
pub struct DbPipelineDefinition {
    #[primary_key]
    pub id: PipelineDefinitionId,
    pub name: String,
    // pub should_register_exit_hooks: bool,
    pub primary_target_override: Option<PipelineTarget>,
    pub platform: DbTopLevelDefinition,
    pub toplevel: Vec<DbTopLevelDefinition>,
    pub desktop_controller_layout_hack: DbDesktopControllerLayoutHack,
}

// #[derive(Clone, Serialize, Deserialize)]
// pub struct DbBtnChord(u32, DbPressType);

// #[derive(Clone, Copy, Serialize, Deserialize)]
// enum DbPressType {
//     Short,
//     Long,
// }

// impl From<BtnChord> for DbBtnChord {
//     fn from(value: BtnChord) -> Self {
//         Self(
//             value.btns.bits(),
//             match value.press {
//                 PressType::Long => DbPressType::Long,
//                 PressType::Short => DbPressType::Short,
//             },
//         )
//     }
// }

// impl From<DbBtnChord> for BtnChord {
//     fn from(value: DbBtnChord) -> Self {
//         Self::new(
//             SteamDeckGamepadButton::from_bits_retain(value.0),
//             match value.1 {
//                 DbPressType::Short => PressType::Short,
//                 DbPressType::Long => PressType::Long,
//             },
//         )
//     }
// }

#[derive(Serialize, Deserialize)]
pub struct DbTopLevelDefinition {
    pub id: TopLevelId,
    pub root: PipelineActionId,
    pub actions: Vec<PipelineActionId>,
}

#[derive(Debug, Serialize, Deserialize)]
#[native_db]
#[native_model(id = 5, version = 1, with = RmpSerde)]
pub struct DbPipelineActionSettings {
    #[primary_key]
    pub id: (PipelineDefinitionId, TopLevelId, PipelineActionId),
    pub enabled: Option<bool>,
    pub is_visible_on_qam: bool,
    pub profile_override: Option<ProfileId>,
    pub selection: DbConfigSelection,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DbConfigSelection {
    Action(DbAction),
    OneOf { selection: PipelineActionId },
    AllOf,
    Versioned,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DbAction {
    pub id: ActionId,
    pub dtype: String, // ActionType as string, to avoid needing to update the join when actions are added
}

// Actions

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[native_db]
#[native_model(id = 1001, version = 1, with = RmpSerde)]
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
#[native_model(id = 1002, version = 1, with = RmpSerdeNamed)]
pub struct DbCitraLayout {
    #[primary_key]
    pub id: ActionId,
    pub layout_option: DbCitraLayoutOption,
    pub swap_screens: bool,
    pub fullscreen: bool,
    pub rotate_upright: bool,
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
            rotate_upright: value.layout.rotate_upright,
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
                rotate_upright: value.rotate_upright,
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
#[native_model(id = 1003, version = 1, with = RmpSerde)]
pub struct DbMelonDSLayout {
    #[primary_key]
    pub id: ActionId,
    pub layout_option: DbMelonDSLayoutOption,
    pub sizing_option: DbMelonDSSizingOption,
    pub book_mode: bool, // if in book mode, set rotation to 270,
    pub swap_screens: bool,
    pub window_index: Option<u8>,
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
            window_index: value.window_index,
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
            window_index: value.window_index,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum DbMelonDSLayoutOption {
    Natural,    // Puts screens vertical normally, horizonal in book mode.
    Vertical,   // Puts screens vertical always,
    Horizontal, // Puts screens horizonal always,
    Hybrid,     // Puts main screen large, with both screens adjacent. Overrides sizing settings.
    Single,     // Displays only one screen,
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
#[native_model(id = 1004, version = 1, with = RmpSerdeNamed)]
pub struct DbDesktopSessionHandler {
    #[primary_key]
    pub id: ActionId,
}

impl From<DesktopSessionHandler> for DbDesktopSessionHandler {
    fn from(value: DesktopSessionHandler) -> Self {
        Self { id: value.id }
    }
}

impl From<DbDesktopSessionHandler> for DesktopSessionHandler {
    fn from(value: DbDesktopSessionHandler) -> Self {
        Self { id: value.id }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[native_db]
#[native_model(id = 1005, version = 1, with = RmpSerde)]
pub struct DbMultiWindow {
    #[primary_key]
    pub id: ActionId,
    pub general: DbMultiWindowGeneralOptions,
    pub cemu: Option<DbMultiWindowCemuOptions>,
    pub citra: Option<DbMultiWindowCitraOptions>,
    pub dolphin: Option<DbMultiWindowDolphinOptions>,
    pub custom: Option<DbMultiWindowCustomOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbMultiWindowGeneralOptions {
    keep_above: bool,
    swap_screens: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbMultiWindowCemuOptions {
    single_screen_layout: DbLimitedMultiWindowLayout,
    multi_screen_layout: DbMultiWindowLayout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbMultiWindowCitraOptions {
    single_screen_layout: DbLimitedMultiWindowLayout,
    multi_screen_layout: DbMultiWindowLayout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbMultiWindowDolphinOptions {
    single_screen_layout: DbLimitedMultiWindowLayout,
    multi_screen_single_secondary_layout: DbMultiWindowLayout,
    multi_screen_multi_secondary_layout: DbMultiWindowLayout,
    gba_blacklist: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbMultiWindowCustomOptions {
    pub primary_window_matcher: Option<String>,
    pub secondary_window_matcher: Option<String>,
    pub classes: Vec<String>,
    pub single_screen_layout: DbLimitedMultiWindowLayout,
    pub multi_screen_single_secondary_layout: DbMultiWindowLayout,
    pub multi_screen_multi_secondary_layout: DbMultiWindowLayout,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum DbLimitedMultiWindowLayout {
    ColumnRight,
    ColumnLeft,
    SquareRight,
    SquareLeft,
}

impl From<LimitedMultiWindowLayout> for DbLimitedMultiWindowLayout {
    fn from(value: LimitedMultiWindowLayout) -> Self {
        match value {
            LimitedMultiWindowLayout::ColumnRight => DbLimitedMultiWindowLayout::ColumnRight,
            LimitedMultiWindowLayout::ColumnLeft => DbLimitedMultiWindowLayout::ColumnLeft,
            LimitedMultiWindowLayout::SquareRight => DbLimitedMultiWindowLayout::SquareRight,
            LimitedMultiWindowLayout::SquareLeft => DbLimitedMultiWindowLayout::SquareLeft,
        }
    }
}

impl From<DbLimitedMultiWindowLayout> for LimitedMultiWindowLayout {
    fn from(value: DbLimitedMultiWindowLayout) -> Self {
        match value {
            DbLimitedMultiWindowLayout::ColumnRight => LimitedMultiWindowLayout::ColumnRight,
            DbLimitedMultiWindowLayout::ColumnLeft => LimitedMultiWindowLayout::ColumnLeft,
            DbLimitedMultiWindowLayout::SquareRight => LimitedMultiWindowLayout::SquareRight,
            DbLimitedMultiWindowLayout::SquareLeft => LimitedMultiWindowLayout::SquareLeft,
        }
    }
}

impl From<MultiWindowLayout> for DbMultiWindowLayout {
    fn from(value: MultiWindowLayout) -> Self {
        match value {
            MultiWindowLayout::ColumnRight => DbMultiWindowLayout::ColumnRight,
            MultiWindowLayout::ColumnLeft => DbMultiWindowLayout::ColumnLeft,
            MultiWindowLayout::SquareRight => DbMultiWindowLayout::SquareRight,
            MultiWindowLayout::SquareLeft => DbMultiWindowLayout::SquareLeft,
            MultiWindowLayout::Separate => DbMultiWindowLayout::Separate,
        }
    }
}

impl From<DbMultiWindowLayout> for MultiWindowLayout {
    fn from(value: DbMultiWindowLayout) -> Self {
        match value {
            DbMultiWindowLayout::ColumnRight => MultiWindowLayout::ColumnRight,
            DbMultiWindowLayout::ColumnLeft => MultiWindowLayout::ColumnLeft,
            DbMultiWindowLayout::SquareRight => MultiWindowLayout::SquareRight,
            DbMultiWindowLayout::SquareLeft => MultiWindowLayout::SquareLeft,
            DbMultiWindowLayout::Separate => MultiWindowLayout::Separate,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DbMultiWindowLayout {
    ColumnRight,
    ColumnLeft,
    SquareRight,
    SquareLeft,
    Separate,
}

impl From<MultiWindow> for DbMultiWindow {
    fn from(value: MultiWindow) -> Self {
        Self {
            id: value.id,
            general: value.general.into(),
            cemu: value.cemu.map(|v| DbMultiWindowCemuOptions {
                single_screen_layout: v.single_screen_layout.into(),
                multi_screen_layout: v.multi_screen_layout.into(),
            }),
            citra: value.citra.map(|v| DbMultiWindowCitraOptions {
                single_screen_layout: v.single_screen_layout.into(),
                multi_screen_layout: v.multi_screen_layout.into(),
            }),
            dolphin: value.dolphin.map(|v| DbMultiWindowDolphinOptions {
                single_screen_layout: v.single_screen_layout.into(),
                multi_screen_single_secondary_layout: v.multi_screen_single_secondary_layout.into(),
                multi_screen_multi_secondary_layout: v.multi_screen_multi_secondary_layout.into(),
                gba_blacklist: v.gba_blacklist,
            }),
            custom: value.custom.map(|v| DbMultiWindowCustomOptions {
                primary_window_matcher: v.primary_window_matcher.clone(),
                secondary_window_matcher: v.secondary_window_matcher.clone(),
                classes: v.classes.clone(),
                single_screen_layout: v.single_screen_layout.into(),
                multi_screen_single_secondary_layout: v.multi_screen_single_secondary_layout.into(),
                multi_screen_multi_secondary_layout: v.multi_screen_multi_secondary_layout.into(),
            }),
        }
    }
}

impl From<DbMultiWindow> for MultiWindow {
    fn from(value: DbMultiWindow) -> Self {
        Self {
            id: value.id,
            general: value.general.into(),
            cemu: value.cemu.map(|v| CemuWindowOptions {
                single_screen_layout: v.single_screen_layout.into(),
                multi_screen_layout: v.multi_screen_layout.into(),
            }),
            citra: value.citra.map(|v| CitraWindowOptions {
                single_screen_layout: v.single_screen_layout.into(),
                multi_screen_layout: v.multi_screen_layout.into(),
            }),
            dolphin: value.dolphin.map(|v| DolphinWindowOptions {
                single_screen_layout: v.single_screen_layout.into(),
                multi_screen_single_secondary_layout: v.multi_screen_single_secondary_layout.into(),
                multi_screen_multi_secondary_layout: v.multi_screen_multi_secondary_layout.into(),
                gba_blacklist: v.gba_blacklist,
            }),
            custom: value.custom.map(|v| CustomWindowOptions {
                primary_window_matcher: v.primary_window_matcher.clone(),
                secondary_window_matcher: v.secondary_window_matcher.clone(),
                classes: v.classes.clone(),
                single_screen_layout: v.single_screen_layout.into(),
                multi_screen_single_secondary_layout: v.multi_screen_single_secondary_layout.into(),
                multi_screen_multi_secondary_layout: v.multi_screen_multi_secondary_layout.into(),
            }),
        }
    }
}

impl From<GeneralOptions> for DbMultiWindowGeneralOptions {
    fn from(value: GeneralOptions) -> Self {
        Self {
            keep_above: value.keep_above,
            swap_screens: value.swap_screens,
        }
    }
}

impl From<DbMultiWindowGeneralOptions> for GeneralOptions {
    fn from(value: DbMultiWindowGeneralOptions) -> Self {
        Self {
            keep_above: value.keep_above,
            swap_screens: value.swap_screens,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize)]
#[native_db]
#[native_model(id = 1006, version = 1, with = RmpSerde)]
pub struct DbSourceFile {
    #[primary_key]
    pub id: ActionId,
    pub source: DbFileSource,
}

impl From<EmuSettingsSourceConfig> for DbSourceFile {
    fn from(value: EmuSettingsSourceConfig) -> Self {
        Self {
            id: value.id,
            source: match value.source {
                EmuSettingsSource::Flatpak(v) => DbFileSource::Flatpak(match v {
                    FlatpakSource::Cemu => DbFlatpakSource::Cemu,
                    FlatpakSource::Citra => DbFlatpakSource::Citra,
                    FlatpakSource::MelonDS => DbFlatpakSource::MelonDS,
                    FlatpakSource::MelonDSPrerelease => DbFlatpakSource::MelonDSPrerelease,
                    FlatpakSource::Lime3ds => DbFlatpakSource::Lime3ds,
                }),
                EmuSettingsSource::AppImage(v) => DbFileSource::AppImage(match v {
                    AppImageSource::Cemu => DbAppImageSource::Cemu,
                }),
                EmuSettingsSource::EmuDeck(v) => DbFileSource::EmuDeck(match v {
                    EmuDeckSource::CemuProton => DbEmuDeckSource::CemuProton,
                }),
                EmuSettingsSource::Custom(v) => DbFileSource::Custom(DbCustomEmuSource {
                    valid_ext: v.valid_ext,
                    settings_path: v.settings_path,
                }),
            },
        }
    }
}

impl From<DbSourceFile> for EmuSettingsSourceConfig {
    fn from(value: DbSourceFile) -> Self {
        Self {
            id: value.id,
            source: match value.source {
                DbFileSource::Flatpak(v) => EmuSettingsSource::Flatpak(match v {
                    DbFlatpakSource::Cemu => FlatpakSource::Cemu,
                    DbFlatpakSource::Citra => FlatpakSource::Citra,
                    DbFlatpakSource::MelonDSPrerelease => FlatpakSource::MelonDS,
                    DbFlatpakSource::MelonDS => FlatpakSource::MelonDS,
                    DbFlatpakSource::Lime3ds => FlatpakSource::Lime3ds,
                }),
                DbFileSource::AppImage(v) => EmuSettingsSource::AppImage(match v {
                    DbAppImageSource::Cemu => AppImageSource::Cemu,
                }),
                DbFileSource::EmuDeck(v) => EmuSettingsSource::EmuDeck(match v {
                    DbEmuDeckSource::CemuProton => EmuDeckSource::CemuProton,
                }),
                DbFileSource::Custom(v) => EmuSettingsSource::Custom(CustomEmuSource {
                    valid_ext: v.valid_ext,
                    settings_path: v.settings_path,
                    // emu_cmd: v.emu_cmd,
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
    Custom(DbCustomEmuSource),
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize)]
pub struct DbCustomEmuSource {
    /// valid file extensions for source file
    pub valid_ext: Vec<String>,
    /// user defined custom path
    pub settings_path: Option<PathBuf>,
    // pub emu_cmd: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DbFlatpakSource {
    Cemu,
    Citra,
    MelonDSPrerelease,
    MelonDS,
    Lime3ds,
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
#[native_model(id = 1007, version = 1, with = RmpSerde)]
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
#[native_model(id = 1008, version = 1, with = RmpSerdeNamed)]
pub struct DbDisplayConfig {
    #[primary_key]
    pub id: ActionId,
    pub external_display_settings: Option<DbExternalDisplaySettings>,
    pub deck_is_enabled: Option<bool>,
}

impl From<DisplayConfig> for DbDisplayConfig {
    fn from(value: DisplayConfig) -> Self {
        Self {
            id: value.id,
            deck_is_enabled: value.deck_is_enabled,
            external_display_settings: value.external_display_settings.map(|v| v.into()),
        }
    }
}

impl From<DbDisplayConfig> for DisplayConfig {
    fn from(value: DbDisplayConfig) -> Self {
        Self {
            id: value.id,
            deck_is_enabled: value.deck_is_enabled,
            external_display_settings: value.external_display_settings.map(|v| v.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[native_db]
#[native_model(id = 1009, version = 1, with = RmpSerde)]
pub struct DbLaunchSecondaryFlatpakApp {
    #[primary_key]
    pub id: ActionId,
    pub app: DbSecondaryFlatpakApp,
    pub windowing_behavior: DbSecondaryAppWindowingBehavior,
    pub screen_preference: DbSecondaryAppScreenPreference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DbSecondaryApp {
    Flatpak(DbSecondaryFlatpakApp),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbSecondaryFlatpakApp {
    app_id: String,
    args: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DbSecondaryAppWindowingBehavior {
    Fullscreen,
    Maximized,
    Minimized,
    Unmanaged,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DbSecondaryAppScreenPreference {
    PreferSecondary,
    PreferPrimary,
}

impl From<FlatpakApp> for DbSecondaryFlatpakApp {
    fn from(value: FlatpakApp) -> Self {
        Self {
            app_id: value.app_id,
            args: value.args,
        }
    }
}

impl From<DbSecondaryFlatpakApp> for FlatpakApp {
    fn from(value: DbSecondaryFlatpakApp) -> Self {
        Self {
            app_id: value.app_id,
            args: value.args,
        }
    }
}

impl From<LaunchSecondaryFlatpakApp> for DbLaunchSecondaryFlatpakApp {
    fn from(value: LaunchSecondaryFlatpakApp) -> Self {
        Self {
            id: value.id,
            app: value.app.into(),
            windowing_behavior: value.windowing_behavior.into(),
            screen_preference: value.screen_preference.into(),
        }
    }
}

impl From<DbLaunchSecondaryFlatpakApp> for LaunchSecondaryFlatpakApp {
    fn from(value: DbLaunchSecondaryFlatpakApp) -> Self {
        Self {
            id: value.id,
            app: value.app.into(),
            windowing_behavior: value.windowing_behavior.into(),
            screen_preference: value.screen_preference.into(),
        }
    }
}

impl From<SecondaryApp> for DbSecondaryApp {
    fn from(value: SecondaryApp) -> Self {
        match value {
            SecondaryApp::Flatpak(app) => DbSecondaryApp::Flatpak(app.into()),
        }
    }
}

impl From<DbSecondaryApp> for SecondaryApp {
    fn from(value: DbSecondaryApp) -> Self {
        match value {
            DbSecondaryApp::Flatpak(app) => SecondaryApp::Flatpak(app.into()),
        }
    }
}

impl From<SecondaryAppWindowingBehavior> for DbSecondaryAppWindowingBehavior {
    fn from(value: SecondaryAppWindowingBehavior) -> Self {
        match value {
            SecondaryAppWindowingBehavior::Fullscreen => {
                DbSecondaryAppWindowingBehavior::Fullscreen
            }
            SecondaryAppWindowingBehavior::Maximized => DbSecondaryAppWindowingBehavior::Maximized,
            SecondaryAppWindowingBehavior::Minimized => DbSecondaryAppWindowingBehavior::Minimized,
            SecondaryAppWindowingBehavior::Unmanaged => DbSecondaryAppWindowingBehavior::Unmanaged,
        }
    }
}

impl From<DbSecondaryAppWindowingBehavior> for SecondaryAppWindowingBehavior {
    fn from(value: DbSecondaryAppWindowingBehavior) -> Self {
        match value {
            DbSecondaryAppWindowingBehavior::Fullscreen => {
                SecondaryAppWindowingBehavior::Fullscreen
            }
            DbSecondaryAppWindowingBehavior::Maximized => SecondaryAppWindowingBehavior::Maximized,
            DbSecondaryAppWindowingBehavior::Minimized => SecondaryAppWindowingBehavior::Minimized,
            DbSecondaryAppWindowingBehavior::Unmanaged => SecondaryAppWindowingBehavior::Unmanaged,
        }
    }
}

impl From<SecondaryAppScreenPreference> for DbSecondaryAppScreenPreference {
    fn from(value: SecondaryAppScreenPreference) -> Self {
        match value {
            SecondaryAppScreenPreference::PreferSecondary => {
                DbSecondaryAppScreenPreference::PreferSecondary
            }
            SecondaryAppScreenPreference::PreferPrimary => {
                DbSecondaryAppScreenPreference::PreferPrimary
            }
        }
    }
}

impl From<DbSecondaryAppScreenPreference> for SecondaryAppScreenPreference {
    fn from(value: DbSecondaryAppScreenPreference) -> Self {
        match value {
            DbSecondaryAppScreenPreference::PreferSecondary => {
                SecondaryAppScreenPreference::PreferSecondary
            }
            DbSecondaryAppScreenPreference::PreferPrimary => {
                SecondaryAppScreenPreference::PreferPrimary
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[native_db]
#[native_model(id = 1010, version = 1, with = RmpSerde)]
pub struct DbLaunchSecondaryAppPreset {
    #[primary_key]
    pub id: ActionId,
    pub preset: SecondaryAppPresetId,
    pub windowing_behavior: DbSecondaryAppWindowingBehavior,
    pub screen_preference: DbSecondaryAppScreenPreference,
}

impl From<LaunchSecondaryAppPreset> for DbLaunchSecondaryAppPreset {
    fn from(value: LaunchSecondaryAppPreset) -> Self {
        Self {
            id: value.id,
            preset: value.preset,
            windowing_behavior: value.windowing_behavior.into(),
            screen_preference: value.screen_preference.into(),
        }
    }
}

impl From<DbLaunchSecondaryAppPreset> for LaunchSecondaryAppPreset {
    fn from(value: DbLaunchSecondaryAppPreset) -> Self {
        Self {
            id: value.id,
            preset: value.preset,
            windowing_behavior: value.windowing_behavior.into(),
            screen_preference: value.screen_preference.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[native_db]
#[native_model(id = 1011, version = 1, with = RmpSerde)]
pub struct DbMainAppAutomaticWindowing {
    #[primary_key]
    id: ActionId,
    general: DbMultiWindowGeneralOptions,
    gamescope: DbGamescopeOptions,
}

impl From<MainAppAutomaticWindowing> for DbMainAppAutomaticWindowing {
    fn from(value: MainAppAutomaticWindowing) -> Self {
        Self {
            id: value.id,
            general: value.general.into(),
            gamescope: value.gamescope.into(),
        }
    }
}

impl From<DbMainAppAutomaticWindowing> for MainAppAutomaticWindowing {
    fn from(value: DbMainAppAutomaticWindowing) -> Self {
        Self {
            id: value.id,
            general: value.general.into(),
            gamescope: value.gamescope.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbGamescopeOptions {
    pub use_gamescope: bool,
    pub game_resolution: Option<Resolution>,
    pub game_refresh: Option<u16>,
    pub fullscreen_option: DbGamescopeFullscreenOption,
    pub scaler: DbGamescopeScaler,
    pub filter: DbGamescopeFilter,
    pub fsr_sharpness: u8,
    pub nis_sharpness: u8,
}

impl From<GamescopeOptions> for DbGamescopeOptions {
    fn from(value: GamescopeOptions) -> Self {
        Self {
            use_gamescope: value.use_gamescope,
            game_resolution: value.game_resolution,
            game_refresh: value.game_refresh,
            nis_sharpness: value.nis_sharpness,
            fsr_sharpness: value.fsr_sharpness,
            fullscreen_option: match value.fullscreen_option {
                GamescopeFullscreenOption::Borderless => DbGamescopeFullscreenOption::Borderless,
                GamescopeFullscreenOption::Fullscreen => DbGamescopeFullscreenOption::Fullscreen,
            },
            scaler: match value.scaler {
                GamescopeScaler::Auto => DbGamescopeScaler::Auto,
                GamescopeScaler::Integer => DbGamescopeScaler::Integer,
                GamescopeScaler::Fit => DbGamescopeScaler::Fit,
                GamescopeScaler::Fill => DbGamescopeScaler::Fill,
                GamescopeScaler::Stretch => DbGamescopeScaler::Stretch,
            },
            filter: match value.filter {
                GamescopeFilter::Linear => DbGamescopeFilter::Linear,
                GamescopeFilter::Fsr => DbGamescopeFilter::Fsr,
                GamescopeFilter::Nis => DbGamescopeFilter::Nis,
                GamescopeFilter::Pixel => DbGamescopeFilter::Pixel,
            },
        }
    }
}

impl From<DbGamescopeOptions> for GamescopeOptions {
    fn from(value: DbGamescopeOptions) -> Self {
        Self {
            use_gamescope: value.use_gamescope,
            game_resolution: value.game_resolution,
            game_refresh: value.game_refresh,
            nis_sharpness: value.nis_sharpness,
            fsr_sharpness: value.fsr_sharpness,
            fullscreen_option: match value.fullscreen_option {
                DbGamescopeFullscreenOption::Borderless => GamescopeFullscreenOption::Borderless,
                DbGamescopeFullscreenOption::Fullscreen => GamescopeFullscreenOption::Fullscreen,
            },
            scaler: match value.scaler {
                DbGamescopeScaler::Auto => GamescopeScaler::Auto,
                DbGamescopeScaler::Integer => GamescopeScaler::Integer,
                DbGamescopeScaler::Fit => GamescopeScaler::Fit,
                DbGamescopeScaler::Fill => GamescopeScaler::Fill,
                DbGamescopeScaler::Stretch => GamescopeScaler::Stretch,
            },
            filter: match value.filter {
                DbGamescopeFilter::Linear => GamescopeFilter::Linear,
                DbGamescopeFilter::Fsr => GamescopeFilter::Fsr,
                DbGamescopeFilter::Nis => GamescopeFilter::Nis,
                DbGamescopeFilter::Pixel => GamescopeFilter::Pixel,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub enum DbGamescopeFullscreenOption {
    Borderless,
    Fullscreen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DbGamescopeScaler {
    Auto,
    Integer,
    Fit,
    Fill,
    Stretch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DbGamescopeFilter {
    Linear,
    Fsr,
    Nis,
    Pixel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[native_db]
#[native_model(id = 1012, version = 1, with = RmpSerdeNamed)]
pub struct DbLime3dsLayout {
    #[primary_key]
    pub id: ActionId,
    pub layout_option: DbCitraLayoutOption,
    pub swap_screens: bool,
    pub fullscreen: bool,
    pub rotate_upright: bool,
}

impl From<Lime3dsLayout> for DbLime3dsLayout {
    fn from(value: Lime3dsLayout) -> Self {
        let value = value.0;

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
            rotate_upright: value.layout.rotate_upright,
        }
    }
}

impl From<DbLime3dsLayout> for Lime3dsLayout {
    fn from(value: DbLime3dsLayout) -> Self {
        Lime3dsLayout(CitraLayout {
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
                rotate_upright: value.rotate_upright,
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[native_db]
#[native_model(id = 1013, version = 1, with = RmpSerde)]
pub struct DbCemuAudio {
    #[primary_key]
    pub id: ActionId,
    pub state: DbCemuAudioState,
}

impl From<CemuAudio> for DbCemuAudio {
    fn from(value: CemuAudio) -> Self {
        Self {
            id: value.id,
            state: value.state.into(),
        }
    }
}

impl From<DbCemuAudio> for CemuAudio {
    fn from(value: DbCemuAudio) -> Self {
        Self {
            id: value.id,
            state: value.state.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DbCemuAudioState {
    pub tv_out: DbCemuAudioSetting,
    pub pad_out: DbCemuAudioSetting,
    pub mic_in: DbCemuAudioSetting,
}

impl From<CemuAudioState> for DbCemuAudioState {
    fn from(value: CemuAudioState) -> Self {
        Self {
            tv_out: value.tv_out.into(),
            pad_out: value.pad_out.into(),
            mic_in: value.mic_in.into(),
        }
    }
}

impl From<DbCemuAudioState> for CemuAudioState {
    fn from(value: DbCemuAudioState) -> Self {
        Self {
            tv_out: value.tv_out.into(),
            pad_out: value.pad_out.into(),
            mic_in: value.mic_in.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DbCemuAudioSetting {
    pub device: String,
    pub volume: u8,
    pub channels: DbCemuAudioChannels,
}

impl From<CemuAudioSetting> for DbCemuAudioSetting {
    fn from(value: CemuAudioSetting) -> Self {
        Self {
            device: value.device,
            volume: value.volume,
            channels: match value.channels {
                CemuAudioChannels::Mono => DbCemuAudioChannels::Mono,
                CemuAudioChannels::Stereo => DbCemuAudioChannels::Stereo,
                CemuAudioChannels::Surround => DbCemuAudioChannels::Surround,
            },
        }
    }
}

impl From<DbCemuAudioSetting> for CemuAudioSetting {
    fn from(value: DbCemuAudioSetting) -> Self {
        Self {
            device: value.device,
            volume: value.volume,
            channels: match value.channels {
                DbCemuAudioChannels::Mono => CemuAudioChannels::Mono,
                DbCemuAudioChannels::Stereo => CemuAudioChannels::Stereo,
                DbCemuAudioChannels::Surround => CemuAudioChannels::Surround,
            },
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum DbCemuAudioChannels {
    Mono,
    Stereo,
    Surround,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[native_db]
#[native_model(id = 1014, version = 1, with = RmpSerde)]
pub struct DbDesktopControllerLayoutHack {
    #[primary_key]
    pub id: ActionId,
    pub nonsteam_override: Option<bool>,
    pub steam_override: Option<bool>,
}

impl From<DesktopControllerLayoutHack> for DbDesktopControllerLayoutHack {
    fn from(value: DesktopControllerLayoutHack) -> Self {
        Self {
            id: value.id,
            nonsteam_override: value.nonsteam_override,
            steam_override: value.steam_override,
        }
    }
}

impl From<DbDesktopControllerLayoutHack> for DesktopControllerLayoutHack {
    fn from(value: DbDesktopControllerLayoutHack) -> Self {
        Self {
            id: value.id,
            nonsteam_override: value.nonsteam_override,
            steam_override: value.steam_override,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[native_db]
#[native_model(id = 1015, version = 1, with = RmpSerde)]
pub struct DbTouchConfig {
    #[primary_key]
    pub id: ActionId,
    pub touch_mode: DbTouchSelectionMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DbTouchSelectionMode {
    PerDisplay,
    PreferEmbedded,
    PreferExternal,
}

impl From<TouchConfig> for DbTouchConfig {
    fn from(value: TouchConfig) -> Self {
        Self {
            id: value.id,
            touch_mode: match value.touch_mode {
                TouchSelectionMode::PerDisplay => DbTouchSelectionMode::PerDisplay,
                TouchSelectionMode::PreferEmbedded => DbTouchSelectionMode::PreferEmbedded,
                TouchSelectionMode::PreferExternal => DbTouchSelectionMode::PreferExternal,
            },
        }
    }
}

impl From<DbTouchConfig> for TouchConfig {
    fn from(value: DbTouchConfig) -> Self {
        Self {
            id: value.id,
            touch_mode: match value.touch_mode {
                DbTouchSelectionMode::PerDisplay => TouchSelectionMode::PerDisplay,
                DbTouchSelectionMode::PreferEmbedded => TouchSelectionMode::PreferEmbedded,
                DbTouchSelectionMode::PreferExternal => TouchSelectionMode::PreferExternal,
            },
        }
    }
}
