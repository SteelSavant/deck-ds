import { ReactElement, useRef } from 'react';
import { IconForTarget } from '../../components/IconForTarget';
import { useAppState } from '../../context/appContext';
import useLaunchActions from '../../hooks/useLaunchActions';
import { logger } from '../../util/log';

interface PrimaryPlayButtonProps {
    deckDSGameModeSentinel: 'sentinel';
    playButton: any;
}

export default function PrimaryPlayButton({
    playButton,
}: PrimaryPlayButtonProps): ReactElement {
    const { appDetails, appProfile, useAppTarget } = useAppState();
    const launchActions = useLaunchActions(appDetails);
    // Store the original button onclick/icon
    const buttonRef = useRef(playButton.props.children[1]);
    const launchRef = useRef(playButton.props.onClick);

    const action = appProfile?.isOk
        ? launchActions.find(
              (a) => a.profileId == appProfile.data.default_profile,
          ) ?? launchActions[0]
        : null;

    const target = useAppTarget({
        isPrimary: true,
        profileId: action?.profileId ?? null,
    });

    logger.trace(
        'primary play loading:',
        'ad:',
        appDetails,
        'ap:',
        appProfile,
        'la:',
        launchActions,
        'a:',
        action,
        't',
        target,
    );

    const onLaunch = action?.targets?.find((t) => t.target === target)?.action;

    logger.debug(
        'patching play button with target: ',
        target,
        'action:',
        action,
        'onLaunch:',
        onLaunch,
    );

    const children = playButton.props.children as any[];

    if (target && onLaunch) {
        children[1] = <IconForTarget target={target} />;
        playButton.props.onClick = onLaunch;
    } else {
        children[1] = buttonRef.current;
        playButton.props.onClick = launchRef.current;
    }

    return playButton;
}
