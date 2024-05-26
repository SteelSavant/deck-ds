import { useEffect } from 'react';
import { useAppState } from '../context/appContext';

const useEnsureAppOverridePipeline = (profileId: string): void => {
    const { appDetails, reifiedPipelines, loadProfileOverride } = useAppState();

    useEffect(() => {
        if (!reifiedPipelines[profileId] && appDetails) {
            loadProfileOverride(appDetails?.appId, profileId);
        }
    }, [reifiedPipelines[profileId]]);
};

export default useEnsureAppOverridePipeline;
