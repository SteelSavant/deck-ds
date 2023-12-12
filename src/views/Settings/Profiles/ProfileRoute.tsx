import { useParams } from "decky-frontend-lib";
import { ReactElement } from "react";
import { setProfile } from "../../../backend";
import HandleLoading from "../../../components/HandleLoading";
import { ModifiablePipelineDefinitionProvider } from "../../../context/modifiablePipelineContext";
import { useServerApi } from "../../../context/serverApiContext";
import useProfile from "../../../hooks/useProfile";
import PipelineDisplay from "../../PipelineDisplay";


export default function ProfilePreviewRoute(): ReactElement {
    const { profileid } = useParams<{ profileid: string }>()
    const profile = useProfile(profileid);

    const serverApi = useServerApi();

    return <HandleLoading
        value={profile}
        onOk={
            (profile) => {
                if (profile === undefined) {
                    return <div> Profile {profileid} does not exist!</div>;
                } else {
                    return (
                        <ModifiablePipelineDefinitionProvider initialDefinition={profile.pipeline} onUpdate={async (pipeline) => {
                            const res = await setProfile({
                                profile: {
                                    id: profileid,
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
                            <PipelineDisplay />
                        </ModifiablePipelineDefinitionProvider>
                    );
                }
            }
        }
    />;
}

