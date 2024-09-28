import { ConfirmModal, TextField } from '@decky/ui';
import { ReactElement, useState } from 'react';
import { PipelineDefinition } from '../../../../backend';
import { logger } from '../../../../util/log';
import { Result } from '../../../../util/result';

export default function EditProfileNameModal({
    pipeline,
    onSave,
    closeModal,
}: {
    pipeline: PipelineDefinition;
    onSave: (
        name: string,
    ) => Promise<Result<void, string>> | Result<void, string>;
    closeModal?: () => void;
}): ReactElement {
    const [name, setName] = useState(pipeline.name);

    return (
        <ConfirmModal
            strTitle="Edit Profile Name"
            strOKButtonText="Save"
            strCancelButtonText="Cancel"
            onOK={async () => {
                const res = await onSave(name);

                if (res.isOk) {
                    closeModal!();
                } else {
                    logger.toastWarn('Failed to update profile name:', res.err);
                }
            }}
            onCancel={closeModal}
            onEscKeypress={closeModal}
        >
            <TextField
                label="Name"
                value={name}
                onChange={(value) => {
                    setName(value?.target.value ?? '');
                }}
            />
        </ConfirmModal>
    );
}
