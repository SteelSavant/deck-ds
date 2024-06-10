import { ConfirmModal, TextField } from 'decky-frontend-lib';
import { ReactElement, useState } from 'react';
import { PipelineDefinition } from '../../../../backend';
import { useServerApi } from '../../../../context/serverApiContext';
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
    const serverApi = useServerApi();
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
                    logger.toastWarn(
                        serverApi.toaster,
                        'Failed to update profile name:',
                        res.err,
                    );
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
