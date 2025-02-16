import { FileSelectionType, openFilePicker } from '@decky/api';
import {
    DialogButton,
    Dropdown,
    DropdownOption,
    Focusable,
    SliderField,
    TextField,
    Toggle,
} from '@decky/ui';
import _ from 'lodash';
import React, { Fragment, ReactElement, useState } from 'react';
import { FaFile } from 'react-icons/fa';
import { FaPlus, FaTrash } from 'react-icons/fa6';
import {
    Action,
    AudioDeviceInfo,
    CemuAudio,
    CemuAudioChannels,
    CemuAudioSetting,
    CemuWindowOptions,
    CitraWindowOptions,
    CustomWindowOptions,
    DolphinWindowOptions,
    ExternalDisplaySettings,
    GamescopeFilter,
    GamescopeFullscreenOption,
    GamescopeScaler,
    LaunchSecondaryAppPreset,
    LaunchSecondaryFlatpakApp,
    LimitedMultiWindowLayout,
    ModePreference,
    MultiWindowLayout,
    RelativeLocation,
    citraLayoutOptions,
    melonDSLayoutOptions,
    melonDSSizingOptions,
    secondaryAppScreenPreferences,
    secondaryAppWindowingOptions,
    touchSelectionModeOptions,
} from '../backend';
import useAudioDeviceInfo from '../hooks/useAudioDeviceInfo';
import useDisplayInfo from '../hooks/useDisplayInfo';
import useSecondaryAppInfo from '../hooks/useSecondaryAppPresetInfo';

import { labelForCamelCase, labelForKebabCase } from '../util/display';
import {
    ActionChild,
    ActionChildBuilder,
    ActionChildSliderBuilder,
} from './ActionChild';
import HandleLoading from './HandleLoading';

interface EditActionProps {
    action: Action;
    indentLevel: number;
    onChange: (action: Action) => void;
}

export function EditAction(props: EditActionProps): ReactElement | null {
    const internalProps = {
        ...props,
        actionChildBuilder: ActionChild,
        actionChildSliderBuilder: SliderField,
    };
    return InternalEditAction(internalProps);
}

type InternalEditActionProps = {
    actionChildBuilder: ActionChildBuilder;
    actionChildSliderBuilder: ActionChildSliderBuilder;
} & EditActionProps;

