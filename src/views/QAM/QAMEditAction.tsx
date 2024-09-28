import {
    Field,
    Focusable,
    PanelSectionRow,
    SliderField,
    SliderFieldProps,
} from '@decky/ui';
import { ReactElement } from 'react';
import { Action } from '../../backend';
import { ActionChildProps } from '../../components/ActionChild';
import { InternalEditAction } from '../../components/EditAction';

interface QAMEditActionProps {
    action: Action;
    onChange: (action: Action) => void;
}

/// TODO::merge QAMEditAction and EditAction by passing in the ActionChild builder and notConfigurable  value;

export default function QAMEditAction({
    action,
    onChange,
}: QAMEditActionProps): ReactElement | null {
    const internalProps = {
        action,
        onChange,
        indentLevel: 0,
        actionChildBuilder: QAMActionChild,
        actionChildSliderBuilder: QAMActionChildSliderBuilder,
    };
    return InternalEditAction(internalProps);
}

function QAMActionChild({ children, label }: ActionChildProps): ReactElement {
    return (
        <Field
            label={label}
            focusable={false}
            inlineWrap="keep-inline"
            childrenContainerWidth="min"
        >
            <Focusable style={{ paddingRight: '10px' }}>{children}</Focusable>
        </Field>
    );
}

function QAMActionChildSliderBuilder(props: SliderFieldProps): ReactElement {
    return (
        <PanelSectionRow>
            <SliderField {...props} />
        </PanelSectionRow>
    );
}
