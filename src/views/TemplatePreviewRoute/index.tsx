import { useParams } from "decky-frontend-lib";
import { ReactElement } from "react";
import HandleLoading from "../../components/HandleLoading";
import { ModifiablePipelineDefinitionProvider } from "../../context/modifiablePipelineContext";
import useTemplate from "../../hooks/useTemplate";
import PipelineDisplay from "../PipelineDisplay";


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
                            <PipelineDisplay />
                        </ModifiablePipelineDefinitionProvider>
                    );
                }
            }
        }
    />;
}

