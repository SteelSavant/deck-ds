import {
    AutoStartRequest, CategoryProfile, CitraLayoutOption,
    CreateProfileRequest, CreateProfileResponse,
    DeleteProfileRequest, GetAppProfileRequest, GetAppProfileResponse,
    GetAudioDeviceInfoResponse,
    GetDefaultAppOverrideForProfileRequest, GetDefaultAppOverrideForProfileResponse,
    GetDisplayInfoResponse,
    GetProfileRequest, GetProfileResponse, GetProfilesResponse, GetSecondaryAppInfoResponse, GetSettingsResponse,
    GetTemplatesResponse, GetTopLevelResponse, MelonDSLayoutOption, MelonDSSizingOption, PatchPipelineActionRequest, PatchPipelineActionResponse, PipelineAction, PipelineActionSettingsFor_ConfigSelection, PipelineDefinition,
    ReifyPipelineRequest, ReifyPipelineResponse,
    SecondaryAppScreenPreference,
    SecondaryAppWindowingBehavior,
    SetAppProfileOverrideRequest,
    SetAppProfileSettingsRequest, SetProfileRequest, SetSettingsRequest, Template
} from "./types/backend_api";
import { call_backend, init_embedded, init_usdpl, target_usdpl } from "./usdpl_front";
import { Err, Ok, Result } from "./util/result";

export {
    Action, AppProfile, AutoStartRequest, CategoryProfile, CemuWindowOptions, CitraWindowOptions, ConfigSelection, CreateProfileRequest,
    CreateProfileResponse, DeleteProfileRequest, DependencyError, DolphinWindowOptions, ExternalDisplaySettings, GetProfileRequest,
    GetProfileResponse, GetProfilesResponse, GetTemplatesResponse, GlobalConfig, LaunchSecondaryAppPreset, LaunchSecondaryFlatpakApp, LimitedMultiWindowLayout, MultiWindowLayout, PipelineAction, PipelineDefinition, PipelineTarget, ReifyPipelineRequest,
    ReifyPipelineResponse, RelativeLocation, RuntimeSelection, SecondaryAppWindowingBehavior, SetProfileRequest, Template
} from "./types/backend_api";


const USDPL_PORT: number = 44666;

export const secondaryAppWindowingOptions: SecondaryAppWindowingBehavior[] = ['Fullscreen', 'Maximized', 'Minimized', 'Unmanaged'];
export const secondaryAppScreenPreferences: SecondaryAppScreenPreference[] = ['PreferSecondary', 'PreferPrimary'];

// Pipeline
export type ActionOneOf = { selection: string, actions: PipelineAction[] }

export type DefinitionOneOf = { selection: string, actions: string[] }

export type PipelineActionSettings = PipelineActionSettingsFor_ConfigSelection;

export interface AppProfileOveride {
    profileId: string,
    appId: string,
    pipeline: PipelineDefinition
};
export type PipelineContainer = CategoryProfile | Template | AppProfileOveride;

export function isCategoryProfile(container: PipelineContainer): container is CategoryProfile {
    return (container as any).tags !== undefined;
}

// Action 

export const citraLayoutOptions: CitraLayoutOption[] = [{ type: "Default" }, { type: "SingleScreen" }, { type: "LargeScreen" }, { type: "SideBySide" }, { type: "SeparateWindows" }, { type: "HybridScreen" }];
export const melonDSLayoutOptions: MelonDSLayoutOption[] = ['Natural', 'Vertical', 'Horizontal', 'Hybrid', 'Single'];
export const melonDSSizingOptions: MelonDSSizingOption[] = ['Even', 'Auto', 'EmphasizeTop', 'EmphasizeBottom'];

// Utility

export function resolve<T>(promise: Promise<T>, setter: (t: T) => void) {
    (async function () {
        let data = await promise;
        if (data != null) {
            console.debug("Got resolved", data);
            setter(data);
        } else {
            console.warn("Resolve failed:", data, promise);
            log(LogLevel.Warn, "A resolve failed");
        }
    })();
}

export function resolve_nullable<T>(promise: Promise<T | null>, setter: (t: T | null) => void) {
    (async function () {
        let data = await promise;
        console.debug("Got resolved", data);
        setter(data);
    })();
}

export async function initBackend() {
    // init usdpl
    await init_embedded();
    init_usdpl(USDPL_PORT);
    console.log("DeckDS: USDPL started for framework: " + target_usdpl());
    const user_locale =
        navigator.languages && navigator.languages.length
            ? navigator.languages[0]
            : navigator.language;
    console.log("DeckDS: locale", user_locale);
    //let mo_path = "../plugins/DeckDS/translations/" + user_locale.toString() + ".mo";
    // await init_tr(user_locale);
    //await init_tr("../plugins/DeckDS/translations/test.mo");
    //setReady(true);
}

export enum StatusCode {
    Ok = 200,
    BadRequest = 400,
    ServerError = 500,
}

export type ApiError = { code: StatusCode.BadRequest | StatusCode.ServerError, err: string };

export type Response<T> = Promise<Result<T, ApiError>>

let _id = 0;

