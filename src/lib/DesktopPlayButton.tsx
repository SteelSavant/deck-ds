import { DialogButton } from "decky-frontend-lib";
import { ReactElement } from "react";
import { IconForTarget } from "../components/IconForTarget";
import { useAppState } from "../context/appContext";
import useLaunchActions from "../hooks/useLaunchActions";


interface DesktopPlayButtonProps {
    deckDSDesktopSentinel: 'sentinel'
}

export default function DesktopPlayButton({ }: DesktopPlayButtonProps): ReactElement {
    const { appDetails } = useAppState();
    const launchActions = appDetails ? useLaunchActions(appDetails) : [];

    const onLaunch = launchActions[0]?.targets?.find((t) => t.target === 'Desktop')?.action;
    const vPadding = 14;
    const wPadding = 17;

    return onLaunch ? (
        <DialogButton
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
            <IconForTarget target="Desktop" />
        </DialogButton>
    ) : <div />;
}