import { useEffect } from "react";
import { useAppState } from "../context/appContext";


const useEnsureAppOverridePipeline = (profileId: string | null): void => {
    const { appDetails, reifiedPipelines, loadProfileOverride } = useAppState();

    const nonNullId = profileId ??= 'noprofile';


    useEffect(() => {
        profileId ??= 'noprofile';

        if (!reifiedPipelines[nonNullId] && appDetails) {
            loadProfileOverride(appDetails?.appId, profileId);
        }
    }, [appDetails?.appId, reifiedPipelines[nonNullId]]);
}

export default useEnsureAppOverridePipeline;


