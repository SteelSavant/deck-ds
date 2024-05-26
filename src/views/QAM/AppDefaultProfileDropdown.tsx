import {
    DropdownItem,
    PanelSection,
    PanelSectionRow,
} from 'decky-frontend-lib';
import { ReactElement } from 'react';
import { AppProfile } from '../../backend';
import { ShortAppDetails, useAppState } from '../../context/appContext';
import { LaunchActions } from '../../hooks/useLaunchActions';

export default function AppDefaultProfileDropdown({
    appDetails,
    appProfile,
    launchActions,
}: {
    appDetails: ShortAppDetails;
    appProfile: AppProfile;
    launchActions: LaunchActions[];
}): ReactElement {
    const appDetailsState = useAppState();

    const selected =
        launchActions.find((a) => a.profile.id == appProfile.default_profile)
            ?.profile?.id ?? null;

    if (launchActions.length > 1) {
        return (
            <PanelSection title="Default Profile">
                <PanelSectionRow>
                    <DropdownItem
                        selectedOption={selected}
                        rgOptions={[
                            {
                                label: 'Default',
                                data: null,
                            },
                            ...launchActions.map((a) => {
                                return {
                                    label: a.profile.pipeline.name,
                                    data: a.profile.id,
                                };
                            }),
                        ]}
                        onChange={async (option) => {
                            appDetailsState.setAppProfileDefault(
                                appDetails,
                                option.data,
                            );
                        }}
                    />
                </PanelSectionRow>
            </PanelSection>
        );
    } else {
        return <div />;
    }
}
