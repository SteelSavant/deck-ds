import { ConfirmModal, Dropdown, Focusable } from "decky-frontend-lib";
import { ReactElement, useState } from "react";

declare global {
    let collectionStore: CollectionStore;
    let appStore: AppStore;
}

export default function CreateProfileTagModal({ onSave, closeModal }: {
    onSave: (tag: string) => void, closeModal?: () => void,
}): ReactElement {
    const [selected, setSelected] = useState<string | null>(null)
    return (
        <ConfirmModal strTitle='Add Collection'
            onOK={() => {
                if (selected != null) {
                    onSave(selected);
                }
                closeModal!();
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
                            data: uc.id
                        }
                    })}
                    onChange={(value) => {
                        setSelected(value.data)
                    }}
                />
            </Focusable>
        </ConfirmModal>
    )
}