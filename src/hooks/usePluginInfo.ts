import { call } from '@decky/api';
import { useEffect, useState } from 'react';

interface PluginInfo {
    name: string;
    version: string;
}

function usePluginInfo(): PluginInfo | null {
    let [state, setState] = useState<PluginInfo | null>(null);

    useEffect(() => {
        let active = true;

        if (state === null) {
            (async function load() {
                const res = await call<[], PluginInfo>('plugin_info');

                if (!active) {
                    return;
                }

                setState(res);
            })();
        }

        return () => {
            active = false;
        };
    });

    return state;
}

export default usePluginInfo;
