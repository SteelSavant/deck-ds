import { Field, SliderFieldProps } from 'decky-frontend-lib';
import { ReactElement } from 'react';

export interface ActionChildProps {
    children?: ReactElement;
    label: string;
    description?: string;
    indentLevel: number;
    childrenLayout?: 'below' | 'inline';
    inlineWrap?: 'keep-inline' | 'shift-children-below';
}

export type ActionChildBuilder = (props: ActionChildProps) => ReactElement;
export type ActionChildSliderBuilder = (
    props: SliderFieldProps,
) => ReactElement | null;

export function ActionChild({
    children,
    label,
    description,
    indentLevel,
    childrenLayout,
    inlineWrap,
}: ActionChildProps): ReactElement {
    return (
        <Field
            label={label}
            focusable={false}
            description={description}
            indentLevel={indentLevel}
            childrenLayout={childrenLayout}
            inlineWrap={inlineWrap}
        >
            <div style={{ paddingRight: '10px' }}>{children}</div>
        </Field>
    );
}
