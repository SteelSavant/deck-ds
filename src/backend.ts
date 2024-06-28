import {
    AddClientTeardownActionRequest,
    AutoStartRequest,
    CategoryProfile,
    CitraLayoutOption,
    CreateProfileRequest,
    CreateProfileResponse,
    DeleteProfileRequest,
    GetAppProfileRequest,
    GetAppProfileResponse,
    GetAudioDeviceInfoResponse,
    GetClientTeardownActionsResponse,
    GetDefaultAppOverrideForProfileRequest,
    GetDefaultAppOverrideForProfileResponse,
    GetDisplayInfoResponse,
    GetProfileRequest,
    GetProfileResponse,
    GetProfilesResponse,
    GetSecondaryAppInfoResponse,
    GetSettingsResponse,
    GetTemplatesResponse,
    GetTopLevelResponse,
    MelonDSLayoutOption,
    MelonDSSizingOption,
    PatchPipelineActionRequest,
    PatchPipelineActionResponse,
    PipelineAction,
    PipelineActionSettingsFor_ConfigSelection,
    PipelineDefinition,
    ReifyPipelineRequest,
    ReifyPipelineResponse,
    RemoveClientTeardownActionsRequest,
    SecondaryAppScreenPreference,
    SecondaryAppWindowingBehavior,
    SetAppProfileOverrideRequest,
    SetAppProfileSettingsRequest,
    SetProfileRequest,
    SetSettingsRequest,
    Template,
} from './types/backend_api';
import {
    call_backend,
    init_embedded,
    init_usdpl,
    target_usdpl,
} from './usdpl_front';
import { LogLevel, logger } from './util/log';
import { Err, Ok, Result } from './util/result';

export {
    Action,
    AppProfile,
    AudioDeviceInfo,
    AutoStartRequest,
    CategoryProfile,
    CemuAudio,
    CemuAudioChannels,
    CemuAudioSetting,
    CemuWindowOptions,
    CitraWindowOptions,
    ConfigSelection,
    CreateProfileRequest,
    CreateProfileResponse,
    CustomWindowOptions,
    DeleteProfileRequest,
    DependencyError,
    DolphinWindowOptions,
    ExternalDisplaySettings,
    GamescopeFilter,
    GamescopeFullscreenOption,
    GamescopeScaler,
    GetProfileRequest,
    GetProfileResponse,
    GetProfilesResponse,
    GetTemplatesResponse,
    GlobalConfig,
    LaunchSecondaryAppPreset,
    LaunchSecondaryFlatpakApp,
    LimitedMultiWindowLayout,
    ModePreference,
    MultiWindowLayout,
    Pipeline,
    PipelineAction,
    PipelineDefinition,
    PipelineTarget,
    ReifyPipelineRequest,
    ReifyPipelineResponse,
    RelativeLocation,
    RuntimeSelection,
    SecondaryAppWindowingBehavior,
    SetProfileRequest,
    Template,
} from './types/backend_api';

const USDPL_PORT: number = 44666;

export const secondaryAppWindowingOptions: SecondaryAppWindowingBehavior[] = [
    'Fullscreen',
    'Maximized',
    'Minimized',
    'Unmanaged',
];
export const secondaryAppScreenPreferences: SecondaryAppScreenPreference[] = [
    'PreferSecondary',
    'PreferPrimary',
];

// Pipeline
export type ActionOneOf = { selection: string; actions: PipelineAction[] };

export type DefinitionOneOf = { selection: string; actions: string[] };

export type PipelineActionSettings = PipelineActionSettingsFor_ConfigSelection;

export type GamepadButtonSelection = GamepadButton;
// TODO::don't hardcode this; get it from the display map rust-side
export const gamepadButtonSelectionOptions = new Map<number, string>(
    [
        'R1',
        'R2',
        'L1',
        'L2',
        'NORTH',
        'EAST',
        'WEST',
        'SOUTH',
        'DPAD_UP',
        'DPAD_RIGHT',
        'DPAD_LEFT',
        'DPAD_DOWN',
        'START',
        'SELECT',
        'RSTICK',
        'LSTICK',
        'STEAM',
        'QAM',
        'R4',
        'R5',
        'L4',
        'L5',
        'RSTICK_TOUCH',
        'LSTICK_TOUCH',
        'LPAD',
        'RPAD',
        'LPAD_TOUCH',
        'RPAD_TOUCH',
    ].map((v, i) => [1 << (i + 1), v]),
);

