import { ConfirmModal, DialogButton, Field, Navigation, showModal } from "decky-frontend-lib";
import { ReactElement } from "react";
import { FaGear, FaTrash } from "react-icons/fa6";
import { CategoryProfile, DeleteProfileRequest, Response, } from "../../../backend";
import FocusableRow from "../../../components/FocusableRow";
import { useServerApi } from "../../../context/serverApiContext";

export default function ProfileMenuItem({ profile, deleteProfile }: { profile: CategoryProfile, deleteProfile: (request: DeleteProfileRequest) => Response<void> }): ReactElement {
    const serverApi = useServerApi();

    function viewProfile() {
        const route = `/deck-ds/settings/profiles/${profile.id}`;
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
        // description={profile.pipeline.description}
        >
            <FocusableRow>
                <DialogButton
                    style={{ height: '40px', minWidth: '60px', marginRight: '10px' }}
                    onClick={viewProfile}
                    onOKButton={viewProfile}
                >
                    <div style={{ display: 'flex', minWidth: '100px', justifyContent: 'space-between', alignItems: 'center' }}>
                        <FaGear style={{ paddingRight: '1rem' }} />
                        Edit
                    </div>
                </DialogButton>
                <DialogButton style={{
                    backgroundColor: 'red',
                    height: '40px',
                    width: '40px',
                    padding: '10px 12px',
                    minWidth: '40px',
                    display: 'flex',
                    flexDirection: 'column',
                    justifyContent: 'center',
                }}
                    onOKButton={deleteProfileWithConfirmation}
                    onClick={deleteProfileWithConfirmation}
                >
                    <FaTrash />
                </DialogButton>
            </FocusableRow>
        </Field>
    );
}