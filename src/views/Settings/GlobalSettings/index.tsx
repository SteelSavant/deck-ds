import { Dropdown, Field, Toggle } from 'decky-frontend-lib';
import { Fragment, VFC } from 'react';
import { ActionChild } from '../../../components/ActionChild';
import { EditAction } from '../../../components/EditAction';
import { EditExitHooks } from '../../../components/EditExitHooks';
import HandleLoading from '../../../components/HandleLoading';
import useGlobalSettings from '../../../hooks/useGlobalSettings';
import { LogLevel, logger } from '../../../util/log';

export const GlobalSettingsPage: VFC = () => {
    const { settings, updateSettings } = useGlobalSettings();

    const Builder = ActionChild;

    // TODO::global config for editing exit hooks

    return (
        <HandleLoading
            value={settings}
            onOk={(settings) => {
                return (
                    <>
                        <Field
                            label="Desktop Mode"
                            description={
                                'Configuration for actions run in desktop mode.'
                            }
                            focusable={true}
                        />
                        <Builder
                            label="Exit Hooks"
                            description="The button chord to hold to exit the app in desktop mode, if enabled."
                            indentLevel={1}
                            // TODO::consider dropdown arrow child to expand/hide the edit, since moving the "add chord button" into the field is inconvenient
                        />
                        <EditExitHooks
                            exitHooks={settings.exit_hooks}
                            indentLevel={2}
                            onChange={(hooks) => {
                                updateSettings({
                                    ...settings,
                                    exit_hooks: hooks,
                                });
                            }}
                        />
                        <Builder
                            label="Overwrite Desktop Controller Layout (Hack)"
                            indentLevel={1}
                            description="Overwrites the main desktop controller layout with the layout of the app being launched, in case Steam fails to switch to the game's controller layout.
                        Currently only works for the Deck's built-in  controller."
                        >
                            <Toggle
                                value={
                                    settings.use_desktop_controller_layout_hack
                                }
                                onChange={(value) => {
                                    updateSettings({
                                        ...settings,
                                        use_desktop_controller_layout_hack:
                                            value,
                                    });
                                }}
                            />
                        </Builder>
                        <Field
                            focusable={false}
                            label="Display Settings"
                            description="Settings to apply when restoring the desktop displays after an app launch."
                        />
                        <EditAction
                            action={{
                                type: 'DesktopSessionHandler',
                                value: settings.display_restoration,
                            }}
                            indentLevel={1}
                            onChange={(action) => {
                                if (action.type !== 'DesktopSessionHandler') {
                                    throw 'display settings are incorrect type; something has gone terribly wrong...';
                                }
                                updateSettings({
                                    ...settings,
                                    display_restoration: action.value,
                                });
                            }}
                        />
                        <Builder
                            indentLevel={1}
                            label="Apply Display Settings When Opening Desktop"
                            description="Apply display settings when switching to desktop normally, not just when restoring the displays from an app launch."
                        >
                            <Toggle
                                value={
                                    settings.restore_displays_if_not_executing_pipeline
                                }
                                onChange={(value) => {
                                    updateSettings({
                                        ...settings,
                                        restore_displays_if_not_executing_pipeline:
                                            value,
                                    });
                                }}
                            />
                        </Builder>
                        <Field label="Deck UI" />
                        <Builder
                            indentLevel={1}
                            label="Enable UI Patching"
                            description="Allow patching game pages to have custom buttons to launch DeckDS profiles, instead of having to launch from the Quick Acess Menu."
                        >
                            <Toggle
                                value={settings.enable_ui_inject}
                                onChange={(value) => {
                                    updateSettings({
                                        ...settings,
                                        enable_ui_inject: value,
                                    });
                                }}
                            />
                        </Builder>
                        <Builder
                            indentLevel={1}
                            label="Primary Target"
                            description="Determines which target is used by the primary 'Play' button when patching the UI"
                        >
                            <Dropdown
                                selectedOption={settings.primary_ui_target}
                                rgOptions={['Gamemode', 'Desktop'].map((t) => {
                                    return {
                                        label: t,
                                        data: t,
                                    };
                                })}
                                onChange={
                                    settings.enable_ui_inject
                                        ? (data) => {
                                              updateSettings({
                                                  ...settings,
                                                  primary_ui_target: data.data,
                                              });
                                          }
                                        : undefined
                                }
                            />
                        </Builder>
                        <Field label="Debug" />
                        <Builder
                            indentLevel={1}
                            label="Log Level"
                            description="Sets the log level for both the frontend and backend. Useful for debugging. Don't touch this unless you need to."
                        >
                            <Dropdown
                                selectedOption={settings.log_level}
                                rgOptions={[
                                    LogLevel.Trace,
                                    LogLevel.Debug,
                                    LogLevel.Info,
                                    LogLevel.Warn,
                                    LogLevel.Error,
                                ].map((l) => {
                                    const label = (function () {
                                        switch (l) {
                                            case LogLevel.Trace:
                                                return 'Trace';
                                            case LogLevel.Debug:
                                                return 'Debug';
                                            case LogLevel.Info:
                                                return 'Info';
                                            case LogLevel.Warn:
                                                return 'Warn';
                                            case LogLevel.Error:
                                                return 'Error';
                                        }
                                    })();

                                    return {
                                        label,
                                        data: l,
                                    };
                                })}
                                onChange={(props) => {
                                    logger.minLevel = props.data;
                                    updateSettings({
                                        ...settings,
                                        log_level: props.data,
                                    });
                                }}
                            />
                        </Builder>
                    </>
                );
            }}
        />
    );
};
