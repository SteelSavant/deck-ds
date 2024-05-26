/* eslint-disable */
/**
 * This file was automatically generated by json-schema-to-typescript.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run json-schema-to-typescript to regenerate this file.
 */

export type PipelineTarget = 'Desktop' | 'Gamemode';
/**
 * The required button chord to hold to exit. At least 2 buttons are required.
 *
 * @minItems 3
 * @maxItems 3
 */
export type ExitHooks = [GamepadButton, GamepadButton, GamepadButton[]];
export type GamepadButton =
    | 'Start'
    | 'Select'
    | 'North'
    | 'East'
    | 'South'
    | 'West'
    | 'RightThumb'
    | 'LeftThumb'
    | 'DPadUp'
    | 'DPadLeft'
    | 'DPadRight'
    | 'DPadDown'
    | 'L1'
    | 'L2'
    | 'R1'
    | 'R2';
/**
 * Configured selection for an specific pipeline. Only user values are saved; everything else is pulled at runtime to ensure it's up to date.
 */
export type ConfigSelection =
    | {
          type: 'Action';
          value: Action;
      }
    | {
          type: 'OneOf';
          value: {
              selection: string;
          };
      }
    | {
          type: 'AllOf';
      };
export type Action =
    | {
          type: 'DesktopSessionHandler';
          value: DesktopSessionHandler;
      }
    | {
          type: 'DisplayConfig';
          value: DisplayConfig;
      }
    | {
          type: 'VirtualScreen';
          value: VirtualScreen;
      }
    | {
          type: 'MultiWindow';
          value: MultiWindow;
      }
    | {
          type: 'CitraLayout';
          value: CitraLayout;
      }
    | {
          type: 'CemuLayout';
          value: CemuLayout;
      }
    | {
          type: 'CemuAudio';
          value: CemuAudio;
      }
    | {
          type: 'Lime3dsLayout';
          value: CitraLayout;
      }
    | {
          type: 'MelonDSLayout';
          value: MelonDSLayout;
      }
    | {
          type: 'SourceFile';
          value: SourceFile;
      }
    | {
          type: 'LaunchSecondaryFlatpakApp';
          value: LaunchSecondaryFlatpakApp;
      }
    | {
          type: 'LaunchSecondaryAppPreset';
          value: LaunchSecondaryAppPreset;
      }
    | {
          type: 'MainAppAutomaticWindowing';
          value: MainAppAutomaticWindowing;
      };
export type RelativeLocation =
    | 'Above'
    | 'Below'
    | 'LeftOf'
    | 'RightOf'
    | 'SameAs';
export type ExternalDisplaySettings =
    | {
          type: 'Previous';
      }
    | {
          type: 'Native';
      }
    | {
          type: 'Preference';
          value: ModePreference;
      };
export type AspectRatioOption =
    | {
          type: 'Any';
      }
    | {
          type: 'Native';
      }
    | {
          type: 'Exact';
          value: number;
      };
export type ModeOptionForDouble =
    | {
          type: 'Exact';
          value: number;
      }
    | {
          type: 'AtLeast';
          value: number;
      }
    | {
          type: 'AtMost';
          value: number;
      };
export type ModeOptionFor_Resolution =
    | {
          type: 'Exact';
          value: Resolution;
      }
    | {
          type: 'AtLeast';
          value: Resolution;
      }
    | {
          type: 'AtMost';
          value: Resolution;
      };
export type MultiWindowLayout =
    | 'column-right'
    | 'column-left'
    | 'square-right'
    | 'square-left'
    | 'separate';
export type LimitedMultiWindowLayout =
    | 'column-right'
    | 'column-left'
    | 'square-right'
    | 'square-left';
export type CitraLayoutOption =
    | {
          type: 'Default';
      }
    | {
          type: 'SingleScreen';
      }
    | {
          type: 'LargeScreen';
      }
    | {
          type: 'SideBySide';
      }
    | {
          type: 'SeparateWindows';
      }
    | {
          type: 'HybridScreen';
      }
    | {
          type: 'Unknown';
          value: number;
      };
export type CemuAudioChannels = 'Mono' | 'Stereo' | 'Surround';
/**
 * melonDS layout options. Because of the "unique" way melonDS handles layouts, these options do not map 1:1.
 */
export type MelonDSLayoutOption =
    | 'Natural'
    | 'Vertical'
    | 'Horizontal'
    | 'Hybrid'
    | 'Single';
export type MelonDSSizingOption =
    | 'Even'
    | 'EmphasizeTop'
    | 'EmphasizeBottom'
    | 'Auto';
