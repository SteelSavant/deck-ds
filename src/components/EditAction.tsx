import { DialogButton, Dropdown, Field, FileSelectionType, Focusable, Toggle } from "decky-frontend-lib";
import _ from "lodash";
import { ReactElement } from "react";
import { FaFile } from "react-icons/fa";
import { Action, citraLayoutOptions, melonDSLayoutOptions, melonDSSizingOptions } from "../backend";
import { useServerApi } from "../context/serverApiContext";


interface EditActionProps {
    action: Action,
    indentLevel: number,
    onChange: (action: Action) => void,
}

export default function EditAction({
    action,
    indentLevel,
    onChange,
}: EditActionProps): ReactElement {
    const cloned = _.cloneDeep(action);
    const type = cloned.type;

    const serverApi = useServerApi();

    const notConfigurable = (<div />);

    switch (type) {
        case 'CemuLayout':
            return (
                <div>
                    <ActionChild indentLevel={indentLevel} label="Separate Gamepad View">
                        <Toggle value={cloned.value.separate_gamepad_view} onChange={(isEnabled) => {
                            console.log("toggling separate gamepad view:", isEnabled);
                            cloned.value.separate_gamepad_view = isEnabled;
                            onChange(cloned);
                        }} />
                    </ActionChild>
                </div>
            );
        case 'CitraLayout':
            return (
                <div>
                    <ActionChild indentLevel={indentLevel} label="Layout Option">

                        <Dropdown selectedOption={cloned.value.layout_option.type} rgOptions={citraLayoutOptions.map((a) => {
                            return {
                                label: a.type,
                                data: a.type
                            }
                        })} onChange={(option) => {
                            cloned.value.layout_option = { type: option.data };
                            onChange(cloned);
                        }} />
                    </ActionChild>
                    <ActionChild indentLevel={indentLevel} label="Swap Screens">
                        <Toggle value={cloned.value.swap_screens} onChange={(isEnabled) => {
                            cloned.value.swap_screens = isEnabled;
                            onChange(cloned);
                        }} />
                    </ActionChild>
                </div>
            );
        case 'MelonDSLayout':
            return (
                <div>
                    <ActionChild indentLevel={indentLevel} label="Layout Option">
                        <Dropdown selectedOption={cloned.value.layout_option} rgOptions={melonDSLayoutOptions.map((a) => {
                            return {
                                label: a,
                                data: a
                            }
                        })} onChange={(option) => {
                            cloned.value.layout_option = option.data;
                            onChange(cloned);
                        }} />
                    </ActionChild>
                    <ActionChild indentLevel={indentLevel} label="Sizing Option">

                        <Dropdown selectedOption={cloned.value.sizing_option} rgOptions={melonDSSizingOptions.map((a) => {
                            return {
                                label: a,
                                data: a
                            }
                        })} onChange={(option) => {
                            cloned.value.sizing_option = option.data;
                            onChange(cloned);
                        }} />
                    </ActionChild>
                    <ActionChild indentLevel={indentLevel} label="Swap Screens">

                        <Toggle value={cloned.value.swap_screens} onChange={(isEnabled) => {
                            cloned.value.swap_screens = isEnabled;
                            onChange(cloned);
                        }} />
                    </ActionChild>
                    <ActionChild indentLevel={indentLevel} label="Book Mode (Rotate Screens)">
                        <Toggle value={cloned.value.book_mode} onChange={(isEnabled) => {
                            cloned.value.book_mode = isEnabled;
                            onChange(cloned);
                        }} />

                    </ActionChild>
                </div >
            );
        case 'SourceFile':
            const sourceValue = cloned.value;
            const sourceType = sourceValue.type;

            switch (sourceType) {
                case 'Custom':
                    const file = sourceValue.value.path;
                    const extensions = sourceValue.value.valid_ext;
                    async function onSelectFile() {
                        const pickedFile = await serverApi.openFilePickerV2(FileSelectionType.FILE, file ?? '/home/deck', true, true, undefined, extensions, false);
                        cloned.value = {
                            type: 'Custom',
                            value: {
                                path: pickedFile.realpath, // TODO::consider path instead of realpath
                                valid_ext: extensions
                            }
                        }
                        onChange(cloned)
                    }
                    return <ActionChild indentLevel={indentLevel} label="File Path" description={file ?? 'Not set'}>
                        <DialogButton style={{ display: 'flex', width: '100%', position: 'relative' }} onClick={onSelectFile} onOKButton={onSelectFile}>
                            <div style={{ display: 'flex', minWidth: '100px', justifyContent: 'space-between', alignItems: 'center' }}>
                                <FaFile style={{ paddingRight: '1rem' }} />
                                Select File
                            </div>
                        </DialogButton>
                    </ActionChild>
                default:
                    return notConfigurable;
            }
        case 'DisplayConfig': // fallthrough
        case 'MultiWindow': // fallthrough
        case 'VirtualScreen':
            return notConfigurable;
        default:
            const typecheck: never = type;
            throw typecheck ?? 'action for edit failed to typecheck'
    }
}

function ActionChild({ children, label, description, indentLevel }: { children: ReactElement, label: string, description?: string | undefined, indentLevel: number }): ReactElement {
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