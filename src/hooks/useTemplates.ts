import { useEffect, useState } from "react";
import { PipelineDefinition, getTemplates } from "../backend";
import { Loading } from "../util/loading";

const useTemplates = (): Loading<Array<PipelineDefinition>> => {
    const [result, setResult] = useState<Loading<Array<PipelineDefinition>>>(null);

    useEffect(() => {
        let active = true;

        if (result === null) {
            (async function load() {
                const res = await getTemplates();

                if (!active) { return; }

                setResult(res.map((v) => v.templates));
            })();
        }

        return () => { active = false; };
    });

    return result;
}

export default useTemplates;