export function InternalEditAction({
    action,
    indentLevel,
    onChange,
    actionChildBuilder,
    actionChildSliderBuilder,
}: InternalEditActionProps): ReactElement | null {
    const Builder = actionChildBuilder;
    const SliderBuilder = actionChildSliderBuilder;
    const cloned = _.cloneDeep(action);
    const type = cloned.type;

    const notConfigurable = null;

    switch (type) {
        case 'DesktopSessionHandler':
            const display = cloned.value;
            const locations: RelativeLocation[] = [
                'Above',
                'Below',
                'LeftOf',
                'RightOf',
            ]; // SameAs excluded because it doesn't really make sense
            return (
                <>
                    <ExternalDisplaySettingsSelector
                        indentLevel={indentLevel}
                        settings={cloned.value.teardown_external_settings}
                        Builder={Builder}
                        onChange={(settings) => {
                            cloned.value.teardown_external_settings = settings;
                            onChange(cloned);
                        }}
                    />
                    <Builder
                        indentLevel={indentLevel}
                        label="Deck Screen Location"
                        description="Location of the Deck screen on the desktop relative to the external screen."
                    >
                        <Dropdown
                            selectedOption={display.teardown_deck_location}
                            rgOptions={[
                                {
                                    label: 'Disabled',
                                    data: null,
                                },
                                ...locations.map((location) => {
                                    return {
                                        label: labelForCamelCase(location, '-'),
                                        data: location,
                                    };
                                }),
                            ]}
                            onChange={(settings) => {
                                cloned.value.teardown_deck_location =
                                    settings.data;
                                onChange(cloned);
                            }}
                        />
                    </Builder>
                    {cloned.value.teardown_deck_location ? (
                        <Builder
                            indentLevel={indentLevel}
                            label="Deck is Primary Display"
                            description="If enabled, the Deck's embedded display will be the primary desktop in KDE (the one with the taskbar)."
                        >
                            <Toggle
                                value={cloned.value.deck_is_primary_display}
                                onChange={(isEnabled) => {
                                    cloned.value.deck_is_primary_display =
                                        isEnabled;
                                    onChange(cloned);
                                }}
                            />
                        </Builder>
                    ) : null}
                </>
            );
        case 'DisplayConfig': {
            // TODO::This is largely a duplicate of the above DesktopSessionHandler; refactor when Preference gets configured in UI.
            const display = cloned.value;
            const locations: RelativeLocation[] = [
                'Above',
                'Below',
                'LeftOf',
                'RightOf',
            ]; // SameAs excluded because it doesn't really make sense
            return (
                <>
                    <ExternalDisplaySettingsSelector
                        indentLevel={indentLevel}
                        settings={cloned.value.external_display_settings}
                        Builder={Builder}
                        onChange={(settings) => {
                            cloned.value.external_display_settings = settings;
                            onChange(cloned);
                        }}
                    />
                    <Builder
                        indentLevel={indentLevel}
                        label="Deck Screen Location"
                        description="Location of the Deck screen on the desktop relative to the external screen."
                    >
                        <Dropdown
                            selectedOption={display.deck_location}
                            rgOptions={[
                                {
                                    label: 'Disabled',
                                    data: null,
                                },
                                ...locations.map((location) => {
                                    return {
                                        label: labelForCamelCase(location, '-'),
                                        data: location,
                                    };
                                }),
                            ]}
                            onChange={(settings) => {
                                cloned.value.deck_location = settings.data;
                                onChange(cloned);
                            }}
                        />
                    </Builder>
                    {cloned.value.deck_location ? (
                        <Builder
                            indentLevel={indentLevel}
                            label="Deck is Primary Display"
                            description="If enabled, the Deck's embedded display will be the primary desktop in KDE (the one with the taskbar)."
                        >
                            <Toggle
                                value={cloned.value.deck_is_primary_display}
                                onChange={(isEnabled) => {
                                    cloned.value.deck_is_primary_display =
                                        isEnabled;
                                    onChange(cloned);
                                }}
                            />
                        </Builder>
                    ) : null}
                </>
            );
        }
        case 'CemuLayout':
            return (
                <>
                    <Builder
                        indentLevel={indentLevel}
                        label="Separate Gamepad View"
                    >
                        <Toggle
                            value={cloned.value.layout.separate_gamepad_view}
                            onChange={(isEnabled) => {
                                cloned.value.layout.separate_gamepad_view =
                                    isEnabled;
                                onChange(cloned);
                            }}
                        />
                    </Builder>
                </>
            );

        case 'CemuAudio':
            return (
                <CemuAudioSelector
                    indentLevel={indentLevel}
                    settings={cloned.value}
                    onChange={(settings) => {
                        cloned.value = settings;
                        onChange(cloned);
                    }}
                    Builder={Builder}
                    SliderBuilder={SliderBuilder}
                />
            );
        case 'CitraLayout':
            return (
                <>
                    <Builder indentLevel={indentLevel} label="Layout Option">
                        <Dropdown
                            selectedOption={
                                cloned.value.layout.layout_option.type
                            }
                            rgOptions={citraLayoutOptions.map((a) => {
                                return {
                                    label: labelForCamelCase(a.type),
                                    data: a.type,
                                };
                            })}
                            onChange={(option) => {
                                cloned.value.layout.layout_option = {
                                    type: option.data,
                                };
                                onChange(cloned);
                            }}
                        />
                    </Builder>
                    <Builder indentLevel={indentLevel} label="Swap Screens">
                        <Toggle
                            value={cloned.value.layout.swap_screens}
                            onChange={(isEnabled) => {
                                cloned.value.layout.swap_screens = isEnabled;
                                onChange(cloned);
                            }}
                        />
                    </Builder>
                    <Builder indentLevel={indentLevel} label="Rotate Upright">
                        <Toggle
                            value={cloned.value.layout.swap_screens}
                            onChange={(isEnabled) => {
                                cloned.value.layout.swap_screens = isEnabled;
                                onChange(cloned);
                            }}
                        />
                    </Builder>
                </>
            );
        case 'Lime3dsLayout':
            return InternalEditAction({
                action: {
                    type: 'CitraLayout',
                    value: cloned.value,
                },
                indentLevel,
                actionChildBuilder,
                actionChildSliderBuilder,
                onChange,
            });
        case 'MelonDSLayout':
            return (
                <>
                    <Builder indentLevel={indentLevel} label="Layout Option">
                        <Dropdown
                            selectedOption={cloned.value.layout_option}
                            rgOptions={melonDSLayoutOptions.map((a) => {
                                return {
                                    label: labelForCamelCase(a),
                                    data: a,
                                };
                            })}
                            onChange={(option) => {
                                cloned.value.layout_option = option.data;
                                onChange(cloned);
                            }}
                        />
                    </Builder>
                    <Builder indentLevel={indentLevel} label="Sizing Option">
                        <Dropdown
                            selectedOption={cloned.value.sizing_option}
                            rgOptions={melonDSSizingOptions.map((a) => {
                                return {
                                    label: labelForCamelCase(a),
                                    data: a,
                                };
                            })}
                            onChange={(option) => {
                                cloned.value.sizing_option = option.data;
                                onChange(cloned);
                            }}
                        />
                    </Builder>
                    <Builder indentLevel={indentLevel} label="Swap Screens">
                        <Toggle
                            value={cloned.value.swap_screens}
                            onChange={(isEnabled) => {
                                cloned.value.swap_screens = isEnabled;
                                onChange(cloned);
                            }}
                        />
                    </Builder>
                    <Builder
                        indentLevel={indentLevel}
                        label="Book Mode (Rotate Screens)"
                    >
                        <Toggle
                            value={cloned.value.book_mode}
                            onChange={(isEnabled) => {
                                cloned.value.book_mode = isEnabled;
                                onChange(cloned);
                            }}
                        />
                    </Builder>
                </>
            );
        case 'SourceFile':
            const sourceValue = cloned.value;
            const sourceType = sourceValue.source.type;

            switch (sourceType) {
                case 'Custom':
                    const file = sourceValue.source.value.settings_path;
                    const extensions = sourceValue.source.value.valid_ext;
                    async function onSelectFile() {
                        const pickedFile = await openFilePicker(
                            FileSelectionType.FILE,
                            file ?? '/home/deck',
                            true,
                            true,
                            undefined,
                            extensions,
                            false,
                        );
                        cloned.value = {
                            id: sourceValue.id,
                            source: {
                                type: 'Custom',
                                value: {
                                    settings_path: pickedFile.realpath, // TODO::consider path instead of realpath
                                    valid_ext: extensions,
                                },
                            },
                        };
                        onChange(cloned);
                    }
                    return (
                        <Builder
                            indentLevel={indentLevel}
                            label="File Path"
                            description={file ?? 'Not set'}
                        >
                            <DialogButton
                                style={{
                                    display: 'flex',
                                    width: '100%',
                                    position: 'relative',
                                }}
                                onClick={onSelectFile}
                                onOKButton={onSelectFile}
                            >
                                <div
                                    style={{
                                        display: 'flex',
                                        minWidth: '100px',
                                        justifyContent: 'space-between',
                                        alignItems: 'center',
                                    }}
                                >
                                    <FaFile style={{ paddingRight: '1rem' }} />
                                    Select File
                                </div>
                            </DialogButton>
                        </Builder>
                    );
                default:
                    return notConfigurable;
            }
        case 'MultiWindow':
            const options = [
                cloned.value.cemu,
                cloned.value.citra,
                cloned.value.dolphin,
                cloned.value.custom,
            ]
                .filter((v) => v)
                .map((v) => v!);

            if (options.length !== 1) {
                // TODO::properly handle multi-emu config if required
                return (
                    <p>
                        {' '}
                        invalid multi-window configuration; must have exactly
                        one option
                    </p>
                );
            }

            function isDolphin(
                o:
                    | DolphinWindowOptions
                    | CemuWindowOptions
                    | CitraWindowOptions
                    | CustomWindowOptions,
            ): o is DolphinWindowOptions {
                return !!(o as DolphinWindowOptions).gba_blacklist;
            }

            function isCustom(
                o:
                    | DolphinWindowOptions
                    | CemuWindowOptions
                    | CitraWindowOptions
                    | CustomWindowOptions,
            ): o is CustomWindowOptions {
                return !!(o as CustomWindowOptions).classes;
            }

            const option = options[0];
            const layoutOptions: MultiWindowLayout[] = [
                'column-right',
                'column-left',
                'square-right',
                'square-left',
                'separate',
            ];
            const dolphinLimitedLayoutOptions: LimitedMultiWindowLayout[] = [
                'column-right',
                'column-left',
                'square-right',
                'square-left',
            ];
            const dsLimitedLayoutOptions: MultiWindowLayout[] = [
                'column-right',
                'column-left',
            ];
            const limitedLayoutOptions = isDolphin(option)
                ? dolphinLimitedLayoutOptions
                : dsLimitedLayoutOptions;

            function DolphinAction(option: DolphinWindowOptions): ReactElement {
                return (
                    <>
                        <Builder
                            indentLevel={indentLevel}
                            label="Multi-Screen Layout"
                            description="Layout when the Deck's embedded display is enabled and an external display is connected."
                        >
                            <Fragment />
                        </Builder>
                        <Builder
                            indentLevel={indentLevel + 1}
                            label="Single-GBA Layout"
                            description="Layout when a single GBA window is visible."
                        >
                            <Dropdown
                                selectedOption={
                                    option.multi_screen_single_secondary_layout
                                }
                                rgOptions={layoutOptions.map((a) => {
                                    return {
                                        label: labelForKebabCase(a),
                                        data: a,
                                    };
                                })}
                                onChange={(value) => {
                                    option.multi_screen_single_secondary_layout =
                                        value.data;
                                    onChange(cloned);
                                }}
                            />
                        </Builder>
                        <Builder
                            indentLevel={indentLevel + 1}
                            label="Multi-GBA Layout"
                            description="Layout when multiple GBA windows are visible."
                        >
                            <Dropdown
                                selectedOption={
                                    option.multi_screen_multi_secondary_layout
                                }
                                rgOptions={layoutOptions.map((a) => {
                                    return {
                                        label: labelForKebabCase(a),
                                        data: a,
                                    };
                                })}
                                onChange={(value) => {
                                    option.multi_screen_multi_secondary_layout =
                                        value.data;
                                    onChange(cloned);
                                }}
                            />
                        </Builder>
                    </>
                );
            }

            function CustomAction(option: CustomWindowOptions): ReactElement {
                // Note: classes, etc. aren't set by the user, but instead reconfigured at runtime
                // In the future, we may consider using the fields as overrides to the automatic scraping

                return (
                    <>
                        <Builder
                            indentLevel={indentLevel}
                            label="Multi-Screen Layout"
                            description="Layout when the Deck's embedded display is enabled and an external display is connected."
                        >
                            <Fragment />
                        </Builder>
                        <Builder
                            indentLevel={indentLevel + 1}
                            label="Single-Window Layout"
                            description="Layout when a single alternate window is visible."
                        >
                            <Dropdown
                                selectedOption={
                                    option.multi_screen_single_secondary_layout
                                }
                                rgOptions={layoutOptions.map((a) => {
                                    return {
                                        label: labelForKebabCase(a),
                                        data: a,
                                    };
                                })}
                                onChange={(value) => {
                                    option.multi_screen_single_secondary_layout =
                                        value.data;
                                    onChange(cloned);
                                }}
                            />
                        </Builder>
                        <Builder
                            indentLevel={indentLevel + 1}
                            label="Multi-Window Layout"
                            description="Layout when multiple alternate windows are visible."
                        >
                            <Dropdown
                                selectedOption={
                                    option.multi_screen_multi_secondary_layout
                                }
                                rgOptions={layoutOptions.map((a) => {
                                    return {
                                        label: labelForKebabCase(a),
                                        data: a,
                                    };
                                })}
                                onChange={(value) => {
                                    option.multi_screen_multi_secondary_layout =
                                        value.data;
                                    onChange(cloned);
                                }}
                            />
                        </Builder>
                    </>
                );
            }

            function DsAction(
                option: CemuWindowOptions | CitraWindowOptions,
            ): ReactElement {
                return (
                    <Builder
                        indentLevel={indentLevel}
                        label="Multi-Screen Layout"
                        description="Layout when the Deck's embedded display is enabled and an external display is connected."
                    >
                        <Dropdown
                            selectedOption={option.multi_screen_layout}
                            rgOptions={layoutOptions.map((a) => {
                                return {
                                    label: labelForKebabCase(a),
                                    data: a,
                                };
                            })}
                            onChange={(value) => {
                                option.multi_screen_layout = value.data;
                                onChange(cloned);
                            }}
                        />
                    </Builder>
                );
            }

            return (
                <>
                    <Builder
                        indentLevel={indentLevel}
                        label="Keep Above"
                        description="Keep emulator windows above others."
                    >
                        <Toggle
                            value={cloned.value.general.keep_above}
                            onChange={(isEnabled) => {
                                cloned.value.general.keep_above = isEnabled;
                                onChange(cloned);
                            }}
                        />
                    </Builder>
                    <Builder
                        indentLevel={indentLevel}
                        label="Swap Screens"
                        description="Use the Deck's embedded display as the main display, instead of as the secondary display."
                    >
                        <Toggle
                            value={cloned.value.general.swap_screens}
                            onChange={(isEnabled) => {
                                cloned.value.general.swap_screens = isEnabled;
                                onChange(cloned);
                            }}
                        />
                    </Builder>
                    <Builder
                        indentLevel={indentLevel}
                        label="Single Screen Layout"
                        description="Layout when only the Deck's embedded display is available, or when an external display is connected while the Deck's embedded display is disabled."
                    >
                        <Dropdown
                            selectedOption={option.single_screen_layout}
                            rgOptions={limitedLayoutOptions.map((a) => {
                                return {
                                    label: labelForKebabCase(a),
                                    data: a,
                                };
                            })}
                            onChange={(value) => {
                                option.single_screen_layout = value.data;
                                onChange(cloned);
                            }}
                        />
                    </Builder>
                    {isDolphin(option)
                        ? DolphinAction(option)
                        : isCustom(option)
                        ? CustomAction(option)
                        : DsAction(option)}
                </>
            );
        case 'LaunchSecondaryFlatpakApp': {
            return (
                <SecondaryFlatpakApp
                    cloned={cloned}
                    indentLevel={indentLevel}
                    Builder={Builder}
                    onChange={onChange}
                />
            );
        }
        case 'LaunchSecondaryAppPreset': {
            return (
                <SecondaryAppPreset
                    cloned={cloned}
                    indentLevel={indentLevel}
                    Builder={Builder}
                    onChange={onChange}
                />
            );
        }
        case 'MainAppAutomaticWindowing':
            const general = cloned.value.general;
            const gamescope = cloned.value.gamescope;

            const fullscreenOptions: GamescopeFullscreenOption[] = [
                'Borderless',
                'Fullscreen',
            ];
            const scalerOptions: GamescopeScaler[] = [
                'Auto',
                'Integer',
                'Fit',
                'Stretch',
                'Fill',
            ];
            const filterOptions: GamescopeFilter[] = [
                'Linear',
                'Pixel',
                'Fsr',
                'Nis',
            ];

            return (
                <>
                    <Builder
                        indentLevel={indentLevel}
                        label="Keep Above"
                        description="Keep app windows above others."
                    >
                        <Toggle
                            value={general.keep_above}
                            onChange={(isEnabled) => {
                                general.keep_above = isEnabled;
                                onChange(cloned);
                            }}
                        />
                    </Builder>
                    <Builder
                        indentLevel={indentLevel}
                        label="Swap Screens"
                        description="Use the Deck's embedded display as the main display, instead of as the secondary display."
                    >
                        <Toggle
                            value={general.swap_screens}
                            onChange={(isEnabled) => {
                                general.swap_screens = isEnabled;
                                onChange(cloned);
                            }}
                        />
                    </Builder>
                    <Builder
                        indentLevel={indentLevel}
                        label="Use Gamescope"
                        description="Launch the game within gamescope. Fixes windowing issues for most titles."
                    >
                        <Toggle
                            value={gamescope.use_gamescope}
                            onChange={(isEnabled) => {
                                gamescope.use_gamescope = isEnabled;
                                onChange(cloned);
                            }}
                        />
                    </Builder>
                    {gamescope.use_gamescope ? (
                        <>
                            <Builder
                                indentLevel={indentLevel + 1}
                                label="Fullscreen Mode"
                                description="Fullscreen mode to use for the app window."
                            >
                                <Dropdown
                                    selectedOption={gamescope.fullscreen_option}
                                    rgOptions={fullscreenOptions.map((f) => {
                                        return {
                                            label: labelForCamelCase(f),
                                            data: f,
                                        };
                                    })}
                                    onChange={(v) => {
                                        gamescope.fullscreen_option = v.data;
                                        onChange(cloned);
                                    }}
                                />
                            </Builder>
                            <Builder
                                indentLevel={indentLevel + 1}
                                label="Scaling Mode"
                                description="Scaling mode to use for the app window."
                            >
                                <Dropdown
                                    selectedOption={gamescope.scaler}
                                    rgOptions={scalerOptions.map((f) => {
                                        return {
                                            label: labelForCamelCase(f),
                                            data: f,
                                        };
                                    })}
                                    onChange={(v) => {
                                        gamescope.scaler = v.data;
                                        onChange(cloned);
                                    }}
                                />
                            </Builder>
                            <Builder
                                indentLevel={indentLevel + 1}
                                label="Scaling Filter"
                                description="Scaling filter to use for the app window."
                            >
                                <Dropdown
                                    selectedOption={gamescope.filter}
                                    rgOptions={filterOptions.map((f) => {
                                        return {
                                            label: labelForCamelCase(f),
                                            data: f,
                                        };
                                    })}
                                    onChange={(v) => {
                                        gamescope.filter = v.data;
                                        onChange(cloned);
                                    }}
                                />
                            </Builder>
                            {gamescope.filter === 'Fsr' ? (
                                <SliderBuilder
                                    label="FSR Sharpness"
                                    value={gamescope.fsr_sharpness}
                                    indentLevel={indentLevel + 1}
                                    min={0}
                                    max={20}
                                    bottomSeparator="none"
                                    notchCount={5}
                                    showValue={true}
                                    onChange={(value) => {
                                        gamescope.fsr_sharpness = value;
                                        onChange(cloned);
                                    }}
                                />
                            ) : gamescope.filter === 'Nis' ? (
                                <SliderBuilder
                                    label="NIS Sharpness"
                                    value={gamescope.nis_sharpness}
                                    indentLevel={indentLevel + 1}
                                    min={0}
                                    max={20}
                                    bottomSeparator="none"
                                    notchCount={5}
                                    showValue={true}
                                    onChange={(value) => {
                                        gamescope.nis_sharpness = value;
                                        onChange(cloned);
                                    }}
                                />
                            ) : null}
                        </>
                    ) : null}
                </>
            );
        case 'TouchConfig':
            return (
                <Builder indentLevel={indentLevel} label="Touch Mode">
                    <Dropdown
                        selectedOption={cloned.value.touch_mode}
                        rgOptions={touchSelectionModeOptions.map((a) => {
                            return {
                                label: labelForCamelCase(a),
                                data: a,
                            };
                        })}
                        onChange={(option) => {
                            cloned.value.touch_mode = option.data;
                            onChange(cloned);
                        }}
                    />
                </Builder>
            );

        case 'VirtualScreen':
            return notConfigurable;
        case 'DesktopControllerLayoutHack':
            throw new Error('layout hack not currently configurable as action'); // TODO::fix this
        default:
            const typecheck: never = type;
            throw `action for edit failed to typecheck: ${typecheck}`;
    }
}

