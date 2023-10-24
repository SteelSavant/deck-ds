import {init_usdpl, target_usdpl, init_embedded, call_backend, init_tr} from "usdpl-front";

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

