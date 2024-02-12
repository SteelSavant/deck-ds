/* eslint-disable */
/**
 * This file was automatically generated by json-schema-to-typescript.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run json-schema-to-typescript to regenerate this file.
 */

export type PipelineTarget = "Desktop" | "Gamemode";
export type SelectionFor_ActionAnd_String =
  | {
      type: "Action";
      value: Action;
    }
  | {
      type: "OneOf";
      value: {
        actions: string[];
        selection: string;
      };
    }
  | {
      type: "AllOf";
      value: string[];
    };
export type Action =
  | {
      type: "DesktopSessionHandler";
      value: DesktopSessionHandler;
    }
  | {
      type: "DisplayConfig";
      value: DisplayConfig;
    }
  | {
      type: "VirtualScreen";
      value: VirtualScreen;
    }
  | {
      type: "MultiWindow";
      value: MultiWindow;
    }
  | {
      type: "CitraLayout";
      value: CitraLayout;
    }
  | {
      type: "CemuLayout";
      value: CemuLayout;
    }
  | {
      type: "MelonDSLayout";
      value: MelonDSLayout;
    }
  | {
      type: "SourceFile";
      value: SourceFile;
    };
export type RelativeLocation = "Above" | "Below" | "LeftOf" | "RightOf" | "SameAs";
export type ExternalDisplaySettings =
  | {
      type: "Previous";
    }
  | {
      type: "Native";
    }
  | {
      type: "Preference";
      value: ModePreference;
    };
export type AspectRatioOption =
  | ("Any" | "Native")
  | {
      Exact: number;
    };
export type ModeOptionForDouble =
  | {
      Exact: number;
    }
  | {
      AtLeast: number;
    }
  | {
      AtMost: number;
    };
export type ModeOptionFor_Resolution =
  | {
      Exact: Resolution;
    }
  | {
      AtLeast: Resolution;
    }
  | {
      AtMost: Resolution;
    };
export type CitraLayoutOption =
  | {
      type: "Default";
    }
  | {
      type: "SingleScreen";
    }
  | {
      type: "LargeScreen";
    }
  | {
      type: "SideBySide";
    }
  | {
      type: "SeparateWindows";
    }
  | {
      type: "HybridScreen";
    }
  | {
      type: "Unknown";
      value: number;
    };
/**
 * melonDS layout options. Because of the "unique" way melonDS handles layouts, these options do not map 1:1.
 */
export type MelonDSLayoutOption = "Natural" | "Vertical" | "Horizontal" | "Hybrid" | "Single";
export type MelonDSSizingOption = "Even" | "EmphasizeTop" | "EmphasizeBottom" | "Auto";
export type FileSource =
  | {
      type: "Flatpak";
      value: FlatpakSource;
    }
  | {
      type: "AppImage";
      value: AppImageSource;
    }
  | {
      type: "EmuDeck";
      value: EmuDeckSource;
    }
  | {
      type: "Custom";
      value: CustomFileOptions;
    };
export type FlatpakSource = "Cemu" | "Citra" | "MelonDS";
export type AppImageSource = "Cemu";
export type EmuDeckSource = "CemuProton";
export type DependencyError =
  | {
      type: "SystemCmdNotFound";
      value: string;
    }
  | {
      type: "PathIsNotFile";
      value: string;
    }
  | {
      type: "PathIsNotDir";
      value: string;
    }
  | {
      type: "PathNotFound";
      value: string;
    }
  | {
      type: "KwinScriptNotFound";
      value: string;
    }
  | {
      type: "KwinScriptFailedInstall";
      value: string;
    }
  | {
      type: "FieldNotSet";
      value: string;
    };
