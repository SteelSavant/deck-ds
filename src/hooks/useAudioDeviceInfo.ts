import { useEffect, useState } from 'react';
import { getAudioDeviceInfo } from '../backend';
import { GetAudioDeviceInfoResponse } from '../types/backend_api';
import { Loading } from '../util/loading';

const useAudioDeviceInfo = (): Loading<GetAudioDeviceInfoResponse> => {
    const [result, setResult] =
        useState<Loading<GetAudioDeviceInfoResponse>>(null);

    useEffect(() => {
        let active = true;

        if (result === null) {
            (async function load() {
                const res = await getAudioDeviceInfo();

                if (!active) {
                    return;
                }

                setResult(
                    res.map((v) => {
                        return v;
                    }),
                );
            })();
        }

        return () => {
            active = false;
        };
    });

    return result;
};

export default useAudioDeviceInfo;
