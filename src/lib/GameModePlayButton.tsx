import { DialogButton } from "decky-frontend-lib";
import { ReactElement } from "react";
import { FaGamepad } from "react-icons/fa";
import { useShortAppDetailsState } from "../context/shortAppDetailsContext";
import useLaunchActions from "../hooks/useLaunchActions";


interface GameModePlayButtonProps {
    deckDSGameModeSentinel: 'sentinel'
    playButton: any
}

export default function GameModePlayButton({
    playButton,
}: GameModePlayButtonProps): ReactElement {
    const { appDetails } = useShortAppDetailsState();
    const launchActions = appDetails ? useLaunchActions(appDetails) : [];

    console.log(launchActions);

    const onLaunch = launchActions[0]?.targets?.find((t) => t.target === 'Gamemode')?.action;


    return onLaunch ? (
        <DialogButton
            // I would be thrilled if this matched the actual play button (including CSS loader styling), but with a custom icon, but alas...
            onClick={onLaunch}
            onOKButton={onLaunch}
        >
            <FaGamepad />
            Play
        </DialogButton>
    ) : playButton;
}