import { ReactElement } from "react";
import { PipelineActionDefinition } from "../../backend";

export default function Pipeline({ root }: { root: PipelineActionDefinition }): ReactElement {
    return (
        <div>
            {root.name}
            {root.description}
        </div>
    )
}
