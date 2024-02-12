import { DialogButton, Dropdown, FileSelectionType, Toggle } from "decky-frontend-lib";
import _ from "lodash";
import { ReactElement } from "react";
import { FaFile } from "react-icons/fa";
import { Action, ExternalDisplaySettings, RelativeLocation, citraLayoutOptions, melonDSLayoutOptions, melonDSSizingOptions } from "../backend";
import { useServerApi } from "../context/serverApiContext";
import { ActionChildBuilder } from "./ActionChild";


interface EditActionProps {
    action: Action,
    indentLevel: number,
    onChange: (action: Action) => void,
    ActionChild: ActionChildBuilder,
}

export default function EditAction(props: EditActionProps): ReactElement | null {
    return <InternalEditAction {...props} />
}

export function InternalEditAction({
    action,
    indentLevel,
    onChange,
    ActionChild,
}: EditActionProps): ReactElement | null {
    const cloned = _.cloneDeep(action);
    const type = cloned.type;

    const serverApi = useServerApi();
    const notConfigurable = null;

    switch (type) {
        case 'DesktopSessionHandler':
            const display = cloned.value;
            const locations: RelativeLocation[] = ['Above', 'Below', 'LeftOf', 'RightOf']; // SameAs excluded because it doesn't really make sense
            const externalSettings: ExternalDisplaySettings[] = [{ type: 'Previous' }, { type: 'Native' }] // Preference excluded because its a pain to configure, and I'm pretty sure doesn't work
            return (
                <div>
                    <ActionChild indentLevel={indentLevel} label="External Display Settings" description="External display settings (resolution, refresh rate, etc.).">
                        <Dropdown selectedOption={display.teardown_external_settings.type} rgOptions={externalSettings.map((setting) => {
                            return {
                                label: setting.type,
                                data: setting.type
                            };
                        })}
                            onChange={(settings) => {
                                cloned.value.teardown_external_settings.type = settings.data;
                                onChange(cloned)
                            }}
                        />
                    </ActionChild>
                    <ActionChild indentLevel={indentLevel} label="Deck Screen Location" description="Location of the Deck screen on the desktop relative to the external screen.">
                        <Dropdown selectedOption={display.teardown_deck_location} rgOptions={[
                            {
                                label: 'Disabled',
                                data: null,
                            },
                            ...locations.map((location) => {
                                return {
                                    label: location,
                                    data: location,
                                }
                            })
                        ]}
                            onChange={(settings) => {
                                cloned.value.teardown_deck_location = settings.data;
                                onChange(cloned)
                            }}
                        />
                    </ActionChild>
                </div>
            );
        case 'DisplayConfig': {
            // TODO::This is largely a duplicate of the above; refactor when Preference gets configured in UI.
            const display = cloned.value;
            const locations: RelativeLocation[] = ['Above', 'Below', 'LeftOf', 'RightOf']; // SameAs excluded because it doesn't really make sense
            const externalSettings: ExternalDisplaySettings[] = [{ type: 'Previous' }, { type: 'Native' }] // Preference excluded because its a pain to configure, and I'm pretty sure doesn't work
            return (
                <div>
                    <ActionChild indentLevel={indentLevel} label="External Display Settings" description="External display settings.">
                        <Dropdown selectedOption={display.external_display_settings.type} rgOptions={externalSettings.map((setting) => {
                            return {
                                label: setting.type,
                                data: setting.type
                            };
                        })}
                            onChange={(settings) => {
                                cloned.value.external_display_settings.type = settings.data;
                                onChange(cloned)
                            }}
                        />
                    </ActionChild>
                    <ActionChild indentLevel={indentLevel} label="Deck Screen Location" description="Location of the Deck screen on the desktop.">
                        <Dropdown selectedOption={display.deck_location} rgOptions={[
                            {
                                label: 'Disabled',
                                data: null,
                            },
                            ...locations.map((location) => {
                                return {
                                    label: location,
                                    data: location,
                                }
                            })
                        ]}
                            onChange={(settings) => {
                                cloned.value.deck_location = settings.data;
                                onChange(cloned)
                            }}
                        />
                    </ActionChild>
                </div>
            );
        }
        case 'CemuLayout':
            return (
                <div>
                    <ActionChild indentLevel={indentLevel} label="Separate Gamepad View">
                        <Toggle value={cloned.value.layout.separate_gamepad_view} onChange={(isEnabled) => {
                            cloned.value.layout.separate_gamepad_view = isEnabled;
                            onChange(cloned);
                        }} />
                    </ActionChild>
                </div>
            );
        case 'CitraLayout':
            return (
                <div>
                    <ActionChild indentLevel={indentLevel} label="Layout Option">
                        <Dropdown selectedOption={cloned.value.layout.layout_option.type} rgOptions={citraLayoutOptions.map((a) => {
                            return {
                                label: a.type,
                                data: a.type
                            }
                        })} onChange={(option) => {
                            cloned.value.layout.layout_option = { type: option.data };
                            onChange(cloned);
                        }} />
                    </ActionChild>
                    <ActionChild indentLevel={indentLevel} label="Swap Screens">
                        <Toggle value={cloned.value.layout.swap_screens} onChange={(isEnabled) => {
                            cloned.value.layout.swap_screens = isEnabled;
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
            const sourceType = sourceValue.source.type;

            switch (sourceType) {
                case 'Custom':
                    const file = sourceValue.source.value.path;
                    const extensions = sourceValue.source.value.valid_ext;
                    async function onSelectFile() {
                        const pickedFile = await serverApi.openFilePickerV2(FileSelectionType.FILE, file ?? '/home/deck', true, true, undefined, extensions, false);
                        cloned.value = {
                            id: sourceValue.id,
                            source: {
                                type: 'Custom',
                                value: {
                                    path: pickedFile.realpath, // TODO::consider path instead of realpath
                                    valid_ext: extensions
                                }
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
        case 'MultiWindow':
            return (
                <ActionChild indentLevel={indentLevel} label="Disable Embedded Display" description="Disables the embedded display. Useful for emulators that don't have a combined window mode.">
                    <Toggle value={cloned.value.disable_embedded_display} onChange={(isEnabled) => {
                        cloned.value.disable_embedded_display = isEnabled;
                        onChange(cloned);
                    }} />
                </ActionChild>
            );
        case 'VirtualScreen':
            return notConfigurable;
        default:
            const typecheck: never = type;
            throw typecheck ?? 'action for edit failed to typecheck'
    }
}

