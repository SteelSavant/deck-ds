import { ConfirmModal } from "decky-frontend-lib";
import { ReactElement } from "react";
import { DependencyError } from "../backend";

export default function ConfigErrorModal({ errors, closeModal }: { errors: DependencyError[], closeModal?: () => void }): ReactElement {
    return <ConfirmModal
        strTitle="Configuration Error" bAlertDialog={true} onOK={closeModal}
        onCancel={closeModal}
        onEscKeypress={closeModal}
    >
        <div>
            <p>Configuration {errors.length > 1 ? 'errors' : 'error'} ocurred:</p>
            {errors.map(DisplayError)}
        </div>
    </ConfirmModal >
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
            case "FlatpakNotFound":
                // TODO::this
                throw "not implemented"
            case "SecondaryAppPresetNotFound":
                // TODO::this
                throw "not implemented"
            default:
                const typecheck: never = type;
                throw typecheck ?? 'DependencyError failed to typecheck';
        }
    })();

    return <li>{msg}</li>
}