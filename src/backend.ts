import { AutoStartRequest, CitraLayoutOption, CreateProfileRequest, CreateProfileResponse, GetProfileRequest, GetProfileResponse, GetProfilesResponse, GetTemplatesResponse, MelonDSLayoutOption, MelonDSSizingOption, PipelineAction, ReifyPipelineRequest, ReifyPipelineResponse, SelectionFor_PipelineAction, SelectionFor_String, SetProfileRequest } from "./types/backend_api";
import { call_backend, init_embedded, init_usdpl, target_usdpl } from "./usdpl_front";
import { Err, Ok, Result } from "./util/result";

export {
    Action,
    AutoStartRequest,
    CreateProfileRequest,
    CreateProfileResponse,
    GetProfileRequest,
    GetProfileResponse,
    GetProfilesResponse,
    GetTemplatesResponse,
    PipelineTarget,
    Profile,
    ReifyPipelineRequest,
    ReifyPipelineResponse,
    SetProfileRequest,
    Template
} from "./types/backend_api";


const USDPL_PORT: number = 44666;


// Pipeline

export type ActionOneOf = { selection: string, actions: PipelineAction[] }
export type ActionSelection = SelectionFor_PipelineAction;

export type DefinitionOneOf = { selection: string, actions: string[] }
export type DefinitionSelection = SelectionFor_String;



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

async function call_backend_typed<T, R>(fn: string, arg: T): Response<R> {
    const args = [arg];
    const res = (await call_backend(fn, args));
    console.log("DeckDS: api", `${fn}(`, args, ') ->', res);
    const code = res ? res[0] : 0;

    switch (code) {
        case StatusCode.Ok: {
            return Ok(res[1]); // no good way to typecheck here, so we assume the value is valid.
        }
        default: {
            return Err({
                code: code,
                err: res ? res[1] : 'unspecified error occurred' // assume an error string
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

export async function reifyPipeline(request: ReifyPipelineRequest): Response<ReifyPipelineResponse> {
    return await call_backend_typed('reify_pipeline', request);
}


