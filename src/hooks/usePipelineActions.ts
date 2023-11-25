import { useEffect, useState } from "react";
import { PipelineActionDefinition, getPipelineActions } from "../backend";
import { Loading } from "../util/loading";

const usePipelineActions = (): Loading<{ [k: string]: PipelineActionDefinition }> => {
    const [result, setResult] = useState<Loading<{ [k: string]: PipelineActionDefinition }>>(null);

    useEffect(() => {
        let active = true;

        if (result === null) {
            (async function load() {
                const res = await getPipelineActions();

                if (!active) {
                    return;
                }

                setResult(res.map((v) => {
                    return v.pipeline_actions;
                }));
            })();
        }

        return () => { active = false; };
    });

    return result;
}

export default usePipelineActions;