import { Button } from 'decky-frontend-lib';
import { ReactElement, useEffect, useRef } from 'react';
import { IconForTarget } from '../../components/IconForTarget';
import { useAppState } from '../../context/appContext';
import useAppTarget from '../../hooks/useAppTarget';
import useLaunchActions from '../../hooks/useLaunchActions';
import { logger } from '../../util/log';

interface PrimaryPlayButtonProps {
    deckDSGameModeSentinel: 'sentinel';
    playButton: any;
}

export default function PrimaryPlayButton({
    playButton,
}: PrimaryPlayButtonProps): ReactElement {
    const { appDetails, appProfile } = useAppState();
    const launchActions = useLaunchActions(appDetails);
    const ref = useRef<HTMLDivElement>(null);

    const action = appProfile?.isOk
        ? launchActions.find(
              (a) => a.profile.id == appProfile.data.default_profile,
          ) ?? launchActions[0]
        : null;

    const target = useAppTarget({
        isPrimary: true,
        profileId: action?.profile.id ?? null,
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

    useEffect(() => {
        setTimeout(() => {
            ref?.current?.focus();
        }, 750);
    }, []);

    logger.debug(
        'patching play button with target: ',
        target,
        'action:',
        action,
        'onLaunch:',
        onLaunch,
    );

    const playText = (playButton.props.children as any[])[2] ?? <div>Play</div>;

    return target && onLaunch ? (
        <Button
            // I would be thrilled if this matched the actual play button (including CSS loader styling), but with a custom icon + action, but alas...
            // I genuinely don't know how to style things properly.
            ref={ref}
            onClick={onLaunch}
            onOKButton={onLaunch}
            onOKActionDescription={`Launch ${target}`}
            className={playButton.props.className}
        >
            <div
                style={{
                    alignContent: 'center',
                    justifyContent: 'left',
                    display: 'flex',
                    flexDirection: 'row',
                }}
            >
                <IconForTarget target={target} />
                {playText}
            </div>
        </Button>
    ) : (
        playButton
    );
}
