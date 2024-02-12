import { Field, Focusable } from "decky-frontend-lib";
import { ReactElement } from "react";

export interface ActionChildProps { children: ReactElement, label: string, description?: string | undefined, indentLevel: number };

export type ActionChildBuilder = (props: ActionChildProps) => ReactElement;

export function ActionChild({ children, label, description, indentLevel }: ActionChildProps): ReactElement {
    return (
        <Field label={label} focusable={false} description={description} indentLevel={indentLevel} >
            <div style={{ paddingRight: '10px' }}>
                <Focusable >
                    {children}
                </Focusable>
            </div>
        </Field>
    );
}

