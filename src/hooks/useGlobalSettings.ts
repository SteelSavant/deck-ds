import { useEffect, useState } from "react";
import { getSettings, setSettings } from "../backend";
import { GlobalConfig } from "../types/backend_api";
import { Loading } from "../util/loading";
import { Err, Ok } from "../util/result";

const useGlobalSettings = () => {
    const [state, setState] = useState<Loading<GlobalConfig>>();

    useEffect(() => {
        let active = true;

        if (!state) {
            (async function load() {
                const res = await getSettings();

                if (!active) {
                    return;
                }

                setState(res.map((v) => v.global_settings));
            })();
        }

        return () => { active = false; };
    }, [state]);

    const updateSettings = async (settings: GlobalConfig) => {
        setState(Ok(settings));
        let res = await setSettings({
            global_settings: settings
        })

        if (!res.isOk) {
            setState(Err(res.err))
        }
    }

    return { settings: state, updateSettings };
}

export default useGlobalSettings;