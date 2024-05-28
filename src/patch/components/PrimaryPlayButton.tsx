import { DialogButton, Focusable } from 'decky-frontend-lib';
import { ReactElement, useEffect, useRef, useState } from 'react';
import { IconForTarget } from '../../components/IconForTarget';
import { useAppState } from '../../context/appContext';
import useAppTarget from '../../hooks/useAppTarget';
import useLaunchActions from '../../hooks/useLaunchActions';
import { logger } from '../../util/log';

interface PrimaryPlayButtonProps {
    deckDSGameModeSentinel: 'sentinel';
    hasStream: boolean;
    playButton: any;
}

export default function PrimaryPlayButton({
    playButton,
    hasStream,
}: PrimaryPlayButtonProps): ReactElement {
    const { appDetails, appProfile } = useAppState();
    const launchActions = useLaunchActions(appDetails);
    const [isFocused, setIsFocused] = useState(false);
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
        'DeckDS: patching play button with target: ',
        target,
        'action:',
        action,
        'onLaunch:',
        onLaunch,
    );

    const width = '100%';
    const height = '100%';

    const okColor = '#70d61d';

    return target && onLaunch ? (
        <Focusable
            onFocus={() => {
                setIsFocused(true);
            }}
            onBlur={() => {
                setIsFocused(false);
            }}
            style={{
                width: width,
                height: height,
            }}
        >
            <DialogButton
                // I would be thrilled if this matched the actual play button (including CSS loader styling), but with a custom icon + action, but alas...
                // I genuinely don't know how to style things properly.
                ref={ref}
                onClick={onLaunch}
                onOKButton={onLaunch}
                onOKActionDescription={`Launch ${target}`}
                style={{
                    borderTopRightRadius: hasStream ? 0 : undefined,
                    borderBottomRightRadius: hasStream ? 0 : undefined,
                    color: isFocused ? 'white' : undefined,
                    backgroundColor: isFocused ? okColor : 'transparent',
                    alignContent: 'center',
                    justifyContent: 'left',
                    minWidth: '210px',
                    maxWidth: width,
                    width: width,
                    height: height,
                    display: 'flex',
                    flexDirection: 'row',
                    paddingTop: '12px',
                }}
            >
                <IconForTarget target={target} />
                <div style={{ width: 15 }} />
                Play
            </DialogButton>
        </Focusable>
    ) : (
        playButton
    );
}
