import { useEffect, useState } from 'react';
import { GlobalConfig, getSettings, setSettings } from '../backend';
import { Loading } from '../util/loading';
import { Ok } from '../util/result';

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

        return () => {
            active = false;
        };
    }, [state]);

    const updateSettings = async (settings: GlobalConfig) => {
        let res = await setSettings({
            global_settings: settings,
        });

        if (res.isOk) {
            setState(Ok(settings));
        }

        return res;
    };

    return { settings: state, updateSettings };
};

export default useGlobalSettings;
