import { ConfirmModal, Dropdown } from "decky-frontend-lib";
import { ReactElement, useState } from "react";
import { GamepadButtonSelection } from "../backend";

export function AddGamepadButtonModal({ buttons, onSave, closeModal }: {
    buttons: GamepadButtonSelection[],
    onSave: (button: GamepadButtonSelection) => void,
    closeModal?: () => void,
}): ReactElement {
    const [button, setButton] = useState<GamepadButtonSelection>(buttons[0]);
    const [done, setDone] = useState(false);

    return <ConfirmModal
        onOK={() => {
            if (!done) {
                setDone(true);
                onSave(button);
            }
            closeModal!();
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
                }
            })}
            onChange={(data) => {
                setButton(data.data);
            }}
        />
    </ConfirmModal>
}