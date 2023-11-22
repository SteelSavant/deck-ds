import { Focusable } from "decky-frontend-lib";
import { CSSProperties, ReactElement } from "react";
import { PipelineDefinition } from "../../backend";

export const TemplateInfoContainer: CSSProperties = {
    margin: "20px 20px 0px 20px",
    paddingBottom: "15px",
    flexDirection: "column",
    minWidth: "95%",
    display: "flex",
}

export default function TemplateInfo({ template }: { template: PipelineDefinition }): ReactElement {
    console.log("Template info rendering template", template);

    return (
        <Focusable style={TemplateInfoContainer} >
            <div>
                <h3>{template.name} </h3>
                <p>{template.description}</p>
                <h3>Default Tags
                    <ul>
                        {template.tags.map((t) => <li>{t}</li>)}
                    </ul>
                </h3>
            </div>
        </Focusable>
    );

    // TODO:: dependencies section
}