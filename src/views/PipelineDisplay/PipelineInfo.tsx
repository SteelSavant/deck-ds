import { Focusable } from "decky-frontend-lib";
import { CSSProperties, ReactElement } from "react";
import { Pipeline } from "../../types/backend_api";

export const TemplateInfoContainer: CSSProperties = {
    margin: "20px 20px 0px 20px",
    paddingBottom: "15px",
    flexDirection: "column",
    minWidth: "95%",
    display: "flex",
}

export default function PipelineInfo({ pipeline }: { pipeline: Pipeline }): ReactElement {
    console.log("PipelineInfo rendering pipeline", pipeline);

    return (
        <Focusable style={TemplateInfoContainer} >
            <div>
                <h3>{pipeline.name} </h3>
                <p>{pipeline.description}</p>
                <h3>Default Tags
                    <ul>
                        {pipeline.tags.map((t) => <li>{t}</li>)}
                    </ul>
                </h3>
            </div>
        </Focusable>
    );

    // TODO:: dependencies section
}