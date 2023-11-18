import { VFC, useEffect, useState } from 'react';
import { PipelineDefinition, getTemplates } from "../../backend";


export const TemplatesPage: VFC = () => {
    const [loading, setLoading] = useState(true);
    const [templates, setTemplates] = useState(Array<PipelineDefinition>);

    useEffect(() => {
        const loadTemplates = async () => {
            setLoading(true);

            const response = await getTemplates();

            if (response.ok) {
                console.log("Got ", response.data.templates.length, " templates");

                setTemplates(response.data.templates);
                setLoading(false);
            } else {
                console.log(response.err);

                setTimeout(() => {
                    loadTemplates();
                }, 5000);
            }
        }
    });


    return <div>
        <div> Templates</div>
        {loading
            ? <div> Loading...</div>
            : <div> Got {templates.length} Templates!</div>}
    </div>
}