interface LaunchSecondaryFlatpakAppProps {
    cloned: {
        type: 'LaunchSecondaryFlatpakApp';
        value: LaunchSecondaryFlatpakApp;
    };
    indentLevel: number;
    onChange: (action: Action) => void;
    Builder: ActionChildBuilder;
}

function SecondaryFlatpakApp({
    cloned,
    indentLevel,
    onChange,
    Builder,
}: LaunchSecondaryFlatpakAppProps): ReactElement {
    const secondaryInfo = useSecondaryAppInfo();
    const [args, setArgs] = useState(cloned.value.app.args);

    return (
        <HandleLoading
            value={secondaryInfo}
            onOk={(secondaryInfo) => {
                const windowing = cloned.value.windowing_behavior;

                var i = 0;

                const textStyle: React.CSSProperties = {
                    width: '8rem',
                    borderTopRightRadius: 0,
                    borderBottomRightRadius: 0,
                };

                const displayArgs = args.map((arg) => {
                    const index = i++;
                    const deleteArg = () => {
                        args.splice(index, 1);
                        setArgs(args);
                        cloned.value.app.args = args;
                        onChange(cloned);
                    };

                    return (
                        <Focusable
                            style={{
                                display: 'flex',
                                flexDirection: 'row',
                                justifyContent: 'space-between',
                                alignItems: 'center',
                                marginBottom: '5px',
                            }}
                        >
                            <TextField
                                style={textStyle}
                                value={arg}
                                onChange={(v) => {
                                    args[index] = v.target.value;
                                    setArgs(args);
                                    cloned.value.app.args = args;
                                    onChange(cloned);
                                }}
                            />
                            <DialogButton
                                style={{
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
                    );
                });

                const addArg = () => {
                    args.push('');
                    setArgs(args);
                    cloned.value.app.args = args;
                    onChange(cloned);
                };

                return (
                    <>
                        <Builder
                            indentLevel={indentLevel}
                            label="Flatpak App"
                            description="The flatpak to run."
                        >
                            <Dropdown
                                selectedOption={cloned.value.app.app_id}
                                rgOptions={secondaryInfo.installed_flatpaks.map(
                                    (v) => {
                                        return {
                                            label: `${v.name} (${v.app_id})`,
                                            data: v.app_id,
                                        };
                                    },
                                )}
                                onChange={(value) => {
                                    cloned.value.app.app_id = value.data;
                                    onChange(cloned);
                                }}
                            />
                        </Builder>
                        <Builder
                            indentLevel={indentLevel}
                            label="Args"
                            description="Arguments for the flatpak app."
                        >
                            <>
                                {displayArgs}
                                <DialogButton
                                    onOKButton={addArg}
                                    onClick={addArg}
                                >
                                    <FaPlus />
                                </DialogButton>
                            </>
                        </Builder>
                        <Builder
                            indentLevel={indentLevel - 1}
                            label="Windowing"
                        >
                            <Dropdown
                                selectedOption={windowing}
                                rgOptions={secondaryAppWindowingOptions.map(
                                    (a) => {
                                        return {
                                            label: labelForCamelCase(a),
                                            data: a,
                                        };
                                    },
                                )}
                                onChange={(value) => {
                                    cloned.value.windowing_behavior =
                                        value.data;
                                    onChange(cloned);
                                }}
                            />
                        </Builder>
                    </>
                );
            }}
        />
    );
}

interface SecondaryAppPresetProps {
    cloned: {
        type: 'LaunchSecondaryAppPreset';
        value: LaunchSecondaryAppPreset;
    };
    indentLevel: number;
    onChange: (action: Action) => void;
    Builder: ActionChildBuilder;
}

function SecondaryAppPreset({
    cloned,
    indentLevel,
    onChange,
    Builder,
}: SecondaryAppPresetProps): ReactElement {
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
                    const flatpakInstalled =
                        preset.app.type === 'Flatpak' &&
                        secondaryInfo.installed_flatpaks.find(
                            (v) => v.app_id === preset.app.app_id,
                        );
                    const shouldPush = !filtered || flatpakInstalled;

                    if (shouldPush) {
                        options.push({
                            label: preset.name,
                            data: k,
                        });
                    }
                }

                options.sort((a, b) => a.label.localeCompare(b.label));

                return (
                    <>
                        <Builder
                            indentLevel={indentLevel}
                            label="Filter By Installed"
                        >
                            <Toggle
                                value={filtered}
                                onChange={(isEnabled) => {
                                    setFiltered(isEnabled);
                                }}
                            />
                        </Builder>
                        <Builder
                            indentLevel={indentLevel}
                            label="Selected Preset"
                        >
                            <Dropdown
                                selectedOption={cloned.value.preset}
                                rgOptions={options}
                                onChange={(value) => {
                                    cloned.value.preset = value.data;
                                    onChange(cloned);
                                }}
                            />
                        </Builder>
                        <Builder
                            indentLevel={indentLevel - 1}
                            label="Windowing"
                            description="Behavior of secondary app window(s)."
                        >
                            <Dropdown
                                selectedOption={cloned.value.windowing_behavior}
                                rgOptions={secondaryAppWindowingOptions.map(
                                    (a) => {
                                        return {
                                            label: labelForCamelCase(a),
                                            data: a,
                                        };
                                    },
                                )}
                                onChange={(value) => {
                                    cloned.value.windowing_behavior =
                                        value.data;
                                    onChange(cloned);
                                }}
                            />
                        </Builder>
                        {cloned.value.windowing_behavior === 'Fullscreen' ? (
                            <Builder
                                indentLevel={indentLevel - 1}
                                label="Screen Preference"
                                description="Screen to send secondary app window(s) to if windowing is fullscreen."
                            >
                                <Dropdown
                                    selectedOption={
                                        cloned.value.screen_preference
                                    }
                                    rgOptions={secondaryAppScreenPreferences.map(
                                        (a) => {
                                            return {
                                                label: labelForCamelCase(a),
                                                data: a,
                                            };
                                        },
                                    )}
                                    onChange={(value) => {
                                        cloned.value.screen_preference =
                                            value.data;
                                        onChange(cloned);
                                    }}
                                />
                            </Builder>
                        ) : null}
                    </>
                );
            }}
        />
    );
}

