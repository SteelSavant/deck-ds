import { Field } from "decky-frontend-lib";
import { ReactElement } from "react";
import { PipelineDefinition } from "../../backend";

export default function TemplateInfo({ template }: { template: PipelineDefinition }): ReactElement {
    return <div>
        {template.name}
        {template.description}
        <Field label="Tags">
            {template.tags.map((t) => <div>{t}</div>)}
        </Field>
    </div>

    // TODO:: dependencies section
}