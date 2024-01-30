import { Field, Toggle } from "decky-frontend-lib";
import { Fragment, VFC } from "react";
import EditAction from "../../../components/EditAction";
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
                    <EditAction
                        action={{ type: 'UIManagement', value: settings.display_restoration }}
                        indentLevel={1}
                        onChange={(action) => {
                            if (action.type !== 'UIManagement') {
                                throw 'display settings are incorrect type; something has gone terribly wrong...'
                            }
                            updateSettings({
                                ...settings,
                                display_restoration: action.value
                            })
                        }} />
                    <Field
                        focusable={false}
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
                </Fragment>
            )
        }}
    />
}