interface ExternalDisplaySettingsSelectorProps {
    indentLevel: number;
    settings: ExternalDisplaySettings;
    onChange: (settings: ExternalDisplaySettings) => void;
    Builder: ActionChildBuilder;
}

function ExternalDisplaySettingsSelector({
    indentLevel,
    settings,
    onChange,
    Builder,
}: ExternalDisplaySettingsSelectorProps): ReactElement {
    const displayInfo = useDisplayInfo();

    return (
        <HandleLoading
            value={displayInfo}
            onOk={(displayInfo) => {
                const fixed: ExternalDisplaySettings[] = [
                    { type: 'Previous' },
                    { type: 'Native' },
                ];

                const options: DropdownOption[] = fixed.map((setting) => {
                    return {
                        label: labelForCamelCase(setting.type),
                        data: setting,
                    };
                });

                // only show display options if we actually have some
                if (displayInfo != null && displayInfo.length > 0) {
                    options.push({
                        label: 'Custom',
                        options: displayInfo.map((info) => {
                            // note: most settings are "AtMost" in case the user changes displays without changing the plugin settings.

                            if (info.refresh) {
                                // If we have a refresh rate, use it
                                const value: ModePreference = {
                                    aspect_ratio: {
                                        type: 'Exact',
                                        value: info.width / info.height,
                                    },
                                    refresh: {
                                        type: 'AtMost',
                                        value: info.refresh,
                                    },
                                    resolution: {
                                        type: 'AtMost',
                                        value: {
                                            h: info.height,
                                            w: info.width,
                                        },
                                    },
                                };
                                return {
                                    label: `${info.width}x${
                                        info.height
                                    } @ ${info.refresh.toFixed(2)}`,
                                    data: {
                                        type: 'Preference',
                                        value,
                                    },
                                };
                            } else {
                                // If not, have the system scale up as high as possible
                                const value: ModePreference = {
                                    aspect_ratio: {
                                        type: 'Exact',
                                        value: info.width / info.height,
                                    },
                                    refresh: {
                                        type: 'AtMost',
                                        value: 2000.0,
                                    },
                                    resolution: {
                                        type: 'AtMost',
                                        value: {
                                            h: info.height,
                                            w: info.width,
                                        },
                                    },
                                };
                                return {
                                    label: `${info.width}x${info.height}`,
                                    data: {
                                        type: 'Preference',
                                        value,
                                    },
                                };
                            }
                        }),
                    });
                }
                function comparator(value: any, other: any) {
                    if (
                        typeof value === 'number' &&
                        typeof other === 'number'
                    ) {
                        const tolerance = 0.000001;
                        return Math.abs(value - other) < tolerance;
                    }

                    // Return undefined for default comparison behavior
                    return undefined;
                }

                var selected = options.find((v) => {
                    if (v.data != null) {
                        return _.isEqual(v.data, settings);
                    } else {
                        return v.options?.find((v) => {
                            const equal = _.isEqualWith(
                                v.data,
                                settings,
                                comparator,
                            );

                            return equal;
                        });
                    }
                });

                let defaultLabel =
                    settings.type === 'Preference'
                        ? `${settings.value.resolution.value.w}x${settings.value.resolution.value.h} (Previous Display)`
                        : undefined;

                if (selected?.options) {
                    selected = selected.options?.find((v) =>
                        _.isEqualWith(v.data, settings, comparator),
                    );
                }

                return (
                    <Builder
                        indentLevel={indentLevel}
                        label="External Display Settings"
                        description="Desired resolution of the external display."
                    >
                        <Dropdown
                            selectedOption={selected?.data}
                            rgOptions={options}
                            strDefaultLabel={defaultLabel}
                            onChange={(settings) => {
                                onChange(settings.data);
                            }}
                        />
                    </Builder>
                );
            }}
        />
    );
}

