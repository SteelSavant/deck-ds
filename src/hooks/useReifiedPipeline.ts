import { useEffect, useState } from "react";
import { Pipeline, PipelineDefinition, reifyPipeline } from "../backend";
import { Loading } from "../util/loading";


const useReifiedPipeline = (definition: PipelineDefinition): Loading<Pipeline> => {
    const [result, setResult] = useState<Loading<Pipeline>>(null);

    console.log("rebuild reified pipeline with:", definition);

    useEffect(() => {
        console.log("rerun reified pipeline effect");
        let active = true;

        (async function load() {
            const res = await reifyPipeline({
                pipeline: definition
            });

            if (!active) {
                return;
            }

            console.log("reify returned", res);

            setResult(res.map((r) => r.pipeline));
        })();

        return () => { active = false; };
    }, [definition]);

    return result;
}

export default useReifiedPipeline;


