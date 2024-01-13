import { Field } from "decky-frontend-lib";
import { ReactElement } from "react";
import { PipelineContainer } from "../../../backend";

export default function ProfileInfo(container: PipelineContainer): ReactElement {

    // TODO::make description editable
    // TODO::dependencies section
    // TODO::maybe have/show default tags?
    // TODO::maybe have some info/instructions in the blank space? Definitely if not showing dependencies
    return (
        <div>
            <Field focusable={false} description={container.pipeline.description} />
        </div>
    );
}

