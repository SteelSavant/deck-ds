import { Field, Focusable } from "decky-frontend-lib";
import { ReactElement } from "react";
import { Action } from "../../backend";
import { ActionChildProps } from "../../components/ActionChild";
import InternalEditAction from "../../components/EditAction";


interface QAMEditActionProps {
    action: Action,
    onChange: (action: Action) => void,
}

/// TODO::merge QAMEditAction and EditAction by passing in the ActionChild builder and notConfigurable  value;

export default function QAMEditAction({
    action,
    onChange,
}: QAMEditActionProps): ReactElement | null {
    return <InternalEditAction
        action={action}
        onChange={onChange}
        indentLevel={0}
        ActionChild={QAMActionChild}
    />
}

function QAMActionChild({ children, label, }: ActionChildProps): ReactElement {
    return (
        <Field label={label} focusable={false} >
            <div style={{ paddingRight: '10px' }}>
                <Focusable >
                    {children}
                </Focusable>
            </div>
        </Field>
    );
}