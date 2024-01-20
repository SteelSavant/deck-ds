import { DropdownItem, PanelSection, PanelSectionRow } from "decky-frontend-lib";
import { ReactElement, useState } from "react";
import { setAppProfileSettings } from "../backend";
import { ShortAppDetails } from "../context/shortAppDetailsContext";
import { LaunchActions } from "../hooks/useLaunchActions";
import { AppProfile } from "../types/backend_api";

export default function AppDefaultProfileDropdown({
    appDetails,
    appProfile,
    launchActions
}: {
    appDetails: ShortAppDetails
    appProfile: AppProfile | null
    launchActions: LaunchActions[],
}): ReactElement {
    const useAppDefault = launchActions
        .find((a) => a.profile.id == appProfile?.default_profile);

    const [selected, setSelected] = useState(useAppDefault ?
        appProfile?.default_profile : null);

    console.log("checking with", launchActions.length, "actions");

    if (launchActions.length > 1) {
        return (
            <PanelSection title="Default Profile">
                <PanelSectionRow>
                    <DropdownItem selectedOption={selected} rgOptions={
                        [
                            {
                                label: 'Default',
                                data: null
                            },
                            ...launchActions.map((a) => {
                                return {
                                    label: a.profile.pipeline.name,
                                    data: a.profile.id
                                }
                            })
                        ]}
                        onChange={async (option) => {
                            const res = await setAppProfileSettings({
                                app_id: appDetails.appId.toString(),
                                default_profile: option.data
                            });
                            if (res?.isOk) {
                                setSelected(option.data);
                            }
                        }} />
                </PanelSectionRow >
            </PanelSection >)
    } else {
        return <div />
    }
}