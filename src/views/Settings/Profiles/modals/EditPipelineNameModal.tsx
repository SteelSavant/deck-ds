import { ConfirmModal, TextField } from "decky-frontend-lib";
import { ReactElement, useState } from "react";
import { Pipeline } from "../../../../types/backend_api";

export default function EditProfileNameModal({ pipeline, onSave, closeModal }: { pipeline: Pipeline, onSave: (name: string) => void, closeModal?: () => void }): ReactElement {
    const [name, setName] = useState(pipeline.name);



    return <ConfirmModal strTitle="Edit Profile Name"
        strOKButtonText='Save' strCancelButtonText='Cancel'
        onOK={() => {
            onSave(name);
            closeModal!();
        }}
        onCancel={closeModal}
        onEscKeypress={closeModal}
    >
        <TextField label='Name' value={name} onChange={(value) => {
            setName(value?.target.value ?? '');
        }} />
    </ConfirmModal>
}