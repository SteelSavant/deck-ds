import { DropdownItem, PanelSection, PanelSectionRow } from '@decky/ui';
import { ReactElement } from 'react';
import { AppProfile } from '../../backend';
import { ShortAppDetails, useAppState } from '../../context/appContext';
import { LaunchActions } from '../../hooks/useLaunchActions';
import useProfiles from '../../hooks/useProfiles';

export default function AppDefaultProfileDropdown({
    appDetails,
    appProfile,
    launchActions,
}: {
    appDetails: ShortAppDetails;
    appProfile: AppProfile;
    launchActions: LaunchActions[];
}): ReactElement | null {
    const appDetailsState = useAppState();
    const { profiles } = useProfiles();

    const availableProfiles = profiles?.isOk ? profiles.data : [];

    if (launchActions.length <= 1) {
        return null;
    }

    const selected =
        launchActions.find((a) => a.profileId == appProfile.default_profile)
            ?.profileId ?? null;

    console.log(
        'selected profile:',
        selected,
        ', was actually found:',
        !!launchActions.find((a) => a.profileId == appProfile.default_profile),
    );

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
                                label:
                                    availableProfiles.find(
                                        (v) => v.id === a.profileId,
                                    )?.pipeline.name ?? '',
                                data: a.profileId,
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
}
