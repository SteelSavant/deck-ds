import { Focusable } from "decky-frontend-lib";
import { ReactElement } from "react";

export default function FocusableRow({ children }: { children: ReactElement[] }): ReactElement {
    return (
        <Focusable style={{ display: 'flex', width: '100%', position: 'relative' }}>
            {children}
        </Focusable>
    )
}