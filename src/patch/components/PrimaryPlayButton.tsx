import { DialogButton } from "decky-frontend-lib";
import { ReactElement } from "react";
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

    const action = appProfile?.isOk
        ? launchActions.find((a) =>
            a.profile.id == appProfile.data.default_profile)
        ?? launchActions[0]
        : null;

    const target = useAppTarget({ isPrimary: true, profileId: action?.profile.id });
    console.log('primary play loading:',
        'ad:', appDetails,
        'ap:', appProfile,
        'la:', launchActions,
        'a:', action,
        't', target,
    )

    const onLaunch = action?.targets?.find((t) => t.target === target)?.action;

    console.log('DeckDS: patching play button with target: ',
        target, 'action:',
        action,
        'onLaunch:',
        onLaunch
    );

    return target && onLaunch ? (
        <DialogButton
            // I would be thrilled if this matched the actual play button (including CSS loader styling), but with a custom icon, but alas...
            // className="basicappdetailssectionstyler_AppActionButton_QsZdW appactionbutton_PlayButtonContainer_1FnJ6 appactionbutton_Green_3cI5T Panel Focusable gpfocuswithin"
            onClick={onLaunch}
            onOKButton={onLaunch}
        >
            <div
            // className="appactionbutton_PlayButton_3ydig appactionbutton_ButtonChild_2AzIX Focusable gpfocus gpfocuswithin"
            >
                <IconForTarget target={target} />
                Play
            </div>
        </DialogButton>
    ) : playButton;


}