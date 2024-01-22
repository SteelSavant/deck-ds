import { DialogButton, Field, showModal, useParams } from "decky-frontend-lib";
import { ReactElement } from "react";
import { FaEdit } from "react-icons/fa";
import { PipelineContainer, isCategoryProfile, setProfile } from "../../../backend";
import HandleLoading from "../../../components/HandleLoading";
import { ModifiablePipelineContainerProvider, useModifiablePipelineContainer } from "../../../context/modifiablePipelineContext";
import { useServerApi } from "../../../context/serverApiContext";
import useProfile from "../../../hooks/useProfile";
import PipelineDisplay from "../../PipelineDisplay";
import ProfileInfo from "./ProfileInfo";
import EditProfileNameModal from "./modals/EditPipelineNameModal";


export default function ProfilePreviewRoute(): ReactElement {
    const { profileid } = useParams<{ profileid: string }>()
    const profile = useProfile(profileid);

    const serverApi = useServerApi();

    return <HandleLoading
        value={profile}
        onOk={
            (profile) => {
                if (!profile) {
                    return <div> Profile {profileid} does not exist! Something has gone terribly wrong...</div>;
                } else {
                    return (
                        <ModifiablePipelineContainerProvider initialContainer={profile} onPipelineUpdate={async (profile) => {
                            if (!isCategoryProfile(profile)) {
                                throw 'PipelineContainer should be CategoryProfile'
                            }

                            const res = await setProfile({
                                profile: profile
                            });

                            if (!res.isOk) {
                                serverApi.toaster.toast({
                                    title: 'Error',
                                    body: 'Failed to update profile.'
                                });
                            }
                        }} >
                            <PipelineDisplay header={PipelineHeader} info={ProfileInfo} />
                        </ModifiablePipelineContainerProvider>
                    );
                }
            }
        }
    />;
}

function PipelineHeader(container: PipelineContainer): ReactElement {
    const { state, dispatch } = useModifiablePipelineContainer();

    function onEditTitle() {
        showModal(
            <EditProfileNameModal pipeline={state.container.pipeline} onSave={(name) => {
                dispatch({
                    externalProfile: null,
                    update: {
                        type: 'updatePipelineInfo',
                        info: {
                            ...container.pipeline,
                            name: name,
                        }
                    }
                })
            }} />
        )
    }

    return (
        <Field focusable={false} label={<h3>{container.pipeline.name}</h3>} bottomSeparator="thick" >
            <DialogButton onOKButton={onEditTitle} onClick={onEditTitle}>
                <FaEdit />
            </DialogButton>
        </Field>
    )
}
