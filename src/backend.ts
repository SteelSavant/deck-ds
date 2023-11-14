import { init_usdpl, target_usdpl, init_embedded, call_backend, init_tr } from "usdpl-front";
import { AutoStartRequest, CreateProfileRequest, CreateProfileResponse, GetProfileRequest, GetProfileResponse, GetProfilesResponse, GetTemplateInfosResponse, SetProfileRequest } from "./types/backend_api";

export {
    // Api Types
    AutoStartRequest,
    CreateProfileRequest,
    CreateProfileResponse,
    GetProfileRequest,
    GetProfileResponse,
    SetProfileRequest,
    GetProfilesResponse,
    GetTemplateInfosResponse,

    // Profile Types
    Profile,
    Overrides,
    TemplateInfo,

    // Pipeline Types
    PipelineTarget,
    Selection,
    PipelineAction,
    PipelineDefinition,
    PipelineActionDefinition,
} from "./types/backend_api";

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
    await init_tr(user_locale);
    //await init_tr("../plugins/DeckDS/translations/test.mo");
    //setReady(true);
}

export enum StatusCode {
    Ok = 200,
    BadRequest = 400,
    ServerError = 500,
}

export type Response<T> = Promise<Result<T, { code: StatusCode.BadRequest | StatusCode.ServerError, err: string }>>

async function call_backend_typed<T, R>(fn: string, args: T): Response<R> {
    const res = (await call_backend(fn, [args]));
    const code = res[0];

    switch (code) {
        case StatusCode.Ok: {
            return new Ok(res[1]); // no good way to typecheck here, so we assume the value is valid.
        }
        default: {
            return new Err({
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
    return await call_backend_typed("get_profiles", undefined);
}

export async function getTemplateInfos(): Response<GetTemplateInfosResponse> {
    return await call_backend_typed("get_template_infos", undefined);
}

