import _ from 'lodash';
import { useEffect, useRef } from 'react';
import { useAppState } from '../../context/appContext';
import useLaunchActions from '../../hooks/useLaunchActions';
import { PipelineTarget } from '../../types/backend_api';

export type PipelineTargetOrNative = PipelineTarget | 'Native';

export interface ActionButtonProps {
    target: PipelineTargetOrNative | null;
    onLaunch: (() => Promise<void>) | undefined;
    selectedClientId: string | undefined;
}

const useActionButtonProps = ({
    isPrimary,
}: {
    isPrimary: boolean;
}): ActionButtonProps => {
    let cachedProps = useRef<ActionButtonProps>();
    let cachedState = useRef<any>();

    const {
        appDetails,
        appProfile,
        useAppTarget,
        ensureSelectedClientUpdated,
    } = useAppState();

    const launchActions = useLaunchActions(appDetails);

    const action = appProfile?.isOk
        ? launchActions.find(
              (a) => a.profileId == appProfile.data.default_profile,
          ) ?? launchActions[0]
        : null;

    let target: PipelineTargetOrNative | null = useAppTarget({
        isPrimary,
        profileId: action?.profileId ?? null,
    });

    // Hack to ensure we have the correct selected_clientid
    useEffect(() => {
        for (const timeout of [100, 200, 500, 1000, 2000, 5000, 10000]) {
            setTimeout(() => ensureSelectedClientUpdated(), timeout);
        }
    }, [appDetails?.selected_clientid]);

    const state = {
        appDetails,
        appProfile,
        target,
    };

    if (_.isEqual(state, cachedState)) {
        return cachedProps.current!;
    }

    const selectedClientId = appDetails?.selected_clientid;

    let onLaunch = action?.targets?.find((t) => t.target === target)?.action;
    if (target === 'Gamemode' && appDetails) {
        target = 'Native';
        onLaunch ??= () =>
            SteamClient.Apps.RunGame(
                appDetails.gameId ?? appDetails.appId.toString(),
                '',
                -1,
                100,
            );
    }

    const props = {
        target,
        onLaunch,
        selectedClientId,
    };

    cachedState.current = state;
    cachedProps.current = props;

    return props;
};

export default useActionButtonProps;
