import { DialogButton, Focusable, showModal } from 'decky-frontend-lib';
import { ReactElement } from 'react';
import { FaExclamationTriangle } from 'react-icons/fa';
import { DependencyError } from '../backend';
import ConfigErrorModal from './ConfigErrorModal';

export default function ConfigErrorWarning({
    errors,
}: {
    errors: DependencyError[];
}): ReactElement | null {
    if (!errors || errors.length <= 0) {
        return null;
    }

    const onClick = () => {
        showModal(<ConfigErrorModal errors={errors} />);
    };

    return (
        <Focusable>
            <DialogButton
                onClick={onClick}
                onOKButton={onClick}
                style={{
                    width: 'fit-content',
                    minWidth: 'fit-content',
                    height: 'fit-content',
                    padding: '10px 12px',
                }}
            >
                <FaExclamationTriangle color="yellow" />
            </DialogButton>
        </Focusable>
    );
}
