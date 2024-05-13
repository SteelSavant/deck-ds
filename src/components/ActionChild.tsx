import { Field } from "decky-frontend-lib";
import { ReactElement } from "react";

export interface ActionChildProps {
    children?: ReactElement,
    label: string,
    description?: string | undefined,
    indentLevel: number,
    childrenLayout?: "below" | "inline" | undefined,
    inlineWrap?: 'keep-inline' | 'shift-children-below' | undefined,
};

export type ActionChildBuilder = (props: ActionChildProps) => ReactElement;

export function ActionChild({ children, label, description, indentLevel, childrenLayout, inlineWrap }: ActionChildProps): ReactElement {
    return (
        <Field label={label} focusable={false} description={description} indentLevel={indentLevel} childrenLayout={childrenLayout} inlineWrap={inlineWrap} >
            <div style={{ paddingRight: '10px' }}>
                {children}
            </div>
        </Field>
    );
}

