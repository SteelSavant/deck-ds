import { Field, Navigation, useParams } from "decky-frontend-lib";
import { ReactElement, useState } from "react";
import { PipelineContainer, createProfile } from "../../../backend";
import HandleLoading from "../../../components/HandleLoading";
import { ModifiablePipelineContainerProvider, useModifiablePipelineContainer } from "../../../context/modifiablePipelineContext";
import { useServerApi } from "../../../context/serverApiContext";
import useTemplate from "../../../hooks/useTemplate";
import PipelineDisplay from "../../PipelineDisplay";
import TemplateInfo from "./TemplateInfo";


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
                        <ModifiablePipelineContainerProvider initialContainer={template} >
                            <TemplatePreviewLogic />
                        </ModifiablePipelineContainerProvider>
                    );
                }
            }
        }
    />;
}


function TemplatePreviewLogic(): ReactElement {
    const { state } = useModifiablePipelineContainer();
    const [waiting, setWaiting] = useState(false);
    const serverApi = useServerApi();

    return <PipelineDisplay
        header={TemplateHeader}
        info={TemplateInfo}
        secondaryActionDescription="Save as New Profile"
        secondaryAction={async () => {
            if (!waiting) {
                setWaiting(true);
                const response = await createProfile({ pipeline: state.container.pipeline });

                if (response.isOk) {
                    const route = `/deck-ds/settings/profiles/${response.data.profile_id}`;
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


function TemplateHeader(container: PipelineContainer): ReactElement {
    return (<Field label={<h3>{`${container.pipeline.name} (Template)`}</h3>} bottomSeparator="thick">
        Changes made to this template will not be saved.
    </Field>);
}