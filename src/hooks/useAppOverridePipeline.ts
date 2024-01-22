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


    const [pipeline, setPipeline] = useState<Loading<PipelineDefinition | null>>(initial);

    useEffect(() => {
        let active = true;

        if (!pipeline) {
            (async function load() {

                const res = await getDefaultAppOverrideForProfileRequest({
                    profile_id: profileId
                });

                if (!active) {
                    return;
                }

                setPipeline(res.map((v) => v.pipeline ?? null))
            })();
        }

        return () => { active = false; };

    }, [appProfile?.id, appProfile?.overrides[profileId], profileId]);

    return pipeline
}

export default useAppOverridePipeline;