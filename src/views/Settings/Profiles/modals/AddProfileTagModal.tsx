import { ConfirmModal, Dropdown, Focusable } from '@decky/ui';
import { ReactElement, useState } from 'react';
import { logger } from '../../../../util/log';
import { Result } from '../../../../util/result';

export default function CreateProfileTagModal({
    onSave,
    closeModal,
}: {
    onSave: (
        tag: string,
    ) => Promise<Result<void, string>> | Result<void, string>;
    closeModal?: () => void;
}): ReactElement {
    const [selected, setSelected] = useState<string | null>(null);
    return (
        <ConfirmModal
            strTitle="Add Collection"
            onOK={async () => {
                if (selected != null) {
                    const res = await onSave(selected);

                    if (res.isOk) {
                        closeModal!();
                    } else {
                        logger.toastWarn('Failed to create profile:', res.err);
                    }
                }
            }}
            onCancel={closeModal}
            onEscKeypress={closeModal}
        >
            <Focusable>
                <Dropdown
                    selectedOption={selected}
                    rgOptions={collectionStore.userCollections.map((uc) => {
                        return {
                            label: uc.displayName,
                            data: uc.id,
                        };
                    })}
                    onChange={(value) => {
                        setSelected(value.data);
                    }}
                />
            </Focusable>
        </ConfirmModal>
    );
}
