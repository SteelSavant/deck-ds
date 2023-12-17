import { DialogBody, DialogButton, DialogControlsSection, Navigation } from "decky-frontend-lib";
import { VFC } from "react";
import { FaPlus } from "react-icons/fa6";
import HandleLoading from "../../../components/HandleLoading";
import useProfiles from "../../../hooks/useProfiles";
import ProfileMenuItem from "./ProfileMenuItem";

export const ProfilesPage: VFC = () => {
    const { profiles, deleteProfile } = useProfiles();

    const navigateToTemplates = () => Navigation.Navigate('/deck-ds/settings/templates');

    // TODO:: make profiles reorderable
    return <HandleLoading
        value={profiles}
        onOk={
            (profiles) => <DialogBody>
                <DialogControlsSection>
                    {profiles.length > 0 ? profiles.map((p) => <ProfileMenuItem profile={p} deleteProfile={deleteProfile} />) : 'No profiles have been created.'}
                </DialogControlsSection>
                <div style={{ padding: '15px', display: 'flex', minWidth: '100px', justifyContent: 'space-between', alignItems: 'center', }}>
                    <DialogButton onClick={navigateToTemplates} onOKButton={navigateToTemplates}>
                        <FaPlus style={{ paddingRight: '1rem' }} />
                        Create From Template
                    </DialogButton>
                </div>
            </DialogBody>
        }
    />;
}

