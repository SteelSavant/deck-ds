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
            multi_window::{
                primary_windowing::{
                    CemuWindowOptions, CitraWindowOptions, CustomWindowOptions,
                    DolphinWindowOptions, GeneralOptions,
                },
                secondary_app::{
                    LaunchSecondaryApp, LaunchSecondaryAppPreset, SecondaryAppWindowingBehavior,
                },
            },
            session_handler::DesktopSessionHandler,
            ActionId,
        },
        data::{PipelineActionId, PipelineDefinitionId, PipelineTarget},
    },
    secondary_app::{FlatpakApp, SecondaryApp, SecondaryAppPresetId},
    settings::{AppId, ProfileId},
};

// Core

use crate::{
    pipeline::action::{
        multi_window::primary_windowing::{
            LimitedMultiWindowLayout, MultiWindow, MultiWindowLayout,
        },
        session_handler::{ExternalDisplaySettings, RelativeLocation},
        source_file::{
            AppImageSource, CustomFileOptions, EmuDeckSource, FileSource, FlatpakSource, SourceFile,
        },
        virtual_screen::VirtualScreen,
    },
    sys::x_display::{AspectRatioOption, ModeOption, ModePreference, Resolution},
};

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
pub struct DbAppSettings {
    #[primary_key]
    pub app_id: AppId,
    pub default_profile: Option<ProfileId>,
}

#[derive(Serialize, Deserialize)]
#[native_db]
#[native_model(id = 3, version = 1, with = NativeModelJSON)]
pub struct DbAppOverride {
    #[primary_key]
    pub id: (AppId, ProfileId),
    pub pipeline: DbPipelineDefinition,
}

#[derive(Serialize, Deserialize)]
#[native_db]
#[native_model(id = 4, version = 1, with = NativeModelJSON)]
pub struct DbPipelineDefinition {
    #[primary_key]
    pub id: PipelineDefinitionId,
    pub name: String,
    pub register_exit_hooks: bool,
    pub primary_target_override: Option<PipelineTarget>,
    pub platform: PipelineActionId,
    pub actions: Vec<PipelineActionId>,
}

#[derive(Serialize, Deserialize)]
#[native_db]
#[native_model(id = 5, version = 1, with = NativeModelJSON)]
pub struct DbPipelineActionSettings {
    #[primary_key]
    pub id: (PipelineDefinitionId, PipelineActionId),
    pub enabled: Option<bool>,
    pub is_visible_on_qam: bool,
    pub profile_override: Option<ProfileId>,
    pub selection: DbConfigSelection,
}

