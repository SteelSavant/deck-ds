import { ConfirmModal } from 'decky-frontend-lib';
import { Fragment, ReactElement } from 'react';
import { DependencyError } from '../backend';

export default function ConfigErrorModal({
    errors,
    closeModal,
}: {
    errors: DependencyError[];
    closeModal?: () => void;
}): ReactElement {
    return (
        <ConfirmModal
            strTitle="Configuration Error"
            bAlertDialog={true}
            onOK={closeModal}
            onCancel={closeModal}
            onEscKeypress={closeModal}
        >
            <>
                <p>
                    Configuration {errors.length > 1 ? 'errors' : 'error'}{' '}
                    ocurred:
                </p>
                {errors.map(DisplayError)}
            </>
        </ConfirmModal>
    );
}

function DisplayError(error: DependencyError): ReactElement {
    const type = error.type;
    const msg = (() => {
        switch (type) {
            case 'SystemCmdNotFound':
                return `System Command "${error.value}" not found`;
            case 'PathIsNotFile':
                return `Expected path "${error.value}" to be a file, not a directory`;
            case 'PathIsNotDir':
                return `Expected path "${error.value}" to be a directory, not a file`;
            case 'PathNotFound':
                return `Path "${error.value}" not found`;
            case 'KwinScriptNotFound':
                return `Bundled kwinscript "${error.value}" not found`;
            case 'KwinScriptFailedInstall':
                return `Bundled kwinscript "${error.value}" failed to install`;
            case 'FieldNotSet':
                return `Field "${error.value}" must be set`;
            case 'FlatpakNotFound':
                return `Required flatpak  ${error.value} must be installed`;
            case 'SecondaryAppPresetNotFound':
                return `required secondary app preset ${error.value} must exist`;
            default:
                const typecheck: never = type;
                throw `DependencyError failed to typecheck: ${typecheck}`;
        }
    })();

    return <li>{msg}</li>;
}
