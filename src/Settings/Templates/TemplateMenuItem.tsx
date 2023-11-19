import { ReactElement } from "react";
import { PipelineDefinition } from "../../backend";

export default function TemplateMenuItem({ template }: { template: PipelineDefinition }): ReactElement {

    return <div>
        <div> {template.name}</div>
        <div> {template.description} </div>
    </div>
}