#[derive(Serialize, Deserialize)]
pub enum DbConfigSelection {
    Action(DbAction),
    OneOf { selection: PipelineActionId },
    AllOf,
    UserDefined(Vec<PipelineActionId>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DbAction {
    pub id: ActionId,
    pub dtype: String, // ActionType as string, to avoid needing to update the join when actions are added
}

// Actions

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[native_db]
#[native_model(id = 101, version = 1, with = NativeModelJSON)]
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
#[native_model(id = 102, version = 1, with = NativeModelJSON)]
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
#[native_model(id = 103, version = 1, with = NativeModelJSON)]
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
#[native_model(id = 104, version = 1, with = NativeModelJSON)]
pub struct DbDesktopSessionHandler {
    #[primary_key]
    pub id: ActionId,
    pub teardown_external_settings: DbExternalDisplaySettings,
    pub teardown_deck_location: Option<DbRelativeLocation>,
    pub deck_is_primary_display: bool,
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
            deck_is_primary_display: value.deck_is_primary_display,
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
            deck_is_primary_display: value.deck_is_primary_display,
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
#[native_model(id = 105, version = 1, with = NativeModelJSON)]
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
    pub primary_window_override: Option<String>,
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
            general: DbMultiWindowGeneralOptions {
                keep_above: value.general.keep_above,
                swap_screens: value.general.swap_screens,
            },
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
                primary_window_override: v.primary_window_override.clone(),
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
            general: GeneralOptions {
                keep_above: value.general.keep_above,
                swap_screens: value.general.swap_screens,
            },
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
                primary_window_override: v.primary_window_override.clone(),
                secondary_window_matcher: v.secondary_window_matcher.clone(),
                classes: v.classes.clone(),
                single_screen_layout: v.single_screen_layout.into(),
                multi_screen_single_secondary_layout: v.multi_screen_single_secondary_layout.into(),
                multi_screen_multi_secondary_layout: v.multi_screen_multi_secondary_layout.into(),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize)]
#[native_db]
#[native_model(id = 106, version = 1, with = NativeModelJSON)]
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
#[native_model(id = 107, version = 1, with = NativeModelJSON)]
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
#[native_model(id = 108, version = 1, with = NativeModelJSON)]
pub struct DbDisplayConfig {
    #[primary_key]
    pub id: ActionId,
    pub external_display_settings: DbExternalDisplaySettings,
    pub deck_location: Option<DbRelativeLocation>,
    pub deck_is_primary_display: bool,
}

impl From<DisplayConfig> for DbDisplayConfig {
    fn from(value: DisplayConfig) -> Self {
        Self {
            id: value.id,
            external_display_settings: value.external_display_settings.into(),
            deck_location: value.deck_location.map(std::convert::Into::into),
            deck_is_primary_display: value.deck_is_primary_display,
        }
    }
}

impl From<DbDisplayConfig> for DisplayConfig {
    fn from(value: DbDisplayConfig) -> Self {
        Self {
            id: value.id,
            external_display_settings: value.external_display_settings.into(),
            deck_location: value.deck_location.map(std::convert::Into::into),
            deck_is_primary_display: value.deck_is_primary_display,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[native_db]
#[native_model(id = 109, version = 1, with = NativeModelJSON)]
pub struct DbLaunchSecondaryApp {
    #[primary_key]
    pub id: ActionId,
    pub app: DbSecondaryApp,
    pub windowing_behavior: DbSecondaryAppWindowingBehavior,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DbSecondaryApp {
    Flatpak { app_id: String, args: Vec<String> },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DbSecondaryAppWindowingBehavior {
    PreferSecondary,
    PreferPrimary,
    Hidden,
    Unmanaged,
}

impl From<LaunchSecondaryApp> for DbLaunchSecondaryApp {
    fn from(value: LaunchSecondaryApp) -> Self {
        Self {
            id: value.id,
            app: value.app.into(),
            windowing_behavior: value.windowing_behavior.into(),
        }
    }
}

impl From<DbLaunchSecondaryApp> for LaunchSecondaryApp {
    fn from(value: DbLaunchSecondaryApp) -> Self {
        Self {
            id: value.id,
            app: value.app.into(),
            windowing_behavior: value.windowing_behavior.into(),
        }
    }
}

impl From<SecondaryApp> for DbSecondaryApp {
    fn from(value: SecondaryApp) -> Self {
        match value {
            SecondaryApp::Flatpak(app) => DbSecondaryApp::Flatpak {
                app_id: app.app_id,
                args: app.args,
            },
        }
    }
}

impl From<DbSecondaryApp> for SecondaryApp {
    fn from(value: DbSecondaryApp) -> Self {
        match value {
            DbSecondaryApp::Flatpak { app_id, args } => {
                SecondaryApp::Flatpak(FlatpakApp { app_id, args })
            }
        }
    }
}

impl From<SecondaryAppWindowingBehavior> for DbSecondaryAppWindowingBehavior {
    fn from(value: SecondaryAppWindowingBehavior) -> Self {
        match value {
            SecondaryAppWindowingBehavior::PreferSecondary => {
                DbSecondaryAppWindowingBehavior::PreferSecondary
            }
            SecondaryAppWindowingBehavior::PreferPrimary => {
                DbSecondaryAppWindowingBehavior::PreferPrimary
            }
            SecondaryAppWindowingBehavior::Hidden => DbSecondaryAppWindowingBehavior::Hidden,
            SecondaryAppWindowingBehavior::Unmanaged => DbSecondaryAppWindowingBehavior::Unmanaged,
        }
    }
}

impl From<DbSecondaryAppWindowingBehavior> for SecondaryAppWindowingBehavior {
    fn from(value: DbSecondaryAppWindowingBehavior) -> Self {
        match value {
            DbSecondaryAppWindowingBehavior::PreferSecondary => {
                SecondaryAppWindowingBehavior::PreferSecondary
            }
            DbSecondaryAppWindowingBehavior::PreferPrimary => {
                SecondaryAppWindowingBehavior::PreferPrimary
            }
            DbSecondaryAppWindowingBehavior::Hidden => SecondaryAppWindowingBehavior::Hidden,
            DbSecondaryAppWindowingBehavior::Unmanaged => SecondaryAppWindowingBehavior::Unmanaged,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[native_db]
#[native_model(id = 110, version = 1, with = NativeModelJSON)]
pub struct DbLaunchSecondaryAppPreset {
    #[primary_key]
    pub id: ActionId,
    pub preset: SecondaryAppPresetId,
    pub windowing_behavior: DbSecondaryAppWindowingBehavior,
}

impl From<LaunchSecondaryAppPreset> for DbLaunchSecondaryAppPreset {
    fn from(value: LaunchSecondaryAppPreset) -> Self {
        Self {
            id: value.id,
            preset: value.preset,
            windowing_behavior: value.windowing_behavior.into(),
        }
    }
}

impl From<DbLaunchSecondaryAppPreset> for LaunchSecondaryAppPreset {
    fn from(value: DbLaunchSecondaryAppPreset) -> Self {
        Self {
            id: value.id,
            preset: value.preset,
            windowing_behavior: value.windowing_behavior.into(),
        }
    }
}
