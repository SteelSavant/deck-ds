import { ReactElement } from "react";
import { PipelineDefinition } from "../../backend";

export default function Template({ template }: { template: PipelineDefinition }): ReactElement {

    return <div> Template {template.name}</div>
}