import { useEffect, useState } from 'react';
import { getSecondaryAppInfo } from '../backend';
import { GetSecondaryAppInfoResponse } from '../types/backend_api';
import { Loading } from '../util/loading';

const useSecondaryAppInfo = (): Loading<GetSecondaryAppInfoResponse> => {
    const [result, setResult] =
        useState<Loading<GetSecondaryAppInfoResponse>>(null);

    useEffect(() => {
        let active = true;

        if (result === null) {
            (async function load() {
                const res = await getSecondaryAppInfo();

                if (!active) {
                    return;
                }

                setResult(res);
            })();
        }

        return () => {
            active = false;
        };
    });

    return result;
};

export default useSecondaryAppInfo;
