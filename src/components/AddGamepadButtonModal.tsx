import { ConfirmModal, Dropdown } from 'decky-frontend-lib';
import { ReactElement, useState } from 'react';
import { GamepadButtonSelection } from '../backend';
import { useServerApi } from '../context/serverApiContext';
import { logger } from '../util/log';
import { Result } from '../util/result';

export function AddGamepadButtonModal({
    buttons,
    onSave,
    closeModal,
}: {
    buttons: GamepadButtonSelection[];
    onSave: (
        button: GamepadButtonSelection,
    ) => Promise<Result<void, string>> | Result<void, string>;
    closeModal?: () => void;
}): ReactElement {
    const serverApi = useServerApi();
    const [button, setButton] = useState<GamepadButtonSelection>(buttons[0]);
    const [done, setDone] = useState(false);

    return (
        <ConfirmModal
            onOK={async () => {
                if (!done) {
                    setDone(true);
                    const res = await onSave(button);
                    if (res.isOk) {
                        closeModal!();
                    } else {
                        logger.toastWarn(serverApi.toaster, res.err);
                    }
                }
            }}
            onCancel={closeModal}
            onEscKeypress={closeModal}
        >
            <Dropdown
                selectedOption={button}
                rgOptions={buttons.map((d) => {
                    return {
                        label: d,
                        data: d,
                    };
                })}
                onChange={(data) => {
                    setButton(data.data);
                }}
            />
        </ConfirmModal>
    );
}
