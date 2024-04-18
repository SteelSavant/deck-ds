import { ConfirmModal, Dropdown } from "decky-frontend-lib";
import { ReactElement, useState } from "react";
import HandleLoading from "../../../../components/HandleLoading";
import useToplevel from "../../../../hooks/useToplevel";
import { ToplevelInfo } from "../../../../types/backend_api";

export default function AddToplevelActionModal({ onSave, closeModal }: { onSave: (info: ToplevelInfo) => void, closeModal?: () => void }): ReactElement {
    const toplevel = useToplevel();
    const [state, setState] = useState<ToplevelInfo | null>(null);

    return <ConfirmModal
        strTitle="Add Action"
        onOK={() => {
            if (state) {
                onSave(state);
            } else if (toplevel?.isOk && toplevel.data[0]) {
                onSave(toplevel.data[0]);
            }
            closeModal!()
        }}
        onCancel={closeModal}
        onEscKeypress={closeModal}
    >
        <HandleLoading
            value={toplevel}
            onOk={(toplevel) => {
                return <Dropdown
                    selectedOption={state ?? toplevel[0]}
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
