import { Dropdown, Field, Toggle } from "decky-frontend-lib";
import { Fragment, VFC } from "react";
import InternalEditAction from "../../../components/EditAction";
import HandleLoading from "../../../components/HandleLoading";
import useGlobalSettings from "../../../hooks/useGlobalSettings";

export const GlobalSettingsPage: VFC = () => {
    const { settings, updateSettings } = useGlobalSettings();

    // TODO:: make UI inject configurable
    return <HandleLoading
        value={settings}
        onOk={(settings) => {
            return (
                <Fragment>
                    <Field
                        focusable={false}
                        label="Display Settings"
                        description="Settings to apply when restoring the desktop displays after an app launch."
                    />
                    <InternalEditAction
                        action={{ type: 'DesktopSessionHandler', value: settings.display_restoration }}
                        indentLevel={1}
                        onChange={(action) => {
                            if (action.type !== 'DesktopSessionHandler') {
                                throw 'display settings are incorrect type; something has gone terribly wrong...'
                            }
                            updateSettings({
                                ...settings,
                                display_restoration: action.value
                            })
                        }} />
                    <Field
                        focusable={false}
                        indentLevel={1}
                        label="Apply Display Settings When Opening Desktop"
                        description="Apply display settings when opening the desktop for any reason, not just when restoring the displays from an app launch."
                    >
                        <Toggle value={settings.restore_displays_if_not_executing_pipeline}
                            onChange={(value) => {
                                updateSettings({
                                    ...settings,
                                    restore_displays_if_not_executing_pipeline: value
                                })
                            }}
                        />
                    </Field>
                    <Field label="Deck UI"></Field>
                    <Field
                        focusable={false}
                        indentLevel={1}
                        label="Enable UI Patching"
                        description="Allow patching game pages to have custom buttons to launch DeckDS profiles, instead of having to launch from the Quick Acess Menu."
                    >
                        <Toggle value={settings.enable_ui_inject}
                            onChange={(value) => {
                                updateSettings({
                                    ...settings,
                                    enable_ui_inject: value
                                })
                            }}
                        />
                    </Field>
                    <Field
                        focusable={false}
                        indentLevel={1}
                        label="Primary Target"
                        description="Determines which target is used by the primary 'Play' button when patching the UI"
                    >
                        <Dropdown
                            selectedOption={settings.primary_ui_target}
                            rgOptions={['Gamemode', 'Desktop'].map((t) => {
                                return {
                                    label: t,
                                    data: t
                                }
                            })}
                            onChange={settings.enable_ui_inject
                                ? (data) => {
                                    updateSettings({
                                        ...settings,
                                        primary_ui_target: data.data
                                    })
                                }
                                : undefined
                            }
                        />
                    </Field>
                </Fragment>
            )
        }}
    />
}