async function call_backend_typed<T, R>(fn: string, arg: T): Response<R> {

    // USDPL has a comparatively small content limit, so we chunk manually to bypass.
    const stringified = JSON.stringify(arg);
    const bytesLen = stringified.length;
    const maxBytes = 1024;

    if (bytesLen > maxBytes) {
        _id++;
        if (!isFinite(_id) || _id < 0 || _id >= 18446744073709551615) {
            _id = 0;
        }
        const id = _id;

        const windowLen = maxBytes;
        for (let i = 0; i <= bytesLen; i += windowLen) {
            const end = i + windowLen;
            const slice = stringified.slice(i, end);
            if (slice.length > 0) {
                // console.log('writing chunk', i / windowLen, 'of', bytesLen / windowLen);

                let res = await call_backend("chunked_request", [id, slice]);
                let typed = handle_backend_response<R>(res); // not really <R>, but we'll never return the OK, so its fine.

                if (!typed.isOk) {
                    console.log('error chunking request', typed.err);
                    return typed;
                }
            }
        }

        let res = await call_backend(fn, ["Chunked", id]);
        console.log("DeckDS: api", `${fn}(`, arg, ') ->', res);

        return handle_backend_response(res);

    } else {
        const args = ["Full", arg];
        const res = (await call_backend(fn, args));
        console.log("DeckDS: api", `${fn}(`, arg, ') ->', res);

        return handle_backend_response(res);
    }
}

function handle_backend_response<T>(res: any): Result<T, ApiError> {
    const code = res ? res[0] : 0;

    switch (code) {
        case StatusCode.Ok: {
            return Ok(res[1]); // no good way to typecheck here, so we assume the value is valid.
        }
        default: {
            return Err({
                code: code,
                err: res ? res[1] // assume an error string
                    : 'unspecified error occurred'
            })
        }
    }
}

// Logging

export enum LogLevel {
    Trace = 1,
    Debug = 2,
    Info = 3,
    Warn = 4,
    Error = 5,
}

export async function log(level: LogLevel, msg: string): Promise<boolean> {
    return (await call_backend("LOG", [level, msg]))[0];
}

export async function logPath(): Promise<String> {
    return (await call_backend("LOGPATH", []))[0];
}

// API

// Autostart

export async function autoStart(request: AutoStartRequest): Response<void> {
    return await call_backend_typed("autostart", request)
}

// CategoryProfile

export async function createProfile(request: CreateProfileRequest): Response<CreateProfileResponse> {
    return await call_backend_typed("create_profile", request)
}

export async function getProfile(request: GetProfileRequest): Response<GetProfileResponse> {
    return await call_backend_typed("get_profile", request)
}

export async function setProfile(request: SetProfileRequest): Response<void> {
    return await call_backend_typed("set_profile", request)
}

export async function deleteProfile(request: DeleteProfileRequest): Response<void> {
    return await call_backend_typed("delete_profile", request)
}

export async function getProfiles(): Response<GetProfilesResponse> {
    return await call_backend_typed("get_profiles", null);
}

// AppProfile

export async function getAppProfile(request: GetAppProfileRequest): Response<GetAppProfileResponse> {
    return await call_backend_typed("get_app_profile", request)
}

export async function setAppProfileSettings(request: SetAppProfileSettingsRequest): Response<void> {
    return await call_backend_typed("set_app_profile_settings", request)
}

export async function setAppProfileOverride(request: SetAppProfileOverrideRequest): Response<void> {
    return await call_backend_typed("set_app_profile_override", request)
}

export async function getDefaultAppOverrideForProfileRequest(request: GetDefaultAppOverrideForProfileRequest): Response<GetDefaultAppOverrideForProfileResponse> {
    return await call_backend_typed("get_default_app_override_for_profile_request", request);
}

export async function patchPipelineAction(request: PatchPipelineActionRequest): Response<PatchPipelineActionResponse> {
    return await call_backend_typed('patch_pipeline_action', request);
}

export async function reifyPipeline(request: ReifyPipelineRequest): Response<ReifyPipelineResponse> {
    return await call_backend_typed('reify_pipeline', request);
}

export async function getToplevel(): Response<GetTopLevelResponse> {
    return await call_backend_typed("get_toplevel", null);
}

// Templates

export async function getTemplates(): Response<GetTemplatesResponse> {
    return await call_backend_typed("get_templates", null);
}

// Secondary Apps

export async function getSecondaryAppInfo(): Response<GetSecondaryAppInfoResponse> {
    return await call_backend_typed("get_secondary_app_info", null);
}


// Settings

export async function getSettings(): Response<GetSettingsResponse> {
    return await call_backend_typed("get_settings", null);
}

export async function setSettings(request: SetSettingsRequest): Response<void> {
    return await call_backend_typed("set_settings", request);
}

// system Info

export async function getDisplayInfo(): Response<GetDisplayInfoResponse> {
    return await call_backend_typed("get_display_info", null);
}


export async function getAudioDeviceInfo(): Response<GetAudioDeviceInfoResponse> {
    return await call_backend_typed("get_audio_device_info", null);
}