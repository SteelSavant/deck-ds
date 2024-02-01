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
    const { appDetails } = useAppState();
    const launchActions = useLaunchActions(appDetails)

    const target = useAppTarget({ isPrimary: false });
    const onLaunch = launchActions[0]?.targets?.find((t) => t.target === target)?.action;
    const vPadding = 14;
    const wPadding = 17;

    return target && onLaunch ?
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
            <IconForTarget target={target} />
        </DialogButton>
        : <div />;
}