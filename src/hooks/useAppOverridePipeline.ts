import _ from "lodash";
import { useEffect, useState } from "react";
import { PipelineDefinition, getDefaultAppOverrideForProfileRequest } from "../backend";
import { AppProfile } from "../types/backend_api";
import { Loading } from "../util/loading";
import { Ok } from "../util/result";

const useAppOverridePipeline = (appProfile: AppProfile | null, profileId: string): Loading<PipelineDefinition | null> => {
    const override = appProfile?.overrides[profileId];
    const initial: Loading<PipelineDefinition> = override
        ? Ok(override)
        : null;

    console.log('new override pipeline:', initial?.data);

    const [pipeline, setPipeline] = useState<Loading<PipelineDefinition | null>>(initial);

    console.log('override pipeline:', pipeline);

    useEffect(() => {
        console.log('checking initial effect');
        const newData = initial?.isOk && !pipeline?.isOk;
        const badData = initial?.isOk && pipeline?.isOk && !_.isEqual(initial.data, pipeline.data);
        if (newData || badData) {
            console.log('setting initial effect');

            setPipeline(initial);
        }
    }, [appProfile?.id, appProfile?.overrides[profileId], profileId]);

    useEffect(() => {
        console.log('checking db effect');

        let active = true;

        if (!pipeline) {
            (async function load() {
                const res = await getDefaultAppOverrideForProfileRequest({
                    profile_id: profileId
                });

                if (!active) {
                    return;
                }
                console.log('setting db effect');

                setPipeline(res.map((v) => v.pipeline ?? null))
            })();
        }

        return () => { active = false; };

    }, [appProfile?.id, appProfile?.overrides[profileId], profileId]);

    return pipeline
}

export default useAppOverridePipeline;