export type SelectionFor_ActionAnd_PipelineActionFor_Action =
  | {
      type: "Action";
      value: Action;
    }
  | {
      type: "OneOf";
      value: {
        actions: PipelineActionFor_Action[];
        selection: string;
      };
    }
  | {
      type: "AllOf";
      value: PipelineActionFor_Action[];
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
  get_default_app_override_for_profile_request: GetDefaultAppOverrideForProfileRequest;
  get_default_app_override_for_profile_response: GetDefaultAppOverrideForProfileResponse;
  get_profile_request: GetProfileRequest;
  get_profile_response: GetProfileResponse;
  get_profiles_response: GetProfilesResponse;
  get_settings_response: GetSettingsResponse;
  get_templates_response: GetTemplatesResponse;
  reify_pipeline_request: ReifyPipelineRequest;
  reify_pipeline_response: ReifyPipelineResponse;
  set_app_profile_override_request: SetAppProfileOverrideRequest;
  set_app_profile_settings_request: SetAppProfileSettingsRequest;
  set_profile_request: SetProfileRequest;
  set_settings_request: SetSettingsRequest;
}
export interface AutoStartRequest {
  app_id: string;
  game_id: string;
  profile_id: string;
  target: PipelineTarget;
}
export interface CreateProfileRequest {
  pipeline: PipelineDefinitionFor_Action;
}
export interface PipelineDefinitionFor_Action {
  actions: PipelineActionLookupFor_Action;
  description: string;
  name: string;
  primary_target_override?: PipelineTarget | null;
  register_exit_hooks: boolean;
  targets: {
    [k: string]: SelectionFor_ActionAnd_String;
  };
}
export interface PipelineActionLookupFor_Action {
  actions: {
    [k: string]: PipelineActionSettingsFor_Action;
  };
}
export interface PipelineActionSettingsFor_Action {
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
  selection: SelectionFor_ActionAnd_String;
}
export interface DesktopSessionHandler {
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
  deck_location?: RelativeLocation | null;
  disable_splash: boolean;
  external_display_settings: ExternalDisplaySettings;
  id: string;
}
export interface VirtualScreen {
  id: string;
}
export interface MultiWindow {
  id: string;
}
export interface CitraLayout {
  id: string;
  layout: CitraLayoutState;
}
export interface CitraLayoutState {
  fullscreen: boolean;
  layout_option: CitraLayoutOption;
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
    [k: string]: PipelineDefinitionFor_Action;
  };
}
export interface GetDefaultAppOverrideForProfileRequest {
  profile_id: string;
}
export interface GetDefaultAppOverrideForProfileResponse {
  pipeline?: PipelineDefinitionFor_Action | null;
}
export interface GetProfileRequest {
  profile_id: string;
}
export interface GetProfileResponse {
  profile?: CategoryProfile | null;
}
export interface CategoryProfile {
  id: string;
  pipeline: PipelineDefinitionFor_Action;
  tags: string[];
}
export interface GetProfilesResponse {
  profiles: CategoryProfile[];
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
   * If `enable_ui_inject` is true, set the "Play" button to this target
   */
  primary_ui_target: PipelineTarget;
  restore_displays_if_not_executing_pipeline: boolean;
}
export interface GetTemplatesResponse {
  templates: Template[];
}
export interface Template {
  id: string;
  pipeline: PipelineDefinitionFor_Action;
}
export interface ReifyPipelineRequest {
  pipeline: PipelineDefinitionFor_Action;
}
export interface ReifyPipelineResponse {
  config_errors: {
    [k: string]: DependencyError[];
  };
  pipeline: PipelineFor_Action;
}
export interface PipelineFor_Action {
  description: string;
  name: string;
  primary_target_override?: PipelineTarget | null;
  register_exit_hooks: boolean;
  targets: {
    [k: string]: SelectionFor_ActionAnd_PipelineActionFor_Action;
  };
}
export interface PipelineActionFor_Action {
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
  selection: SelectionFor_ActionAnd_PipelineActionFor_Action;
}
export interface SetAppProfileOverrideRequest {
  app_id: string;
  pipeline: PipelineDefinitionFor_Action;
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
