import { ServerAPI } from "decky-frontend-lib";
import { useEffect, useState } from "react";

interface PluginInfo {
    name: string,
    version: string,
}


function usePluginInfo(serverApi: ServerAPI): PluginInfo | null {
    let [state, setState] = useState<PluginInfo | null>(null);

    useEffect(() => {
        let active = true;

        if (state === null) {
            (async function load() {
                const res = await serverApi.callPluginMethod("plugin_info", {});

                if (!active) {
                    return;
                }

                if (!res.success) {
                    console.log("error fetching plugin info:", res.result);
                    return;
                }

                setState(res.result as PluginInfo);
            })();
        }

        return () => { active = false; };
    });


    return state;
}

export default usePluginInfo;