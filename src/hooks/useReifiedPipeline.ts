import { useEffect, useState } from "react";
import { reifyPipeline } from "../backend";
import { Pipeline, PipelineDefinition } from "../types/backend_api";
import { Loading } from "../util/loading";


const useReifiedPipeline = (definition: PipelineDefinition): Loading<Pipeline> => {
    const [result, setResult] = useState<Loading<Pipeline>>(null);
    useEffect(() => {
        let active = true;

        if (result === null) {
            (async function load() {
                const res = await reifyPipeline({
                    pipeline: definition
                });

                if (!active) {
                    return;
                }

                setResult(res.map((r) => r.pipeline));
            })();
        }

        return () => { active = false; };
    }, [definition]);

    return result;
}

export default useReifiedPipeline;


