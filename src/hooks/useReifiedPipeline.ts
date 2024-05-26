import { useEffect, useState } from 'react';
import {
    PipelineDefinition,
    ReifyPipelineResponse,
    reifyPipeline,
} from '../backend';
import { Loading } from '../util/loading';

const useReifiedPipeline = (
    definition: PipelineDefinition,
): Loading<ReifyPipelineResponse> => {
    const [result, setResult] = useState<Loading<ReifyPipelineResponse>>(null);

    useEffect(() => {
        let active = true;

        (async function load() {
            const res = await reifyPipeline({
                pipeline: definition,
            });

            if (!active) {
                return;
            }

            setResult(res);
        })();

        return () => {
            active = false;
        };
    }, [definition]);

    return result;
};

export default useReifiedPipeline;
