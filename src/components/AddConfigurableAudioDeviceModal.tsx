import { ConfirmModal, Dropdown } from "decky-frontend-lib";
import { ReactElement, useState } from "react";
import { AudioDeviceInfo } from "../types/backend_api";

export function AddConfigurableAudioDeviceModal({ devices, onSave, closeModal }: {
    devices: AudioDeviceInfo[],
    onSave: (device: AudioDeviceInfo) => void,
    closeModal?: () => void,
}): ReactElement {
    const [device, setDevice] = useState(devices[0]);


    return <ConfirmModal
        onOK={() => {
            onSave(device);
            closeModal!();
        }}
        onCancel={closeModal}
        onEscKeypress={closeModal}
    >
        <Dropdown
            selectedOption={device}
            rgOptions={devices.map((d) => {
                return {
                    label: d.description ?? d.name,
                    data: d,
                }
            })}
            onChange={(data) => {
                setDevice(data.data);
            }}
        />
    </ConfirmModal>
}