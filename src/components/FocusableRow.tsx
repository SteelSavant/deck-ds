import { Focusable } from '@decky/ui';
import { ReactElement } from 'react';

export default function FocusableRow({
    children,
}: {
    children: (ReactElement | null)[];
}): ReactElement {
    return (
        <Focusable
            style={{ display: 'flex', width: '100%', position: 'relative' }}
        >
            {children}
        </Focusable>
    );
}
