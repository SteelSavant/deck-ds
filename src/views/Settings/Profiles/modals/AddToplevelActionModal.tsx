import { ConfirmModal, Dropdown } from 'decky-frontend-lib';
import { ReactElement, useState } from 'react';
import HandleLoading from '../../../../components/HandleLoading';
import useToplevel from '../../../../hooks/useToplevel';
import { ToplevelInfo } from '../../../../types/backend_api';
import { logger } from '../../../../util/log';
import { Result } from '../../../../util/result';

export default function AddToplevelActionModal({
    onSave,
    closeModal,
}: {
    onSave: (
        info: ToplevelInfo,
    ) => Promise<Result<void, string>> | Result<void, string>;
    closeModal?: () => void;
}): ReactElement {
    const toplevel = useToplevel();
    const [state, setState] = useState<ToplevelInfo | null>(null);

    return (
        <ConfirmModal
            strTitle="Add Action"
            onOK={async () => {
                const saved = await (async function () {
                    if (state) {
                        return await onSave(state);
                    } else if (toplevel?.isOk && toplevel.data[0]) {
                        return await onSave(toplevel.data[0]);
                    } else {
                        return null;
                    }
                })();

                if (saved && !saved.isOk) {
                    logger.toastWarn(
                        'Failed to add toplevel action:',
                        saved.err,
                    );
                } else {
                    closeModal!();
                }
            }}
            onCancel={closeModal}
            onEscKeypress={closeModal}
        >
            <HandleLoading
                value={toplevel}
                onOk={(toplevel) => {
                    return (
                        <Dropdown
                            selectedOption={state ?? toplevel[0]}
                            rgOptions={toplevel.map((v) => {
                                return {
                                    label: v.name,
                                    data: v,
                                };
                            })}
                            onChange={(v) => {
                                setState(v.data);
                            }}
                        />
                    );
                }}
            />
            <p>{state?.description}</p>
        </ConfirmModal>
    );
}
