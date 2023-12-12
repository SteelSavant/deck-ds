import { Navigation, useParams } from "decky-frontend-lib";
import { ReactElement, useState } from "react";
import { createProfile } from "../../../backend";
import HandleLoading from "../../../components/HandleLoading";
import { ModifiablePipelineDefinitionProvider, useModifiablePipelineDefinition } from "../../../context/modifiablePipelineContext";
import { useServerApi } from "../../../context/serverApiContext";
import useTemplate from "../../../hooks/useTemplate";
import PipelineDisplay from "../../PipelineDisplay";


export default function TemplatePreviewRoute(): ReactElement {
    const { templateid } = useParams<{ templateid: string }>()
    const template = useTemplate(templateid);

    return <HandleLoading
        value={template}
        onOk={
            (template) => {
                if (template === undefined) {
                    return <div> Template {templateid} does not exist!</div>;
                } else {
                    return (
                        <ModifiablePipelineDefinitionProvider initialDefinition={template.pipeline} >
                            <TemplatePreviewLogic />
                        </ModifiablePipelineDefinitionProvider>
                    );
                }
            }
        }
    />;
}


function TemplatePreviewLogic(): ReactElement {
    const { state } = useModifiablePipelineDefinition();
    const [waiting, setWaiting] = useState(false);
    const serverApi = useServerApi();

    return <PipelineDisplay
        secondaryActionDescription="Save as New Profile"
        secondaryAction={async () => {
            if (!waiting) {
                setWaiting(true);
                const response = await createProfile({ pipeline: state.definition });

                if (response.isOk) {
                    const route = `/deck-ds/settings/profiles/${response.data.profile_id}`;
                    console.log("Navigating to", route);
                    Navigation.Navigate(route);
                } else {
                    serverApi.toaster.toast({
                        title: 'Error',
                        body: 'Failed to save profile from template.'
                    })
                }

                setWaiting(false);
            }
        }}
    />
}
