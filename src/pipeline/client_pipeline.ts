import { Pipeline, PipelineTarget } from '../backend';
import { Result } from '../util/result';

export async function setupClientPipeline(
    pipeline: Pipeline,
    target: PipelineTarget,
): Promise<Result<void, string>> {
    const selection = pipeline.targets[target];
}

export async function teardownClientPipeline(): Promise<void> {
    // TODO::fetch actions from backend, then execute them
}
