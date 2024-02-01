import { Button } from "decky-frontend-lib";
import { ReactElement } from "react";
import HandleLoading from "../../components/HandleLoading";
import { IconForTarget } from "../../components/IconForTarget";
import { useAppState } from "../../context/appContext";
import useAppTarget from "../../hooks/useAppTarget";
import useLaunchActions from "../../hooks/useLaunchActions";


interface PrimaryPlayButtonProps {
    deckDSGameModeSentinel: 'sentinel'
    playButton: any
}

export default function PrimaryPlayButton({
    playButton,
}: PrimaryPlayButtonProps): ReactElement {
    const { appDetails, appProfile } = useAppState();
    const launchActions = useLaunchActions(appDetails);
    const target = useAppTarget({ isPrimary: true });

    return <HandleLoading
        value={appProfile}
        onOk={(appProfile) => {
            const action = launchActions.find((a) => a.profile.id == appProfile?.default_profile)
                ?? launchActions[0];
            const onLaunch = action?.targets?.find((t) => t.target === target)?.action;

            return target && onLaunch ? (
                <Button
                    // I would be thrilled if this matched the actual play button (including CSS loader styling), but with a custom icon, but alas...
                    className="basicappdetailssectionstyler_AppActionButton_QsZdW appactionbutton_PlayButtonContainer_1FnJ6 appactionbutton_Green_3cI5T Panel Focusable gpfocuswithin"
                    onClick={onLaunch}
                    onOKButton={onLaunch}
                >
                    <div
                        className="appactionbutton_PlayButton_3ydig appactionbutton_ButtonChild_2AzIX Focusable gpfocus gpfocuswithin"
                    >
                        <IconForTarget target={target} />
                        Play
                    </div>
                </Button>
            ) : playButton;
        }}
        onErr={(_) => playButton} //TODO:: maybe toast error?
        onLoading={() => playButton}
    />

}