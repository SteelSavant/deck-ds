import { Button } from "decky-frontend-lib";
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
        <Button
            className="basicappdetailssectionstyler_AppActionButton_QsZdW appactionbutton_PlayButtonContainer_1FnJ6 appactionbutton_Green_3cI5T Panel Focusable gpfocuswithin"
            // I would be thrilled if this matched the actual play button (including CSS loader styling), but with a custom icon, but alas...
            onClick={onLaunch}
            onOKButton={onLaunch}
        >
            <div
                className="appactionbutton_PlayButton_3ydig appactionbutton_ButtonChild_2AzIX Focusable gpfocus gpfocuswithin"
            >
                <FaGamepad />
                Play
            </div>
        </Button>
    ) : playButton;
}

{/* <div navigator="[object Object]" instance="[object Object]" 
class="
basicappdetailssectionstyler_AppActionButton_QsZdW 
appactionbutton_PlayButtonContainer_1FnJ6 
appactionbutton_Green_3cI5T Panel Focusable gpfocuswithin">
<div class="appactionbutton_PlayButton_3ydig 
appactionbutton_ButtonChild_2AzIX Focusable gpfocus gpfocuswithin" tabindex="0"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 36 36" fill="none"><path d="M7.5 32.135a1 1 0 0 1-1.5-.866V4.73a1 1 0 0 1 1.5-.866l22.999 13.269a1 1 0 0 1 0 1.732l-23 13.269Z" fill="currentColor"></path></svg><div class="appactionbutton_ButtonText_33cnX">Play</div></div></div> */}
// appactionbutton_ButtonChild_2AzIX 

// appactionbutton_PlayButton_3ydig
// appactionbutton_ButtonText_33cnX