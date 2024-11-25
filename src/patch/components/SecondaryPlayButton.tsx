import { DialogButton, Focusable } from '@decky/ui';
import { ReactElement, useState } from 'react';
import { IconForTarget } from '../../components/IconForTarget';
import { logger } from '../../util/log';
import useActionButtonProps from '../hooks/useActionButtonProps';

interface SecondaryPlayButtonProps {
    deckDSDesktopSentinel: 'sentinel';
}

// TODO::ideally, if
// - the secondary action is gamemode, and
// - gamemode is not a target
// then this would display the icon from the normal play button, and run its on clicked/pressed function when pressed
export default function SecondaryPlayButton({}: SecondaryPlayButtonProps): ReactElement | null {
    const { target, onLaunch, selectedClientId } = useActionButtonProps({
        isPrimary: false,
    });
    const [isFocused, setIsFocused] = useState(false);

    function setFocusChecked(shouldFocus: boolean) {
        if (isFocused !== shouldFocus) {
            setIsFocused(shouldFocus);
        }
    }

    const vPadding = 14;
    const wPadding = 15;

    logger.debug(
        'patching secondary button with target: ',
        target,
        'onLaunch:',
        onLaunch,
        'clientid:',
        selectedClientId,
    );

    return selectedClientId === '0' && // hack to ensure we're not using streaming
        target &&
        onLaunch ? (
        <Focusable
            onFocus={() => {
                setFocusChecked(true);
            }}
            onBlur={() => {
                setFocusChecked(false);
            }}
        >
            <DialogButton
                // I would be thrilled if this matched the other buttons exactly, but alas...
                style={{
                    minWidth: 0,
                    paddingLeft: wPadding,
                    paddingRight: wPadding,
                    paddingTop: vPadding + 3,
                    paddingBottom: vPadding - 3,
                    backgroundColor: isFocused ? 'white' : '#ACB2C924',
                }}
                onClick={onLaunch}
                onOKButton={onLaunch}
                onOKActionDescription={`Launch ${target}`}
            >
                <IconForTarget target={target} />
            </DialogButton>
        </Focusable>
    ) : null;
}
