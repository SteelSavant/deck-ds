import { DialogButton } from "decky-frontend-lib";
import { ReactElement } from "react";
import { IconForTarget } from "../../components/IconForTarget";
import { useAppState } from "../../context/appContext";
import useAppTarget from "../../hooks/useAppTarget";
import useLaunchActions from "../../hooks/useLaunchActions";


interface SecondaryPlayButtonProps {
    deckDSDesktopSentinel: 'sentinel'
}

export default function SecondaryPlayButton({ }: SecondaryPlayButtonProps): ReactElement {
    const { appDetails, appProfile } = useAppState();
    const launchActions = useLaunchActions(appDetails);

    const action = appProfile?.isOk
        ? launchActions.find((a) =>
            a.profile.id == appProfile.data.default_profile)
        ?? launchActions[0]
        : null;

    const vPadding = 14;
    const wPadding = 17;

    const target = useAppTarget({ isPrimary: false, profileId: action?.profile.id });

    let onLaunch = launchActions[0]?.targets?.find((t) => t.target === target)?.action;
    if (target === 'Gamemode' && appDetails) {
        onLaunch ??= () => SteamClient.Apps.RunGame(appDetails.gameId ?? (appDetails.appId.toString()), "", -1, 100);
    }

    console.log('DeckDS: patching secondary button with target: ',
        target, 'action:',
        action,
        'onLaunch:',
        onLaunch
    );

    return target && onLaunch
        ? <DialogButton
            // I would be thrilled if this matched the other buttons exactly, but alas...
            style={{
                minWidth: 0,
                paddingLeft: wPadding,
                paddingRight: wPadding,
                paddingTop: vPadding,
                paddingBottom: vPadding,
            }}
            onClick={onLaunch}
            onOKButton={onLaunch}
        >
            <IconForTarget target={target} />
        </DialogButton>
        : <div />;
}