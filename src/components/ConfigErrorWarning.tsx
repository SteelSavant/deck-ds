import { ConfirmModal, DialogButton, Focusable, showModal } from "decky-frontend-lib";
import { ReactElement } from "react";
import { FaExclamationTriangle } from "react-icons/fa";
import { DependencyError } from "../types/backend_api";

export default function ConfigErrorWarning({ errors }: { errors: DependencyError[] | undefined }): ReactElement {
    if (errors) {
        const onClick = () => {
            showModal(
                <ConfirmModal strTitle="Configuration Error" bAlertDialog={true}>
                    <div>
                        <p>Configuration {errors.length > 1 ? 'errors' : 'error'} ocurred:</p>
                        {errors.map(DisplayError)}
                    </div>
                </ConfirmModal>
            );
        }

        return (
            <Focusable>
                <DialogButton
                    onClick={onClick} onOKButton={onClick}
                    style={{
                        width: 'fit-content',
                        minWidth: 'fit-content',
                        height: 'fit-content',
                        padding: '10px 12px',
                    }}
                >
                    <FaExclamationTriangle color="yellow" />
                </DialogButton>
            </Focusable>
        )
    }
    else {
        return <div />
    }
}

function DisplayError(error: DependencyError): ReactElement {
    const type = error.type
    const msg = (() => {
        switch (type) {
            case "SystemCmdNotFound":
                return `System Command "${error.value}" not found`
            case "PathIsNotFile":
                return `Expected path "${error.value}" to be a file, not a directory`
            case "PathIsNotDir":
                return `Expected path "${error.value}" to be a directory, not a file`
            case "PathNotFound":
                return `Path "${error.value}" not found`
            case "KwinScriptNotFound":
                return `Bundled kwinscript "${error.value}" not found`
            case "KwinScriptFailedInstall":
                return `Bundled kwinscript "${error.value}" failed to install`
            case "FieldNotSet":
                return `Field "${error.value}" must be set`
            default:
                const typecheck: never = type;
                throw typecheck ?? 'DependencyError failed to typecheck';
        }
    })();

    return <li>{msg}</li>
}