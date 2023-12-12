import { DialogButton, Field, Focusable, Navigation } from "decky-frontend-lib";
import { ReactElement } from "react";
import { FaGear } from "react-icons/fa6";
import { Profile } from "../../../backend";

export default function ProfileMenuItem({ profile }: { profile: Profile }): ReactElement {

    function viewProfile(profileId: string) {
        const route = `/deck-ds/settings/profiles/${profileId}`;
        console.log("Navigating to", route);
        Navigation.Navigate(route);
    }

    return (
        <Field focusable={false} label={profile.pipeline.name} description={profile.pipeline.description} children={
            <Focusable style={{ display: 'flex', width: '100%', position: 'relative' }}>
                <DialogButton
                    style={{ height: '40px', minWidth: '60px', marginRight: '10px' }}
                    onClick={() => viewProfile(profile.id)}
                    onOKButton={() => viewProfile(profile.id)}
                >
                    <div style={{ display: 'flex', minWidth: '100px', justifyContent: 'space-between', alignItems: 'center' }}>
                        <FaGear style={{ paddingRight: '1rem' }} />
                        Edit
                    </div>
                </DialogButton>
            </Focusable>
        } />
    );
}