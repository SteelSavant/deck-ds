import { ConfirmModal, DialogButton, Field, Focusable, Navigation, showModal } from "decky-frontend-lib";
import { ReactElement } from "react";
import { FaGear, FaTrash } from "react-icons/fa6";
import { Profile, Response, } from "../../../backend";
import { useServerApi } from "../../../context/serverApiContext";
import { DeleteProfileRequest } from "../../../types/backend_api";

export default function ProfileMenuItem({ profile, deleteProfile }: { profile: Profile, deleteProfile: (request: DeleteProfileRequest) => Response<void> }): ReactElement {
    const serverApi = useServerApi();

    function viewProfile() {
        const route = `/deck-ds/settings/profiles/${profile.id}`;
        console.log("Navigating to", route);
        Navigation.Navigate(route);
    }

    function deleteProfileWithConfirmation() {
        showModal((
            <ConfirmModal
                strTitle="Delete Profile"
                strDescription="Are you sure you want to delete this Profile?"
                strOKButtonText="Delete"
                bMiddleDisabled={true}
                bDestructiveWarning={true}
                onOK={async () => {
                    const res = await deleteProfile({ profile: profile.id })
                    if (!res.isOk) {
                        serverApi.toaster.toast({
                            title: 'Error',
                            body: `Failed to delete profile.`
                        })
                    }
                }} />
        ));
    }

    return (
        <Field
            focusable={false}
            label={profile.pipeline.name}
            description={profile.pipeline.description}
        >
            <div style={{ display: 'flex', width: '100%', position: 'relative' }}>

                <Focusable >
                    <DialogButton
                        style={{ height: '40px', minWidth: '40px', marginRight: '10px' }}
                        onClick={viewProfile}
                        onOKButton={viewProfile}
                    >
                        <div style={{ display: 'flex', minWidth: '100px', justifyContent: 'space-between', alignItems: 'center' }}>
                            <FaGear style={{ paddingRight: '1rem' }} />
                            Edit
                        </div>
                    </DialogButton>
                </ Focusable>
                <Focusable>
                    <DialogButton style={{
                        backgroundColor: 'red',
                        height: '40px',
                        width: '40px',
                        padding: '10px 12px',
                        minWidth: '40px',
                        display: 'flex',
                        flexDirection: 'column',
                        justifyContent: 'center',
                        marginLeft: '10px'
                    }}
                        onOKButton={deleteProfileWithConfirmation}
                        onClick={deleteProfileWithConfirmation}
                    >
                        <FaTrash />
                    </DialogButton>
                </Focusable>
            </div>
        </Field>
    );
}