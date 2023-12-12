import { DialogBody, DialogButton, DialogControlsSection, Navigation } from "decky-frontend-lib";
import { VFC } from "react";
import { FaPlus } from "react-icons/fa6";
import HandleLoading from "../../../components/HandleLoading";
import useProfiles from "../../../hooks/useProfiles";

export const ProfilesPage: VFC = () => {
    const profiles = useProfiles();

    const navigateToTemplates = () => Navigation.Navigate('/deck-ds/settings/templates');


    return <HandleLoading
        value={profiles}
        onOk={
            (profiles) => <DialogBody>
                <DialogControlsSection>
                    {profiles.length > 0 ? profiles.map((p) => p.pipeline.name) : 'No profiles have been created.'}
                    <div style={{ paddingTop: '30px', display: 'flex', minWidth: '100px', justifyContent: 'space-between', alignItems: 'center', }}>
                        <DialogButton onClick={navigateToTemplates} onOKButton={navigateToTemplates}>
                            <FaPlus style={{ paddingRight: '1rem' }} />
                            Create From Template
                        </DialogButton>
                    </div>
                </DialogControlsSection>
            </DialogBody>
        }
    />;
}

