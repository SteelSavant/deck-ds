import { ReactElement, useEffect, useRef, useState } from 'react';
import { IconForTarget } from '../../components/IconForTarget';
import { logger } from '../../util/log';
import useActionButtonProps from '../hooks/useActionButtonProps';

interface PrimaryPlayButtonProps {
    deckDSGameModeSentinel: 'sentinel';
    playButton: any;
}

export default function PrimaryPlayButton({
    playButton,
}: PrimaryPlayButtonProps): ReactElement {
    const { target, onLaunch } = useActionButtonProps({
        isPrimary: true,
    });
    const [patch, setPatch] = useState(!!(onLaunch && target)); // hack to force rerenders when necessary

    // Store the original button onclick/icon
    const buttonRef = useRef(playButton.props.children[1]);
    const launchRef = useRef(playButton.props.onClick);
    const keyRef = useRef(playButton.key);

    logger.debug(
        'patching play button with target: ',
        target,
        'onLaunch:',
        onLaunch,
    );

    useEffect(() => {
        const children = playButton.props.children as any[];
        const shouldPatch = !!(target && onLaunch);

        if (shouldPatch) {
            logger.trace('Using play target');
            children[1] = <IconForTarget target={target} />;
            playButton.props.onClick = onLaunch;
        } else {
            logger.trace('Using play original');
            children[1] = buttonRef.current;
            playButton.props.onClick = launchRef.current;
        }

        if (patch !== shouldPatch) {
            logger.trace('forcing primary play button rebuild...');
            playButton.key = shouldPatch ? 'patchedPlayButton' : keyRef.current;
            setPatch(shouldPatch);
        }

        return () => {
            children[1] = buttonRef.current;
            playButton.props.onClick = launchRef.current;
            playButton.ref = keyRef.current;
        };
    }, [target, onLaunch]);

    return playButton;
}
