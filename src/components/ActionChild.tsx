import { Field } from "decky-frontend-lib";
import { ReactElement } from "react";

export interface ActionChildProps { children?: ReactElement, label: string, description?: string | undefined, indentLevel: number, childrenLayout?: "below" | "inline" | undefined };

export type ActionChildBuilder = (props: ActionChildProps) => ReactElement;

export function ActionChild({ children, label, description, indentLevel, childrenLayout }: ActionChildProps): ReactElement {
    return (
        <Field label={label} focusable={false} description={description} indentLevel={indentLevel} childrenLayout={childrenLayout}>
            <div style={{ paddingRight: '10px' }}>
                {children}
            </div>
        </Field>
    );
}