export type FileSource =
    | {
          type: 'Flatpak';
          value: FlatpakSource;
      }
    | {
          type: 'AppImage';
          value: AppImageSource;
      }
    | {
          type: 'EmuDeck';
          value: EmuDeckSource;
      }
    | {
          type: 'Custom';
          value: CustomFileOptions;
      };
export type FlatpakSource = 'Cemu' | 'Citra' | 'MelonDS' | 'Lime3ds';
export type AppImageSource = 'Cemu';
export type EmuDeckSource = 'CemuProton';
export type SecondaryAppScreenPreference = 'PreferSecondary' | 'PreferPrimary';
export type SecondaryAppWindowingBehavior =
    | 'Fullscreen'
    | 'Maximized'
    | 'Minimized'
    | 'Unmanaged';
export type SecondaryApp = {
    app_id: string;
    args: string[];
    type: 'Flatpak';
};
export type PipelineActionUpdate =
    | {
          type: 'UpdateEnabled';
          value: {
              is_enabled: boolean;
          };
      }
    | {
          type: 'UpdateProfileOverride';
          value: {
              profile_override?: string | null;
          };
      }
    | {
          type: 'UpdateOneOf';
          value: {
              selection: string;
          };
      }
    | {
          type: 'UpdateAction';
          value: {
              action: Action;
          };
      }
    | {
          type: 'UpdateVisibleOnQAM';
          value: {
              is_visible: boolean;
          };
      };
export type DependencyError =
    | {
          type: 'SystemCmdNotFound';
          value: string;
      }
    | {
          type: 'PathIsNotFile';
          value: string;
      }
    | {
          type: 'PathIsNotDir';
          value: string;
      }
    | {
          type: 'PathNotFound';
          value: string;
      }
    | {
          type: 'KwinScriptNotFound';
          value: string;
      }
    | {
          type: 'KwinScriptFailedInstall';
          value: string;
      }
    | {
          type: 'FieldNotSet';
          value: string;
      }
    | {
          type: 'FlatpakNotFound';
          value: string;
      }
    | {
          type: 'SecondaryAppPresetNotFound';
          value: string;
      };
export type RuntimeSelection =
    | {
          type: 'Action';
          value: Action;
      }
    | {
          type: 'OneOf';
          value: {
              actions: PipelineAction[];
              selection: string;
          };
      }
    | {
          type: 'AllOf';
          value: PipelineAction[];
      };

/**
 * Marker type for generating API json schema types for ts
 */
export interface Api {
    autostart_request: AutoStartRequest;
    create_profile_request: CreateProfileRequest;
    create_profile_response: CreateProfileResponse;
    delete_profile_request: DeleteProfileRequest;
    get_app_profile_request: GetAppProfileRequest;
    get_app_profile_response: GetAppProfileResponse;
    get_audio_device_info: GetAudioDeviceInfoResponse;
    get_default_app_override_for_profile_request: GetDefaultAppOverrideForProfileRequest;
    get_default_app_override_for_profile_response: GetDefaultAppOverrideForProfileResponse;
    get_display_info: GetDisplayInfoResponse;
    get_profile_request: GetProfileRequest;
    get_profile_response: GetProfileResponse;
    get_profiles_response: GetProfilesResponse;
    get_secondary_app_info: GetSecondaryAppInfoResponse;
    get_settings_response: GetSettingsResponse;
    get_templates_response: GetTemplatesResponse;
    get_toplevel_response: GetTopLevelResponse;
    patch_pipeline_action_request: PatchPipelineActionRequest;
    patch_pipeline_action_response: PatchPipelineActionResponse;
    reify_pipeline_request: ReifyPipelineRequest;
    reify_pipeline_response: ReifyPipelineResponse;
    set_app_profile_override_request: SetAppProfileOverrideRequest;
    set_app_profile_settings_request: SetAppProfileSettingsRequest;
    set_profile_request: SetProfileRequest;
    set_settings_request: SetSettingsRequest;
}
export interface AutoStartRequest {
    app_id: string;
    game_id?: string | null;
    game_title: string;
    profile_id: string;
    target: PipelineTarget;
    user_id_64: string;
}
export interface CreateProfileRequest {
    pipeline: PipelineDefinition;
}
export interface PipelineDefinition {
    desktop_layout_config_hack_override?: boolean | null;
    exit_hooks_override?: ExitHooks | null;
    id: string;
    name: string;
    platform: TopLevelDefinition;
    primary_target_override?: PipelineTarget | null;
    should_register_exit_hooks: boolean;
    toplevel: TopLevelDefinition[];
}
/**
 * Defines a top-level action, with a root id and a unique set of actions. This allows multiple top-level actions of the same type, without complicating the structure too much.
 */
