import { DialogButton, Field, showModal, useParams } from "decky-frontend-lib";
import { ReactElement } from "react";
import { FaEdit } from "react-icons/fa";
import { Pipeline, setProfile } from "../../../backend";
import HandleLoading from "../../../components/HandleLoading";
import { ModifiablePipelineDefinitionProvider, useModifiablePipelineDefinition } from "../../../context/modifiablePipelineContext";
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
                        <ModifiablePipelineDefinitionProvider initialDefinition={profile.pipeline} onUpdate={async (pipeline) => {
                            const res = await setProfile({
                                profile: {
                                    id: profileid,
                                    tags: profile.tags,
                                    pipeline: pipeline
                                }
                            });

                            if (!res.isOk) {
                                serverApi.toaster.toast({
                                    title: 'Error',
                                    body: 'Failed to update profile.'
                                });
                            }
                        }} >
                            <PipelineDisplay header={PipelineHeader} info={ProfileInfo} />
                        </ModifiablePipelineDefinitionProvider>
                    );
                }
            }
        }
    />;
}

function PipelineHeader(pipeline: Pipeline): ReactElement {
    const { dispatch } = useModifiablePipelineDefinition();

    function onEditTitle() {
        showModal(
            <EditProfileNameModal pipeline={pipeline} onSave={(name) => {
                dispatch({
                    type: 'updatePipelineInfo',
                    info: {
                        ...pipeline,
                        name: name,
                    }
                })
            }} />
        )
    }

    return (
        <Field focusable={false} label={<h3>{pipeline.name}</h3>} bottomSeparator="thick" >
            <DialogButton onOKButton={onEditTitle} onClick={onEditTitle}>
                <FaEdit />
            </DialogButton>
        </Field>
    )
}