interface CemuAudioProps {
    indentLevel: number;
    settings: CemuAudio;
    onChange: (settings: CemuAudio) => void;
    Builder: ActionChildBuilder;
    SliderBuilder: ActionChildSliderBuilder;
}

function CemuAudioSelector({
    indentLevel,
    settings,
    onChange,
    Builder,
    SliderBuilder,
}: CemuAudioProps): ReactElement {
    const deviceInfo = useAudioDeviceInfo();

    return (
        <HandleLoading
            value={deviceInfo}
            onOk={(deviceInfo) => {
                const sources: {
                    label: string;
                    dir: string;
                    channelOptions: CemuAudioChannels[];
                    devices: AudioDeviceInfo[];
                    prefs: CemuAudioSetting;
                }[] = [
                    {
                        label: 'TV',
                        dir: 'out',
                        devices: deviceInfo.sinks,
                        channelOptions: ['Surround', 'Stereo', 'Mono'],
                        prefs: settings.state.tv_out,
                    },
                    {
                        label: 'Gamepad',
                        dir: 'out',
                        devices: deviceInfo.sinks,
                        channelOptions: ['Stereo'],
                        prefs: settings.state.pad_out,
                    },
                    {
                        label: 'Microphone',
                        dir: 'in',
                        devices: deviceInfo.sources,
                        channelOptions: ['Mono'],
                        prefs: settings.state.mic_in,
                    },
                ];
                return (
                    <>
                        {sources.map(
                            ({
                                channelOptions,
                                label,
                                dir,
                                prefs,
                                devices,
                            }) => {
                                const fixed: CemuDeviceOption[] = [
                                    { type: 'Disabled' },
                                    { type: 'Default' },
                                ];

                                const deviceOptions: DropdownOption[] =
                                    fixed.map((setting) => {
                                        return {
                                            label: labelForCamelCase(
                                                setting.type,
                                            ),
                                            data:
                                                setting.type === 'Default'
                                                    ? 'default'
                                                    : '',
                                        };
                                    });

                                // only show display options if we actually have some
                                if (devices.length > 0) {
                                    deviceOptions.push({
                                        label: 'Custom',
                                        options: devices.map((info) => {
                                            return {
                                                label: info.description,
                                                data: info.name,
                                            };
                                        }),
                                    });
                                }

                                let deviceSelected = deviceOptions.find((v) => {
                                    const data: string | null = v.data;

                                    if (data != null) {
                                        return prefs.device === data;
                                    } else {
                                        return v.options!.find((v) => {
                                            return v.data === prefs.device;
                                        });
                                    }
                                });

                                if (deviceSelected?.label === 'Custom') {
                                    deviceSelected =
                                        deviceSelected.options!.find((v) => {
                                            return v.data === prefs.device;
                                        });
                                }

                                const defaultLabel = 'Default'; // We use default anyway if missing, may as well show it.

                                const isDisabled = deviceSelected?.data === '';

                                return (
                                    <>
                                        <Builder
                                            indentLevel={indentLevel}
                                            label={`${label} Device`}
                                            description={`Device preferences for Cemu's ${label} ${dir}put. If selected device is not available, Cemu default will be used. `}
                                        >
                                            <Dropdown
                                                rgOptions={deviceOptions}
                                                strDefaultLabel={defaultLabel}
                                                selectedOption={
                                                    deviceSelected?.data
                                                }
                                                onChange={(value) => {
                                                    prefs.device = value.data;
                                                    onChange(settings);
                                                }}
                                            />
                                        </Builder>
                                        {!isDisabled ? (
                                            <>
                                                <Builder
                                                    label="Channels"
                                                    description="Cemu audio channel configuration. Does not affect system audio channel configuration."
                                                    indentLevel={
                                                        indentLevel + 1
                                                    }
                                                >
                                                    <Dropdown
                                                        rgOptions={channelOptions.map(
                                                            (c) => {
                                                                return {
                                                                    label: c,
                                                                    data: c,
                                                                };
                                                            },
                                                        )}
                                                        selectedOption={
                                                            prefs.channels
                                                        }
                                                        onChange={(value) => {
                                                            prefs.channels =
                                                                value.data;
                                                            onChange(settings);
                                                        }}
                                                    />
                                                </Builder>
                                                <div
                                                    style={{
                                                        paddingRight: '15px',
                                                    }}
                                                >
                                                    <SliderBuilder
                                                        key={'cemu-volume'}
                                                        indentLevel={
                                                            indentLevel + 1
                                                        }
                                                        label="Volume"
                                                        notchCount={5}
                                                        bottomSeparator="none"
                                                        step={5}
                                                        value={prefs.volume}
                                                        min={0}
                                                        max={100}
                                                        showValue={true}
                                                        minimumDpadGranularity={
                                                            1
                                                        }
                                                        valueSuffix="%"
                                                        onChange={(value) => {
                                                            prefs.volume =
                                                                value;
                                                            onChange(settings);
                                                        }}
                                                    />
                                                </div>
                                            </>
                                        ) : null}
                                    </>
                                );
                            },
                        )}
                    </>
                );
            }}
        />
    );
}

type CemuDeviceOption =
    | { type: 'Disabled' }
    | { type: 'Default' }
    | { type: 'Custom'; value: string };
