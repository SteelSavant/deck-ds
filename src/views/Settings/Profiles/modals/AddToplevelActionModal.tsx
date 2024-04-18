import { ConfirmModal, Dropdown } from "decky-frontend-lib";
import { ReactElement, useState } from "react";
import HandleLoading from "../../../../components/HandleLoading";
import useToplevel from "../../../../hooks/useToplevel";
import { ToplevelInfo } from "../../../../types/backend_api";

export default function AddToplevelActionModal({ onSave, closeModal }: { onSave: (info: ToplevelInfo) => void, closeModal?: () => void }): ReactElement {
    const toplevel = useToplevel();
    const [state, setState] = useState<ToplevelInfo | null | undefined>(null);

    if (!state && toplevel?.isOk) {
        setState(toplevel.data[0])
    }

    return <ConfirmModal
        strTitle="Add Action" bAlertDialog={true}
        onOK={state
            ? () => {
                onSave(state)
                closeModal?.call([])
            }
            : undefined
        }
        onCancel={closeModal}
        onEscKeypress={closeModal}
    >
        <HandleLoading
            value={toplevel}
            onOk={(toplevel) => {
                return <Dropdown
                    selectedOption={state}
                    rgOptions={toplevel.map((v) => {
                        return {
                            label: v.name,
                            data: v
                        }
                    })}
                    onChange={(v) => {
                        setState(v.data)
                    }}
                />
            }}
        />
        <p>{state?.description}</p>
    </ConfirmModal >
}