export interface TopLevelDefinition {
    actions: PipelineActionLookup;
    id: string;
    root: string;
}
export interface PipelineActionLookup {
    actions: {
        [k: string]: PipelineActionSettingsFor_ConfigSelection;
    };
}
export interface PipelineActionSettingsFor_ConfigSelection {
    /**
     * Flags whether the selection is enabled. If None, not optional. If Some(true), optional and enabled, else disabled.
     */
    enabled?: boolean | null;
    /**
     * Whether or not the pipeline action is hidden on the QAM
     */
    is_visible_on_qam: boolean;
    /**
     * Flags whether the selection is overridden by the setting from a different profile.
     */
    profile_override?: string | null;
    /**
     * The value of the pipeline action
     */
    selection: ConfigSelection;
}
export interface DesktopSessionHandler {
    deck_is_primary_display: boolean;
    id: string;
    teardown_deck_location?: RelativeLocation | null;
    teardown_external_settings: ExternalDisplaySettings;
}
export interface ModePreference {
    aspect_ratio: AspectRatioOption;
    refresh: ModeOptionForDouble;
    resolution: ModeOptionFor_Resolution;
}
export interface Resolution {
    h: number;
    w: number;
}
export interface DisplayConfig {
    deck_is_primary_display: boolean;
    deck_location?: RelativeLocation | null;
    external_display_settings: ExternalDisplaySettings;
    id: string;
}
export interface VirtualScreen {
    id: string;
}
export interface MultiWindow {
    /**
     * Some(options) if Cemu is configurable, None otherwise
     */
    cemu?: CemuWindowOptions | null;
    /**
     * Some(options) if Citra is configurable, None otherwise
     */
    citra?: CitraWindowOptions | null;
    custom?: CustomWindowOptions | null;
    /**
     * Some(options) if Dolphin is configurable, None otherwise
     */
    dolphin?: DolphinWindowOptions | null;
    general: GeneralOptions;
    id: string;
}
export interface CemuWindowOptions {
    multi_screen_layout: MultiWindowLayout;
    single_screen_layout: LimitedMultiWindowLayout;
}
export interface CitraWindowOptions {
    multi_screen_layout: MultiWindowLayout;
    single_screen_layout: LimitedMultiWindowLayout;
}
export interface CustomWindowOptions {
    classes: string[];
    multi_screen_multi_secondary_layout: MultiWindowLayout;
    multi_screen_single_secondary_layout: MultiWindowLayout;
    primary_window_matcher?: string | null;
    secondary_window_matcher?: string | null;
    single_screen_layout: LimitedMultiWindowLayout;
}
export interface DolphinWindowOptions {
    gba_blacklist: number[];
    multi_screen_multi_secondary_layout: MultiWindowLayout;
    multi_screen_single_secondary_layout: MultiWindowLayout;
    single_screen_layout: LimitedMultiWindowLayout;
}
export interface GeneralOptions {
    keep_above: boolean;
    swap_screens: boolean;
}
export interface CitraLayout {
    id: string;
    layout: CitraLayoutState;
}
export interface CitraLayoutState {
    fullscreen: boolean;
    layout_option: CitraLayoutOption;
    rotate_upright: boolean;
    swap_screens: boolean;
}
export interface CemuLayout {
    id: string;
    layout: CemuLayoutState;
}
export interface CemuLayoutState {
    fullscreen: boolean;
    separate_gamepad_view: boolean;
}
export interface CemuAudio {
    id: string;
    state: CemuAudioState;
}
export interface CemuAudioState {
    mic_in: CemuAudioSetting;
    pad_out: CemuAudioSetting;
    tv_out: CemuAudioSetting;
}
export interface CemuAudioSetting {
    channels: CemuAudioChannels;
    device: string;
    volume: number;
}
export interface MelonDSLayout {
    book_mode: boolean;
    id: string;
    layout_option: MelonDSLayoutOption;
    sizing_option: MelonDSSizingOption;
    swap_screens: boolean;
}
export interface SourceFile {
    id: string;
    source: FileSource;
}
export interface CustomFileOptions {
    /**
     * user defined custom path
     */
    path?: string | null;
    /**
     * valid file extensions for source file
     */
    valid_ext: string[];
}
export interface LaunchSecondaryFlatpakApp {
    app: FlatpakApp;
    id: string;
    screen_preference: SecondaryAppScreenPreference;
    windowing_behavior: SecondaryAppWindowingBehavior;
}
export interface FlatpakApp {
    app_id: string;
    args: string[];
}
export interface LaunchSecondaryAppPreset {
    id: string;
    preset: string;
    screen_preference: SecondaryAppScreenPreference;
    windowing_behavior: SecondaryAppWindowingBehavior;
}
export interface MainAppAutomaticWindowing {
    general: GeneralOptions;
    id: string;
}
export interface CreateProfileResponse {
    profile_id: string;
}
export interface DeleteProfileRequest {
    profile: string;
}
export interface GetAppProfileRequest {
    app_id: string;
}
export interface GetAppProfileResponse {
    app: AppProfile;
}
export interface AppProfile {
    default_profile?: string | null;
    id: string;
    overrides: {
        [k: string]: PipelineDefinition;
    };
}
/**
 * Get Audio Device Info
 */