export interface AppProfileOveride {
    profileId: string;
    appId: string;
    pipeline: PipelineDefinition;
}
export type PipelineContainer = CategoryProfile | Template | AppProfileOveride;

export function isCategoryProfile(
    container: PipelineContainer,
): container is CategoryProfile {
    return (container as any).tags !== undefined;
}

// Action

export const citraLayoutOptions: CitraLayoutOption[] = [
    { type: 'Default' },
    { type: 'SingleScreen' },
    { type: 'LargeScreen' },
    { type: 'SideBySide' },
    { type: 'SeparateWindows' },
    { type: 'HybridScreen' },
];
export const melonDSLayoutOptions: MelonDSLayoutOption[] = [
    'Natural',
    'Vertical',
    'Horizontal',
    'Hybrid',
    'Single',
];
export const melonDSSizingOptions: MelonDSSizingOption[] = [
    'Even',
    'Auto',
    'EmphasizeTop',
    'EmphasizeBottom',
];

// Utility

export function resolve<T>(promise: Promise<T>, setter: (t: T) => void) {
    (async function () {
        let data = await promise;
        if (data != null) {
            logger.debug('Got resolved', data);
            setter(data);
        } else {
            logger.warn('Resolve failed:', data, promise);
            log(LogLevel.Warn, 'A resolve failed');
        }
    })();
}

export function resolve_nullable<T>(
    promise: Promise<T | null>,
    setter: (t: T | null) => void,
) {
    (async function () {
        let data = await promise;
        logger.debug('Got resolved', data);
        setter(data);
    })();
}

export async function initBackend() {
    // init usdpl
    await init_embedded();
    init_usdpl(USDPL_PORT);
    await initLogger();

    logger.debug('USDPL started for framework: ' + target_usdpl());
    const user_locale =
        navigator.languages && navigator.languages.length
            ? navigator.languages[0]
            : navigator.language;
    logger.debug('locale', user_locale);
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

export type ApiError = {
    code: StatusCode.BadRequest | StatusCode.ServerError;
    err: string;
};

export type Response<T> = Promise<Result<T, ApiError>>;

let _id = 0;

async function call_backend_typed<T, R>(
    fn: string,
    arg?: T | null,
): Response<R> {
    arg = arg ?? null;

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
                logger.trace(
                    'writing chunk',
                    i / windowLen,
                    'of',
                    bytesLen / windowLen,
                );

                let res = await call_backend('chunked_request', [id, slice]);
                let typed = handle_backend_response<never>(res);

                if (!typed.isOk) {
                    logger.trace('error chunking request', typed.err);
                    return typed;
                }
            }
        }

        let res = await call_backend(fn, ['Chunked', id]);
        logger.debug('api (chunked)', `${fn}(`, arg, ') ->', res);

        return handle_backend_response(res);
    } else {
        const args = ['Full', arg];
        const res = await call_backend(fn, args);
        logger.debug('api (single)', `${fn}(`, arg, ') ->', res);

        return handle_backend_response(res);
    }
}

function handle_backend_response<T>(res: any[]): Result<T, ApiError> {
    const code = res ? res[0] : 0;

    switch (code) {
        case StatusCode.Ok: {
            return Ok(res[1]); // no good way to typecheck here, so we assume the value is valid.
        }
        default: {
            // res[2] is full error string, res[1] is display error
            // TODO::consider proper error type

            res ??= [null, null];

            const unspecifiedMsg = 'unspecified error occurred';

            const level =
                code === StatusCode.BadRequest ? LogLevel.Warn : LogLevel.Error;

            logger.log(
                level,
                'DeckDS backend encountered error:',
                res[2] ?? unspecifiedMsg,
            );

            return Err({
                code: code,
                err: res[1] ?? res[2] ?? unspecifiedMsg,
            });
        }
    }
}

