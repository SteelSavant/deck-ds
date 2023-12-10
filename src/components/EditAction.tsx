import { Dropdown, Field, Focusable, Toggle } from "decky-frontend-lib";
import _ from "lodash";
import { ReactElement } from "react";
import { Action, citraLayoutOptions, melonDSLayoutOptions, melonDSSizingOptions } from "../backend";


interface EditActionProps {
    action: Action,
    onChange: (action: Action) => void,
}

export default function EditAction({
    action,
    onChange,
}: EditActionProps): ReactElement {
    const cloned = _.cloneDeep(action);
    const type = cloned.type;
    switch (type) {
        case 'CemuLayout':
            return (
                <div>
                    <Field focusable={false} label="Separate Gamepad View">
                        <Toggle value={cloned.value.separate_gamepad_view} onChange={(isEnabled) => {
                            cloned.value.separate_gamepad_view = isEnabled;
                            onChange(cloned);
                        }} />
                    </Field>
                </div>
            );
        case 'CitraLayout':
            return (
                <div>

                    <Focusable >
                        <Dropdown selectedOption={cloned.value.layout_option.type} rgOptions={citraLayoutOptions.map((a) => {
                            return {
                                label: a.type,
                                data: a
                            }
                        })} onChange={(option) => {
                            cloned.value.layout_option = option.data;
                            onChange(cloned);
                        }} />
                    </Focusable>
                    <Focusable>
                        <Field focusable={false} label="Swap Screens">
                            <Focusable>
                                <Toggle value={cloned.value.swap_screens} onChange={(isEnabled) => {
                                    cloned.value.swap_screens = isEnabled;
                                    onChange(cloned);
                                }} />
                            </Focusable>
                        </Field>
                    </Focusable>
                </div>
            );
        case 'MelonDSLayout':
            return (
                <div>
                    <Focusable >
                        <Dropdown selectedOption={cloned.value.layout_option} rgOptions={melonDSLayoutOptions.map((a) => {
                            return {
                                label: a,
                                data: a
                            }
                        })} onChange={(option) => {
                            cloned.value.layout_option = option.data;
                            onChange(cloned);
                        }} />
                    </Focusable>
                    <Field focusable={false} label="Swap Screens">
                        <Focusable >
                            <Dropdown selectedOption={cloned.value.sizing_option} rgOptions={melonDSSizingOptions.map((a) => {
                                return {
                                    label: a,
                                    data: a
                                }
                            })} onChange={(option) => {
                                cloned.value.sizing_option = option.data;
                                onChange(cloned);
                            }} />
                        </Focusable>
                    </Field>
                    <Field focusable={false} label="Swap Screens">
                        <Focusable>
                            <Toggle value={cloned.value.swap_screens} onChange={(isEnabled) => {
                                cloned.value.swap_screens = isEnabled;
                                onChange(cloned);
                            }} />
                        </Focusable>
                    </Field>
                    <Field focusable={false} label="Book Mode (Rotate Screens)">
                        <Focusable>
                            <Toggle value={cloned.value.book_mode} onChange={(isEnabled) => {
                                cloned.value.book_mode = isEnabled;
                                onChange(cloned);
                            }} />
                        </Focusable>
                    </Field>
                </div >
            );
        case 'SourceFile':
            const sourceValue = cloned.value;
            const sourceType = sourceValue.type;

            switch (sourceType) {
                case 'Custom':
                    return <Field focusable={false} label='Custom Path'>
                        <p> TODO </p>
                    </Field>
                default:
                    return (
                        <p> Not Configurable</p>
                    );
            }
        case 'DisplayConfig': // fallthrough
        case 'MultiWindow': // fallthrough
        case 'VirtualScreen':
            return (
                <p> Not Configurable</p>
            );
        default:
            const typecheck: never = type;
            throw typecheck ?? 'action for edit failed to typecheck'
    }
}