import { DialogButton, Dropdown, DropdownOption, FileSelectionType, Focusable, TextField, Toggle } from "decky-frontend-lib";
import _ from "lodash";
import React, { Fragment, ReactElement, useState } from "react";
import { FaFile } from "react-icons/fa";
import { FaPlus, FaTrash } from "react-icons/fa6";
import { Action, CemuWindowOptions, CitraWindowOptions, DolphinWindowOptions, ExternalDisplaySettings, LimitedMultiWindowLayout, MultiWindowLayout, RelativeLocation, citraLayoutOptions, melonDSLayoutOptions, melonDSSizingOptions, secondaryAppScreenPreferences, secondaryAppWindowingOptions } from "../backend";
import { useServerApi } from "../context/serverApiContext";
import useDisplayInfo from "../hooks/useDisplayInfo";
import useSecondaryAppInfo from "../hooks/useSecondaryAppPresetInfo";
import { CustomWindowOptions, LaunchSecondaryAppPreset, LaunchSecondaryFlatpakApp, ModePreference } from "../types/backend_api";
import { ActionChild, ActionChildBuilder } from "./ActionChild";
import HandleLoading from "./HandleLoading";


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
            return (
                <Fragment>
                    <ExternalDisplaySettingsSelector
                        indentLevel={indentLevel}
                        settings={cloned.value.teardown_external_settings}
                        Builder={Builder}
                        onChange={(settings) => {
                            cloned.value.teardown_external_settings = settings;
                            onChange(cloned);
                        }}
                    />
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
                </Fragment>
            );
        case 'DisplayConfig': {
            // TODO::This is largely a duplicate of the above DesktopSessionHandler; refactor when Preference gets configured in UI.
            const display = cloned.value;
            const locations: RelativeLocation[] = ['Above', 'Below', 'LeftOf', 'RightOf']; // SameAs excluded because it doesn't really make sense
            return (
                <Fragment>
                    <ExternalDisplaySettingsSelector
                        indentLevel={indentLevel}
                        settings={cloned.value.external_display_settings}
                        Builder={Builder}
                        onChange={(settings) => {
                            cloned.value.external_display_settings = settings;
                            onChange(cloned);
                        }}
                    />
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
                </Fragment>
            );
        }
        case 'CemuLayout':
            return (
                <Fragment>
                    <Builder indentLevel={indentLevel} label="Separate Gamepad View">
                        <Toggle value={cloned.value.layout.separate_gamepad_view} onChange={(isEnabled) => {
                            cloned.value.layout.separate_gamepad_view = isEnabled;
                            onChange(cloned);
                        }} />
                    </Builder>
                </Fragment>
            );
        case 'CitraLayout':
            return (
                <Fragment>
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
                    <Builder indentLevel={indentLevel} label="Rotate Upright">
                        <Toggle value={cloned.value.layout.swap_screens} onChange={(isEnabled) => {
                            cloned.value.layout.swap_screens = isEnabled;
                            onChange(cloned);
                        }} />
                    </Builder>
                </Fragment>
            );
        case 'Lime3dsLayout':
            return InternalEditAction({
                action: {
                    type: 'CitraLayout',
                    value: cloned.value,
                },
                indentLevel,
                actionChildBuilder,
                onChange,
            });
        case 'MelonDSLayout':
            return (
                <Fragment>
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
                </Fragment >
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
            const options = [cloned.value.cemu, cloned.value.citra, cloned.value.dolphin, cloned.value.custom]
                .filter((v) => v)
                .map((v) => v!);

            if (options.length !== 1) {
                // TODO::properly handle multi-emu config if required
                return <p> invalid multi-window configuration; must have exactly one option</p>
            }

            function isDolphin(o: DolphinWindowOptions | CemuWindowOptions | CitraWindowOptions | CustomWindowOptions): o is DolphinWindowOptions {
                return !!(o as DolphinWindowOptions).gba_blacklist;
            }

            function isCustom(o: DolphinWindowOptions | CemuWindowOptions | CitraWindowOptions | CustomWindowOptions): o is CustomWindowOptions {
                return !!(o as CustomWindowOptions).classes;
            }


            const option = options[0];
            const layoutOptions: MultiWindowLayout[] = ['column-right', 'column-left', 'square-right', 'square-left', 'separate'];
            const dolphinLimitedLayoutOptions: LimitedMultiWindowLayout[] = ['column-right', 'column-left', 'square-right', 'square-left']
            const dsLimitedLayoutOptions: MultiWindowLayout[] = ['column-right', 'column-left'];
            const limitedLayoutOptions = isDolphin(option)
                ? dolphinLimitedLayoutOptions
                : dsLimitedLayoutOptions;

            function DolphinAction(option: DolphinWindowOptions): ReactElement {
                return (
                    <Fragment>
                        <Builder indentLevel={indentLevel} label="Multi-Screen Layout" description="Layout when the Deck's embedded display is enabled and an external display is connected." >
                            <Fragment />
                        </Builder>
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

                );
            }

            function CustomAction(option: CustomWindowOptions): ReactElement {
                // Note: classes, etc. aren't set by the user, but instead reconfigured at runtime
                // In the future, we may consider using the fields as overrides to the automatic scraping

                return (
                    <Fragment>
                        <Builder indentLevel={indentLevel} label="Multi-Screen Layout" description="Layout when the Deck's embedded display is enabled and an external display is connected." >
                            <Fragment />
                        </Builder>
                        <Builder indentLevel={indentLevel + 1} label="Single-Window Layout" description="Layout when a single alternate window is visible.">
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
                        <Builder indentLevel={indentLevel + 1} label="Multi-Window Layout" description="Layout when multiple alternate windows are visible.">
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
                );
            }

            function DsAction(option: CemuWindowOptions | CitraWindowOptions): ReactElement {
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
                        : isCustom(option)
                            ? CustomAction(option)
                            : DsAction(option)
                }
            </Fragment>;
        case 'LaunchSecondaryFlatpakApp': {
            return <SecondaryFlatpakApp
                cloned={cloned}
                indentLevel={indentLevel}
                Builder={Builder}
                onChange={onChange}
            />
        }
        case 'LaunchSecondaryAppPreset': {
            return <SecondaryAppPreset
                cloned={cloned}
                indentLevel={indentLevel}
                Builder={Builder}
                onChange={onChange}
            />
        }
        case 'MainAppAutomaticWindowing':
            return (
                <Fragment>
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
                </Fragment>
            );

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

interface LaunchSecondaryFlatpakAppProps {
    cloned: { type: 'LaunchSecondaryFlatpakApp', value: LaunchSecondaryFlatpakApp }, indentLevel: number, onChange: (action: Action) => void, Builder: ActionChildBuilder

}

function SecondaryFlatpakApp({ cloned, indentLevel, onChange, Builder }: LaunchSecondaryFlatpakAppProps): ReactElement {
    const secondaryInfo = useSecondaryAppInfo();
    const [args, setArgs] = useState(cloned.value.app.args);

    return (
        <HandleLoading
            value={secondaryInfo}
            onOk={(secondaryInfo) => {

                // TODO::per-arg reorderable list (likely in a popup-menu), rather than a comma-separated list

                const windowing = cloned.value.windowing_behavior;

                var i = 0;

                const textStyle: React.CSSProperties = {
                    width: '8rem',
                    borderTopRightRadius: 0,
                    borderBottomRightRadius: 0,
                }

                const displayArgs = args.map((arg) => {
                    const index = i++;
                    const deleteArg = () => {
                        args.splice(index, 1)
                        setArgs(args)
                        cloned.value.app.args = args;
                        onChange(cloned);
                    }

                    return (
                        <Focusable style={{
                            display: 'flex',
                            flexDirection: 'row',
                            justifyContent: 'space-between',
                            alignItems: 'center',
                            marginBottom: '5px',
                        }}>
                            <TextField style={textStyle} value={arg} onChange={(v) => {
                                args[index] = v.target.value;
                                setArgs(args)
                                cloned.value.app.args = args;
                                onChange(cloned);
                            }} />
                            <DialogButton style={{
                                backgroundColor: 'red',
                                height: '40px',
                                width: '40px',
                                padding: '10px 12px',
                                minWidth: '40px',
                                display: 'flex',
                                flexDirection: 'column',
                                justifyContent: 'center',
                                borderTopLeftRadius: 0,
                                borderBottomLeftRadius: 0,
                            }}
                                onOKButton={deleteArg}
                                onClick={deleteArg}
                            >
                                <FaTrash />
                            </DialogButton>
                        </Focusable>
                    )
                });

                const addArg = () => {
                    args.push('');
                    setArgs(args)
                    cloned.value.app.args = args;
                    onChange(cloned)
                }

                return (
                    <Fragment>
                        <Builder indentLevel={indentLevel} label="Flatpak App" description="The flatpak to run.">
                            <Dropdown
                                selectedOption={cloned.value.app.app_id}
                                rgOptions={secondaryInfo.installed_flatpaks.map((v) => {
                                    return {
                                        label: `${v.name} (${v.app_id})`,
                                        data: v.app_id
                                    }
                                })}
                                onChange={(value) => {
                                    cloned.value.app.app_id = value.data
                                    onChange(cloned);
                                }}
                            />
                        </Builder >
                        <Builder indentLevel={indentLevel} label="Args" description="Arguments for the flatpak app." >
                            <Fragment>
                                {displayArgs}
                                <DialogButton
                                    onOKButton={addArg}
                                    onClick={addArg}
                                >
                                    <FaPlus />
                                </DialogButton>
                            </Fragment>
                        </Builder>
                        <Builder indentLevel={indentLevel - 1} label="Windowing">
                            <Dropdown
                                selectedOption={windowing} rgOptions={secondaryAppWindowingOptions.map((a) => {
                                    return {
                                        label: labelForCamelCase(a),
                                        data: a
                                    }
                                })} onChange={(value) => {
                                    cloned.value.windowing_behavior = value.data;
                                    onChange(cloned);
                                }}
                            />
                        </Builder>
                    </Fragment >
                );
            }} />
    )
}


interface SecondaryAppPresetProps {
    cloned: { type: 'LaunchSecondaryAppPreset', value: LaunchSecondaryAppPreset }, indentLevel: number, onChange: (action: Action) => void, Builder: ActionChildBuilder
}

function SecondaryAppPreset({ cloned, indentLevel, onChange, Builder }: SecondaryAppPresetProps): ReactElement {
    const secondaryInfo = useSecondaryAppInfo();
    const [filtered, setFiltered] = useState(true);

    // TODO::ability to create new presets (probably as a separate main toplevel tab, and a "convert to preset" option on "custom")

    return (
        <HandleLoading
            value={secondaryInfo}
            onOk={(secondaryInfo) => {
                const options = [];

                for (const k in secondaryInfo.presets) {
                    const preset = secondaryInfo.presets[k];
                    const flatpakInstalled = (preset.app.type === 'Flatpak' && secondaryInfo.installed_flatpaks.find((v) => v.app_id === preset.app.app_id));
                    const shouldPush = !filtered || flatpakInstalled;

                    if (shouldPush) {
                        options.push({
                            label: preset.name,
                            data: k,
                        })
                    }
                }

                options.sort((a, b) => a.label.localeCompare(b.label));

                return (
                    <Fragment>
                        <Builder indentLevel={indentLevel} label="Filter By Installed">
                            <Toggle value={filtered} onChange={(isEnabled) => {
                                setFiltered(isEnabled);
                            }} />
                        </Builder>
                        <Builder indentLevel={indentLevel} label="Selected Preset">
                            <Dropdown
                                selectedOption={cloned.value.preset}
                                rgOptions={options}
                                onChange={(value) => {
                                    cloned.value.preset = value.data;
                                    onChange(cloned);
                                }}
                            />
                        </Builder>
                        <Builder indentLevel={indentLevel - 1} label="Windowing" description="Behavior of secondary app window(s).">
                            <Dropdown
                                selectedOption={cloned.value.windowing_behavior} rgOptions={secondaryAppWindowingOptions.map((a) => {
                                    return {
                                        label: labelForCamelCase(a),
                                        data: a
                                    }
                                })} onChange={(value) => {
                                    cloned.value.windowing_behavior = value.data;
                                    onChange(cloned);
                                }}
                            />
                        </Builder>
                        {
                            cloned.value.windowing_behavior === 'Fullscreen'
                                ? (
                                    <Builder indentLevel={indentLevel - 1} label="Screen Preference" description="Screen to send secondary app window(s) to if windowing is fullscreen.">
                                        <Dropdown
                                            selectedOption={cloned.value.screen_preference} rgOptions={secondaryAppScreenPreferences.map((a) => {
                                                return {
                                                    label: labelForCamelCase(a),
                                                    data: a
                                                }
                                            })} onChange={(value) => {
                                                cloned.value.screen_preference = value.data;
                                                onChange(cloned);
                                            }}
                                        />
                                    </Builder>
                                )
                                : null
                        }
                    </Fragment>
                )
            }}
        />
    )
}

interface ExternalDisplaySettingsSelectorProps {
    indentLevel: number,
    settings: ExternalDisplaySettings
    onChange: (settings: ExternalDisplaySettings) => void,
    Builder: ActionChildBuilder,
}

function ExternalDisplaySettingsSelector({ indentLevel, settings, onChange, Builder }: ExternalDisplaySettingsSelectorProps): ReactElement {
    const displayInfo = useDisplayInfo();

    return (
        <HandleLoading value={displayInfo} onOk={(displayInfo) => {
            const fixed: ExternalDisplaySettings[] = [{ type: 'Previous' }, { type: 'Native' },];

            const options: DropdownOption[] = fixed.map((setting) => {
                return {
                    label: labelForCamelCase(setting.type),
                    data: setting
                };
            });

            // only show display options if we actuall have some 
            if (displayInfo != null && displayInfo.length > 0) {
                options.push({
                    label: 'Exact',
                    options: displayInfo.map((info) => {
                        const value: ModePreference = {
                            aspect_ratio: {
                                type: 'Exact',
                                value: info.width / info.height,
                            },
                            refresh: {
                                type: 'Exact',
                                value: info.refresh
                            },
                            resolution: {
                                type: 'Exact',
                                value: {
                                    h: info.height,
                                    w: info.width,
                                }
                            }
                        };
                        return {
                            label: `${info.width}x${info.height} @ ${info.refresh.toFixed(2)}`,
                            data: {
                                type: 'Preference',
                                value
                            }
                        }
                    })
                });
            }


            var selected = options.find((v) => {
                if (v.data != null) {
                    return _.isEqual(v.data, settings);
                } else {
                    return v.options?.find((v) => _.isEqual(v.data, settings))
                }
            });

            if (selected?.options) {
                selected = selected.options?.find((v) => _.isEqual(v.data, settings))
            }

            return (
                <Builder indentLevel={indentLevel} label="External Display Settings" description="External display settings (resolution, refresh rate, etc.).">
                    <Dropdown selectedOption={selected?.data} rgOptions={options}
                        onChange={(settings) => {
                            onChange(settings.data)
                        }}
                    />
                </Builder>
            )
        }} />
    );
}