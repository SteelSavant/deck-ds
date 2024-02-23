import { DialogButton, Dropdown, FileSelectionType, Toggle } from "decky-frontend-lib";
import _ from "lodash";
import { Fragment, ReactElement } from "react";
import { FaFile } from "react-icons/fa";
import { Action, CemuOptions, CitraOptions, DolphinOptions, ExternalDisplaySettings, LimitedMultiWindowLayout, MultiWindowLayout, RelativeLocation, citraLayoutOptions, melonDSLayoutOptions, melonDSSizingOptions } from "../backend";
import { useServerApi } from "../context/serverApiContext";
import { ActionChild, ActionChildBuilder } from "./ActionChild";


interface EditActionProps {
    action: Action,
    indentLevel: number,
    onChange: (action: Action) => void,
}

export function EditAction(props: EditActionProps): ReactElement | null {
    const internalProps = {
        ...props,
        actionChildBuilder: ActionChild
    };
    return InternalEditAction(internalProps)
}

type InternalEditActionProps = { actionChildBuilder: ActionChildBuilder } & EditActionProps;

export function InternalEditAction({
    action,
    indentLevel,
    onChange,
    actionChildBuilder,
}: InternalEditActionProps): ReactElement | null {

    const Builder = actionChildBuilder;
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
                    <Builder indentLevel={indentLevel} label="External Display Settings" description="External display settings (resolution, refresh rate, etc.).">
                        <Dropdown selectedOption={display.teardown_external_settings.type} rgOptions={externalSettings.map((setting) => {
                            return {
                                label: labelForCamelCase(setting.type),
                                data: setting.type
                            };
                        })}
                            onChange={(settings) => {
                                cloned.value.teardown_external_settings.type = settings.data;
                                onChange(cloned)
                            }}
                        />
                    </Builder>
                    <Builder indentLevel={indentLevel} label="Deck Screen Location" description="Location of the Deck screen on the desktop relative to the external screen.">
                        <Dropdown selectedOption={display.teardown_deck_location} rgOptions={[
                            {
                                label: 'Disabled',
                                data: null,
                            },
                            ...locations.map((location) => {
                                return {
                                    label: labelForCamelCase(location, '-'),
                                    data: location,
                                }
                            })
                        ]}
                            onChange={(settings) => {
                                cloned.value.teardown_deck_location = settings.data;
                                onChange(cloned)
                            }}
                        />
                    </Builder>
                    {
                        cloned.value.teardown_deck_location
                            ? <Builder indentLevel={indentLevel} label="Deck is Primary Display" description="If enabled, the Deck's embedded display will be the primary desktop in KDE (the one with the taskbar).">
                                <Toggle value={cloned.value.deck_is_primary_display} onChange={(isEnabled) => {
                                    cloned.value.deck_is_primary_display = isEnabled;
                                    onChange(cloned);
                                }} />
                            </Builder>
                            : <div />
                    }
                </div>
            );
        case 'DisplayConfig': {
            // TODO::This is largely a duplicate of the above; refactor when Preference gets configured in UI.
            const display = cloned.value;
            const locations: RelativeLocation[] = ['Above', 'Below', 'LeftOf', 'RightOf']; // SameAs excluded because it doesn't really make sense
            const externalSettings: ExternalDisplaySettings[] = [{ type: 'Previous' }, { type: 'Native' }] // Preference excluded because its a pain to configure, and I'm pretty sure doesn't work
            return (
                <div>
                    <Builder indentLevel={indentLevel} label="External Display Settings" description="External display settings.">
                        <Dropdown selectedOption={display.external_display_settings.type} rgOptions={externalSettings.map((setting) => {
                            return {
                                label: labelForCamelCase(setting.type),
                                data: setting.type
                            };
                        })}
                            onChange={(settings) => {
                                cloned.value.external_display_settings.type = settings.data;
                                onChange(cloned)
                            }}
                        />
                    </Builder>
                    <Builder indentLevel={indentLevel} label="Deck Screen Location" description="Location of the Deck screen on the desktop relative to the external screen.">
                        <Dropdown selectedOption={display.deck_location} rgOptions={[
                            {
                                label: 'Disabled',
                                data: null,
                            },
                            ...locations.map((location) => {
                                return {
                                    label: labelForCamelCase(location, '-'),
                                    data: location,
                                }
                            })
                        ]}
                            onChange={(settings) => {
                                cloned.value.deck_location = settings.data;
                                onChange(cloned)
                            }}
                        />
                    </Builder>
                    {
                        cloned.value.deck_location
                            ? <Builder indentLevel={indentLevel} label="Deck is Primary Display" description="If enabled, the Deck's embedded display will be the primary desktop in KDE (the one with the taskbar).">
                                <Toggle value={cloned.value.deck_is_primary_display} onChange={(isEnabled) => {
                                    cloned.value.deck_is_primary_display = isEnabled;
                                    onChange(cloned);
                                }} />
                            </Builder>
                            : <div />
                    }
                </div>
            );
        }
        case 'CemuLayout':
            return (
                <div>
                    <Builder indentLevel={indentLevel} label="Separate Gamepad View">
                        <Toggle value={cloned.value.layout.separate_gamepad_view} onChange={(isEnabled) => {
                            cloned.value.layout.separate_gamepad_view = isEnabled;
                            onChange(cloned);
                        }} />
                    </Builder>
                </div>
            );
        case 'CitraLayout':
            return (
                <div>
                    <Builder indentLevel={indentLevel} label="Layout Option">
                        <Dropdown selectedOption={cloned.value.layout.layout_option.type} rgOptions={citraLayoutOptions.map((a) => {
                            return {
                                label: labelForCamelCase(a.type),
                                data: a.type
                            }
                        })} onChange={(option) => {
                            cloned.value.layout.layout_option = { type: option.data };
                            onChange(cloned);
                        }} />
                    </Builder>
                    <Builder indentLevel={indentLevel} label="Swap Screens">
                        <Toggle value={cloned.value.layout.swap_screens} onChange={(isEnabled) => {
                            cloned.value.layout.swap_screens = isEnabled;
                            onChange(cloned);
                        }} />
                    </Builder>
                </div>
            );
        case 'MelonDSLayout':
            return (
                <div>
                    <Builder indentLevel={indentLevel} label="Layout Option">
                        <Dropdown selectedOption={cloned.value.layout_option} rgOptions={melonDSLayoutOptions.map((a) => {
                            return {
                                label: labelForCamelCase(a),
                                data: a
                            }
                        })} onChange={(option) => {
                            cloned.value.layout_option = option.data;
                            onChange(cloned);
                        }} />
                    </Builder>
                    <Builder indentLevel={indentLevel} label="Sizing Option">
                        <Dropdown selectedOption={cloned.value.sizing_option} rgOptions={melonDSSizingOptions.map((a) => {
                            return {
                                label: labelForCamelCase(a),
                                data: a
                            }
                        })} onChange={(option) => {
                            cloned.value.sizing_option = option.data;
                            onChange(cloned);
                        }} />
                    </Builder>
                    <Builder indentLevel={indentLevel} label="Swap Screens">
                        <Toggle value={cloned.value.swap_screens} onChange={(isEnabled) => {
                            cloned.value.swap_screens = isEnabled;
                            onChange(cloned);
                        }} />
                    </Builder>
                    <Builder indentLevel={indentLevel} label="Book Mode (Rotate Screens)">
                        <Toggle value={cloned.value.book_mode} onChange={(isEnabled) => {
                            cloned.value.book_mode = isEnabled;
                            onChange(cloned);
                        }} />
                    </Builder>
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
                    return <Builder indentLevel={indentLevel} label="File Path" description={file ?? 'Not set'}>
                        <DialogButton style={{ display: 'flex', width: '100%', position: 'relative' }} onClick={onSelectFile} onOKButton={onSelectFile}>
                            <div style={{ display: 'flex', minWidth: '100px', justifyContent: 'space-between', alignItems: 'center' }}>
                                <FaFile style={{ paddingRight: '1rem' }} />
                                Select File
                            </div>
                        </DialogButton>
                    </Builder>
                default:
                    return notConfigurable;
            }
        case 'MultiWindow':
            const options = [cloned.value.cemu, cloned.value.citra, cloned.value.dolphin]
                .filter((v) => v)
                .map((v) => v!);

            if (options.length !== 1) {
                // TODO::properly handle multi-emu config if required
                return <p> invalid multi-window configuration; must have exactly one option</p>
            }

            function isDolphin(o: DolphinOptions | CemuOptions | CitraOptions): o is DolphinOptions {
                return !!(o as DolphinOptions).gba_blacklist;
            }

            const option = options[0];
            const layoutOptions: MultiWindowLayout[] = ['column-right', 'column-left', 'square-right', 'square-left', 'separate'];
            const dolphinLimitedLayoutOptions: LimitedMultiWindowLayout[] = ['column-right', 'column-left', 'square-right', 'square-left']
            const dsLimitedLayoutOptions: MultiWindowLayout[] = ['column-right', 'column-left'];
            const limitedLayoutOptions = isDolphin(option)
                ? dolphinLimitedLayoutOptions
                : dsLimitedLayoutOptions;

            function DolphinAction(option: DolphinOptions): ReactElement {
                return (
                    <Builder indentLevel={indentLevel} label="Multi-Screen Layout" description="Layout when the Deck's embedded display is enabled and an external display is connected." >
                        <Fragment>
                            <Builder indentLevel={indentLevel + 1} label="Single-GBA Layout" description="Layout when a single GBA window is visible.">
                                <Dropdown selectedOption={option.multi_screen_single_secondary_layout} rgOptions={layoutOptions.map((a) => {
                                    return {
                                        label: labelForKebabCase(a),
                                        data: a
                                    }
                                })} onChange={(value) => {
                                    option.multi_screen_single_secondary_layout = value.data;
                                    onChange(cloned);
                                }} />
                            </Builder>
                            <Builder indentLevel={indentLevel + 1} label="Multi-GBA Layout" description="Layout when multiple GBA windows are visible.">
                                <Dropdown selectedOption={option.multi_screen_multi_secondary_layout} rgOptions={layoutOptions.map((a) => {
                                    return {
                                        label: labelForKebabCase(a),
                                        data: a
                                    }
                                })} onChange={(value) => {
                                    option.multi_screen_multi_secondary_layout = value.data;
                                    onChange(cloned);
                                }} />
                            </Builder>
                        </Fragment>
                    </Builder>
                );
            }

            function DsAction(option: CemuOptions | CitraOptions): ReactElement {
                return (
                    <Builder indentLevel={indentLevel} label="Multi-Screen Layout" description="Layout when the Deck's embedded display is enabled and an external display is connected.">
                        <Dropdown selectedOption={option.multi_screen_layout} rgOptions={layoutOptions.map((a) => {
                            return {
                                label: labelForKebabCase(a),
                                data: a
                            }
                        })} onChange={(value) => {
                            option.multi_screen_layout = value.data;
                            onChange(cloned);
                        }} />
                    </Builder>
                );
            }

            return <Fragment>
                <Builder indentLevel={indentLevel} label="Keep Above" description="Keep emulator windows above others.">
                    <Toggle value={cloned.value.general.keep_above} onChange={(isEnabled) => {
                        cloned.value.general.keep_above = isEnabled;
                        onChange(cloned);
                    }} />
                </Builder>
                <Builder indentLevel={indentLevel} label="Swap Screens" description="Use the Deck's embedded display as the main display, instead of as the secondary display.">
                    <Toggle value={cloned.value.general.swap_screens} onChange={(isEnabled) => {
                        cloned.value.general.swap_screens = isEnabled;
                        onChange(cloned);
                    }} />
                </Builder>
                <Builder indentLevel={indentLevel} label="Single Screen Layout" description="Layout when only the Deck's embedded display is available, or when an external display is connected while the Deck's embedded display is disabled.">
                    <Dropdown selectedOption={option.single_screen_layout} rgOptions={limitedLayoutOptions.map((a) => {
                        return {
                            label: labelForKebabCase(a),
                            data: a
                        }
                    })} onChange={(value) => {
                        option.single_screen_layout = value.data;
                        onChange(cloned);
                    }} />
                </Builder>
                {
                    isDolphin(option)
                        ? DolphinAction(option)
                        : DsAction(option)
                }


            </Fragment>
        case 'VirtualScreen':
            return notConfigurable;
        default:
            const typecheck: never = type;
            throw typecheck ?? 'action for edit failed to typecheck'
    }
}


function labelForCamelCase(s: string, separator = ' '): string {
    const splitIndexes: number[] = [];
    s = s[0].toUpperCase() + s.slice(1);

    [...s].forEach((c, i) => {
        if (c === c.toUpperCase()) {
            splitIndexes.push(i)
        }
    });

    splitIndexes.push(s.length);
    let start = splitIndexes.shift();

    const words = [];
    for (const next of splitIndexes) {
        words.push(s.slice(start, next))
        start = next;
    }

    return words.join(separator);
}

function labelForKebabCase(s: string): string {
    return s.split('-').map((v) => v[0].toUpperCase() + v.slice(1).toLowerCase()).join('-');
}
