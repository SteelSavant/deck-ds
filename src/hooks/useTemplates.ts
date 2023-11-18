import { useEffect, useState } from "react";
import { PipelineDefinition, getTemplates } from "../backend";
import { Loading, None, Some } from "../util/loading";

const useTemplates = (): Loading<Array<PipelineDefinition>> => {
    const [result, setResult] = useState<Loading<Array<PipelineDefinition>>>(None);

    useEffect(() => {
        let active = true;

        (async function load() {
            setResult(None);
            const res = await getTemplates();

            if (!active) { return; }

            setResult(Some(res.map((v) => v.templates)));
        })();

        return () => { active = false; };
    });

    return result;
}

export default useTemplates;