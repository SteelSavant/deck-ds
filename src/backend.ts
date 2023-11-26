import { AutoStartRequest, CreateProfileRequest, CreateProfileResponse, GetPipelineActionsResponse, GetProfileRequest, GetProfileResponse, GetProfilesResponse, GetTemplatesResponse, PipelineActionImplFor_Either_WrappedPipelineActionOr_ProfileAction, PipelineActionImplFor_String, PipelineActionImplFor_WrappedPipelineAction, PipelineImplFor_String, PipelineImplFor_WrappedPipelineAction, SelectionFor_Either_WrappedPipelineActionOr_ProfileAction, SelectionFor_String, SelectionFor_WrappedPipelineAction, SetProfileRequest } from "./types/backend_api";
import { call_backend, init_embedded, init_usdpl, target_usdpl } from "./usdpl_front";
import { Err, Ok, Result } from "./util/result";

export {
    // Api Types
    AutoStartRequest,
    CreateProfileRequest,
    CreateProfileResponse,
    GetProfileRequest,
    GetProfileResponse,
    GetProfilesResponse,
    GetTemplatesResponse,
    // Pipeline Types
    PipelineTarget,
    // Profile Types
    Profile, SetProfileRequest, Template
} from "./types/backend_api";

export type DefinitionPipeline = PipelineImplFor_String;
export type ActionPipeline = PipelineImplFor_WrappedPipelineAction;
export type ActionOrProfilePipleine = PipelineActionImplFor_Either_WrappedPipelineActionOr_ProfileAction;

export type DefinitionSelection = SelectionFor_String;
export type ActionSelection = SelectionFor_WrappedPipelineAction;
export type ActionOrProfileSelection = SelectionFor_Either_WrappedPipelineActionOr_ProfileAction;

export type PipelineDefinition = PipelineActionImplFor_String;
export type PipelineAction = PipelineActionImplFor_WrappedPipelineAction;
export type PipelineOrProfileAction = PipelineActionImplFor_Either_WrappedPipelineActionOr_ProfileAction;


const USDPL_PORT: number = 44666;

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

async function call_backend_typed<T, R>(fn: string, arg: T): Response<R> {
    const args = [arg];
    const res = (await call_backend(fn, args));
    console.log("DeckDS: api", `${fn}(${args}) ->`, res);
    const code = res[0];

    switch (code) {
        case StatusCode.Ok: {
            return Ok(res[1]); // no good way to typecheck here, so we assume the value is valid.
        }
        default: {
            return Err({
                code: code,
                err: res[1] // assume an error string
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

export async function autoStart(request: AutoStartRequest): Response<void> {
    return await call_backend_typed("autostart", request)
}

export async function createProfile(request: CreateProfileRequest): Response<CreateProfileResponse> {
    return await call_backend_typed("create_profile", request)
}

export async function getProfile(request: GetProfileRequest): Response<GetProfileResponse> {
    return await call_backend_typed("get_profile", request)
}

export async function setProfile(request: SetProfileRequest): Response<void> {
    return await call_backend_typed("set_profile", request)
}

export async function getProfiles(): Response<GetProfilesResponse> {
    return await call_backend_typed("get_profiles", null);
}

export async function getTemplates(): Response<GetTemplatesResponse> {
    return await call_backend_typed("get_templates", null);
}

export async function getPipelineActions(): Response<GetPipelineActionsResponse> {
    return await call_backend_typed("get_templates", null);
}