export interface GetAudioDeviceInfoResponse {
    sinks: AudioDeviceInfo[];
    sources: AudioDeviceInfo[];
}
export interface AudioDeviceInfo {
    channels?: number | null;
    description: string;
    name: string;
}
export interface GetDefaultAppOverrideForProfileRequest {
    profile_id: string;
}
export interface GetDefaultAppOverrideForProfileResponse {
    pipeline?: PipelineDefinition | null;
}
/**
 * Get Display Info
 */
export interface GetDisplayInfoResponse {
    available_values: DisplayValues[];
}
export interface DisplayValues {
    height: number;
    refresh?: number | null;
    width: number;
}
export interface GetProfileRequest {
    profile_id: string;
}
export interface GetProfileResponse {
    profile?: CategoryProfile | null;
}
export interface CategoryProfile {
    id: string;
    pipeline: PipelineDefinition;
    tags: string[];
}
export interface GetProfilesResponse {
    profiles: CategoryProfile[];
}
export interface GetSecondaryAppInfoResponse {
    installed_flatpaks: FlatpakInfo[];
    presets: {
        [k: string]: SecondaryAppPreset;
    };
}
export interface FlatpakInfo {
    app_id: string;
    name: string;
}
export interface SecondaryAppPreset {
    app: SecondaryApp;
    name: string;
}
export interface GetSettingsResponse {
    global_settings: GlobalConfig;
}
export interface GlobalConfig {
    display_restoration: DesktopSessionHandler;
    /**
     * If true, inject buttons onto app action bar
     */
    enable_ui_inject: boolean;
    /**
     * Button chord to be used to exit profiles that register for exit hooks.
     */
    exit_hooks: ExitHooks;
    /**
     * If `enable_ui_inject` is true, set the "Play" button to this target
     */
    primary_ui_target: PipelineTarget;
    restore_displays_if_not_executing_pipeline: boolean;
    /**
     * Overwrite the desktop layout with the game layout
     */
    use_desktop_controller_layout_hack: boolean;
}
export interface GetTemplatesResponse {
    templates: Template[];
}
export interface Template {
    id: string;
    pipeline: PipelineDefinition;
    tags: string[];
}
export interface GetTopLevelResponse {
    toplevel: ToplevelInfo[];
}
export interface ToplevelInfo {
    description?: string | null;
    id: string;
    name: string;
}
export interface PatchPipelineActionRequest {
    action_id: string;
    pipeline: PipelineDefinition;
    target: PipelineTarget;
    toplevel_id: string;
    update: PipelineActionUpdate;
}
export interface PatchPipelineActionResponse {
    pipeline: PipelineDefinition;
}
export interface ReifyPipelineRequest {
    pipeline: PipelineDefinition;
}
export interface ReifyPipelineResponse {
    config_errors: {
        [k: string]: DependencyError[];
    };
    pipeline: Pipeline;
}
export interface Pipeline {
    description: string;
    desktop_layout_config_hack_override?: boolean | null;
    exit_hooks_override?: ExitHooks | null;
    name: string;
    primary_target_override?: PipelineTarget | null;
    should_register_exit_hooks: boolean;
    targets: {
        [k: string]: RuntimeSelection;
    };
}
export interface PipelineAction {
    description?: string | null;
    /**
     * Flags whether the selection is enabled. If None, not optional. If Some(true), optional and enabled, else disabled.
     */
    enabled?: boolean | null;
    id: string;
    /**
     * Whether or not the pipeline action is hidden on the QAM
     */
    is_visible_on_qam: boolean;
    name: string;
    /**
     * Flags whether the selection is overridden by the setting from a different profile.
     */
    profile_override?: string | null;
    /**
     * The value of the pipeline action
     */
    selection: RuntimeSelection;
    toplevel_id: string;
}
export interface SetAppProfileOverrideRequest {
    app_id: string;
    pipeline: PipelineDefinition;
    profile_id: string;
}
export interface SetAppProfileSettingsRequest {
    app_id: string;
    default_profile?: string | null;
}
export interface SetProfileRequest {
    profile: CategoryProfile;
}
export interface SetSettingsRequest {
    global_settings: GlobalConfig;
}
