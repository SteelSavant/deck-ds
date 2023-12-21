/* eslint-disable */
/**
 * This file was automatically generated by json-schema-to-typescript.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run json-schema-to-typescript to regenerate this file.
 */

export type SelectionFor_PipelineAction =
  | {
      type: "Action";
      value: Action;
    }
  | {
      type: "OneOf";
      value: {
        actions: PipelineAction[];
        selection: string;
      };
    }
  | {
      type: "AllOf";
      value: PipelineAction[];
    };
export type Action =
  | {
      type: "DisplayRestoration";
      value: DisplayRestoration;
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
export type TeardownExternalSettings =
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
export type VirtualScreen = null;
export type MultiWindow = null;
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
export type SourceFile =
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
export type PipelineTarget = "Desktop" | "Gamemode";
export type SelectionFor_String =
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

/**
 * Marker type for generating API json schema types for ts
 */
export interface Api {
  autostart_request: AutoStartRequest;
  create_profile_request: CreateProfileRequest;
  create_profile_response: CreateProfileResponse;
  delete_profile_request: DeleteProfileRequest;
  get_profile_request: GetProfileRequest;
  get_profile_response: GetProfileResponse;
  get_profiles_response: GetProfilesResponse;
  get_templates_response: GetTemplatesResponse;
  reify_pipeline_request: ReifyPipelineRequest;
  reify_pipeline_response: ReifyPipelineResponse;
  set_profile_request: SetProfileRequest;
}
export interface AutoStartRequest {
  app: string;
  pipeline: Pipeline;
  target: PipelineTarget;
}
export interface Pipeline {
  description: string;
  name: string;
  tags: string[];
  targets: {
    [k: string]: SelectionFor_PipelineAction;
  };
}
export interface DisplayRestoration {
  teardown_deck_location: RelativeLocation;
  teardown_external_settings: TeardownExternalSettings;
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
export interface CitraLayout {
  fullscreen: boolean;
  layout_option: CitraLayoutOption;
  swap_screens: boolean;
}
export interface CemuLayout {
  fullscreen: boolean;
  separate_gamepad_view: boolean;
}
export interface MelonDSLayout {
  book_mode: boolean;
  layout_option: MelonDSLayoutOption;
  sizing_option: MelonDSSizingOption;
  swap_screens: boolean;
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
export interface PipelineAction {
  description?: string | null;
  /**
   * Flags whether the selection is enabled. If None, not optional. If Some(true), optional and enabled, else disabled.
   */
  enabled?: boolean | null;
  id: string;
  name: string;
  /**
   * Flags whether the selection is overridden by the setting from a different profile.
   */
  profile_override?: string | null;
  /**
   * The value of the pipeline action
   */
  selection: SelectionFor_PipelineAction;
}
export interface CreateProfileRequest {
  pipeline: PipelineDefinition;
}
export interface PipelineDefinition {
  actions: PipelineActionRegistrar;
  description: string;
  name: string;
  tags: string[];
  targets: {
    [k: string]: SelectionFor_String;
  };
}
export interface PipelineActionRegistrar {
  actions: {
    [k: string]: PipelineActionDefinition;
  };
}
export interface PipelineActionDefinition {
  description?: string | null;
  /**
   * Flags whether the selection is enabled. If None, not optional. If Some(true), optional and enabled, else disabled.
   */
  enabled?: boolean | null;
  id: string;
  name: string;
  /**
   * Flags whether the selection is overridden by the setting from a different profile.
   */
  profile_override?: string | null;
  /**
   * The value of the pipeline action
   */
  selection: SelectionFor_String;
}
export interface CreateProfileResponse {
  profile_id: string;
}
export interface DeleteProfileRequest {
  profile: string;
}
export interface GetProfileRequest {
  profile_id: string;
}
export interface GetProfileResponse {
  profile?: Profile | null;
}
export interface Profile {
  id: string;
  pipeline: PipelineDefinition;
}
export interface GetProfilesResponse {
  profiles: Profile[];
}
export interface GetTemplatesResponse {
  templates: Template[];
}
export interface Template {
  id: string;
  pipeline: PipelineDefinition;
}
export interface ReifyPipelineRequest {
  pipeline: PipelineDefinition;
}
export interface ReifyPipelineResponse {
  pipeline: Pipeline;
}
export interface SetProfileRequest {
  profile: Profile;
}
