import { DialogButton, Focusable } from '@decky/ui';
import { ReactElement, useState } from 'react';
import { IconForTarget } from '../../components/IconForTarget';
import { useAppState } from '../../context/appContext';
import useAppTarget from '../../hooks/useAppTarget';
import useLaunchActions from '../../hooks/useLaunchActions';
import { logger } from '../../util/log';

interface SecondaryPlayButtonProps {
    deckDSDesktopSentinel: 'sentinel';
}

export default function SecondaryPlayButton({}: SecondaryPlayButtonProps): ReactElement | null {
    const { appDetails, appProfile } = useAppState();
    const launchActions = useLaunchActions(appDetails);
    const [isFocused, setIsFocused] = useState(false);

    const action = appProfile?.isOk
        ? launchActions.find(
              (a) => a.profile.id == appProfile.data.default_profile,
          ) ?? launchActions[0]
        : null;

    const vPadding = 14;
    const wPadding = 15;

    const target = useAppTarget({
        isPrimary: false,
        profileId: action?.profile.id ?? null,
    });

    let onLaunch = launchActions[0]?.targets?.find(
        (t) => t.target === target,
    )?.action;
    if (target === 'Gamemode' && appDetails) {
        onLaunch ??= () =>
            SteamClient.Apps.RunGame(
                appDetails.gameId ?? appDetails.appId.toString(),
                '',
                -1,
                100,
            );
    }

    logger.debug(
        'patching secondary button with target: ',
        target,
        'action:',
        action,
        'onLaunch:',
        onLaunch,
    );

    return target && onLaunch ? (
        <Focusable
            onFocus={() => {
                setIsFocused(true);
            }}
            onBlur={() => {
                setIsFocused(false);
            }}
        >
            <DialogButton
                // I would be thrilled if this matched the other buttons exactly, but alas...
                style={{
                    minWidth: 0,
                    paddingLeft: wPadding,
                    paddingRight: wPadding,
                    paddingTop: vPadding + 3,
                    paddingBottom: vPadding - 3,
                    backgroundColor: isFocused ? 'white' : '#ACB2C924',
                }}
                onClick={onLaunch}
                onOKButton={onLaunch}
                onOKActionDescription={`Launch ${target}`}
            >
                <IconForTarget target={target} />
            </DialogButton>
        </Focusable>
    ) : null;
}
