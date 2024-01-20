import { DialogButton } from "decky-frontend-lib";
import { ReactElement } from "react";
import { FaDesktop } from "react-icons/fa";
import { ShortAppDetails } from "../context/shortAppDetailsContext";
import useLaunchActions from "../hooks/useLaunchActions";


interface DesktopPlayButtonProps {
    deckDSDesktopSentinel: 'sentinel'
    appDetails: ShortAppDetails
}

export default function DesktopPlayButton({
    appDetails
}: DesktopPlayButtonProps): ReactElement {
    const launchActions = useLaunchActions(appDetails);

    console.log(launchActions);

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
            <FaDesktop />
        </DialogButton>
    ) : <div />;
}