// Logging

async function initLogger() {
    try {
        const currentSettings = await getSettings();
        if (currentSettings.isOk) {
            logger.minLevel = currentSettings.data.global_settings.log_level;
        } else {
            logger.error(
                'failed to fetch backend settings when initializing logger',
            );
        }
    } catch (ex) {
        logger.error(ex);
    }
}

export async function log(level: LogLevel, msg: string): Promise<boolean> {
    return (await call_backend('LOG', [level, msg]))[0];
}

// API

// Autostart

export async function autoStart(request: AutoStartRequest): Response<void> {
    return await call_backend_typed('autostart', request);
}

// CategoryProfile

export async function createProfile(
    request: CreateProfileRequest,
): Response<CreateProfileResponse> {
    return await call_backend_typed('create_profile', request);
}

export async function getProfile(
    request: GetProfileRequest,
): Response<GetProfileResponse> {
    return await call_backend_typed('get_profile', request);
}

export async function setProfile(request: SetProfileRequest): Response<void> {
    return await call_backend_typed('set_profile', request);
}

export async function deleteProfile(
    request: DeleteProfileRequest,
): Response<void> {
    return await call_backend_typed('delete_profile', request);
}

export async function getProfiles(): Response<GetProfilesResponse> {
    return await call_backend_typed('get_profiles');
}

// AppProfile

export async function getAppProfile(
    request: GetAppProfileRequest,
): Response<GetAppProfileResponse> {
    return await call_backend_typed('get_app_profile', request);
}

export async function setAppProfileSettings(
    request: SetAppProfileSettingsRequest,
): Response<void> {
    return await call_backend_typed('set_app_profile_settings', request);
}

export async function setAppProfileOverride(
    request: SetAppProfileOverrideRequest,
): Response<void> {
    return await call_backend_typed('set_app_profile_override', request);
}

export async function getDefaultAppOverrideForProfileRequest(
    request: GetDefaultAppOverrideForProfileRequest,
): Response<GetDefaultAppOverrideForProfileResponse> {
    return await call_backend_typed(
        'get_default_app_override_for_profile_request',
        request,
    );
}

export async function patchPipelineAction(
    request: PatchPipelineActionRequest,
): Response<PatchPipelineActionResponse> {
    return await call_backend_typed('patch_pipeline_action', request);
}

export async function reifyPipeline(
    request: ReifyPipelineRequest,
): Response<ReifyPipelineResponse> {
    return await call_backend_typed('reify_pipeline', request);
}

export async function getToplevel(): Response<GetTopLevelResponse> {
    return await call_backend_typed('get_toplevel');
}

// Client Pipeline

export async function addClientTeardownAction(
    request: AddClientTeardownActionRequest,
): Response<void> {
    return await call_backend_typed('add_client_teardown_action', request);
}

export async function removeClientTeardownActions(
    request: RemoveClientTeardownActionsRequest,
): Response<void> {
    return await call_backend_typed('remove_client_teardown_actions', request);
}

export async function getClientTeardownActions(): Response<GetClientTeardownActionsResponse> {
    return await call_backend_typed('get_client_teardown_actions');
}

// Templates

export async function getTemplates(): Response<GetTemplatesResponse> {
    return await call_backend_typed('get_templates');
}

// Secondary Apps

export async function getSecondaryAppInfo(): Response<GetSecondaryAppInfoResponse> {
    return await call_backend_typed('get_secondary_app_info');
}

// Settings

export async function getSettings(): Response<GetSettingsResponse> {
    return await call_backend_typed('get_settings');
}

export async function setSettings(request: SetSettingsRequest): Response<void> {
    return await call_backend_typed('set_settings', request);
}

// System Info

export async function getDisplayInfo(): Response<GetDisplayInfoResponse> {
    return await call_backend_typed('get_display_info');
}

export async function getAudioDeviceInfo(): Response<GetAudioDeviceInfoResponse> {
    return await call_backend_typed('get_audio_device_info');
}

// Test

export async function testBackendError(): Response<never> {
    return await call_backend_typed('test_error');
}
