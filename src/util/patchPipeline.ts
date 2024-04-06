import { Action, ApiError, PipelineDefinition, PipelineTarget, patchPipelineAction } from "../backend";
import { PipelineActionUpdate } from "../types/backend_api";
import { MaybeString } from "../types/short";
import { Ok, Result } from "./result";

export type PipelineUpdate = {
    type: 'updatePlatform',
    platform: string,
} | {
    type: 'updatePipelineInfo',
    info: PipelineInfo,
} | {
    type: 'updateProfileOverride',
    id: string,
    target: PipelineTarget,
    profileOverride: MaybeString
} | {
    type: 'updateEnabled',
    id: string,
    target: PipelineTarget,
    isEnabled: boolean
} | {
    type: 'updateOneOf',
    id: string,
    target: PipelineTarget,
    selection: string,
} | {
    type: 'updateAction',
    id: string,
    target: PipelineTarget,
    action: Action
} | {
    type: 'updateVisibleOnQAM',
    id: string,
    target: PipelineTarget,
    visible: boolean,
};

export interface PipelineInfo {
    description?: string | undefined;
    name?: string | undefined;
    register_exit_hooks?: boolean | undefined;
    primary_target_override?: PipelineTarget | null | undefined;
}


export async function patchPipeline(pipeline: PipelineDefinition, update: PipelineUpdate): Promise<Result<PipelineDefinition, ApiError>> {
    if (update.type === 'updatePipelineInfo') {
        const info = update.info;

        return Ok({
            ...pipeline,
            name: info.name ?? pipeline.name,
            register_exit_hooks: info.register_exit_hooks ?? pipeline.register_exit_hooks,
            primary_target_override: info.primary_target_override === undefined
                ? pipeline.primary_target_override
                : info.primary_target_override
        });
    } else if (update.type === 'updatePlatform') {
        return Ok({
            ...pipeline,
            platform: update.platform,
        })
    }
    else {
        const u: PipelineActionUpdate = (function () {
            switch (update.type) {
                case 'updateEnabled':
                    return {
                        type: 'UpdateEnabled',
                        value: {
                            is_enabled: update.isEnabled
                        }
                    }
                case 'updateAction':
                    return {
                        type: 'UpdateAction',
                        value: {
                            action: update.action
                        }
                    }
                case 'updateOneOf':
                    return {
                        type: 'UpdateOneOf',
                        value: {
                            selection: update.selection,
                        }
                    }
                case 'updateProfileOverride':
                    console.log("sending update override action to backend:", update.profileOverride);
                    return {
                        type: 'UpdateProfileOverride',
                        value: {
                            profile_override: update.profileOverride
                        }
                    }
                case 'updateVisibleOnQAM':
                    return {
                        type: 'UpdateVisibleOnQAM',
                        value: {
                            is_visible: update.visible
                        }
                    }
            }
        })();

        const res = await patchPipelineAction({
            id: update.id,
            pipeline,
            update: u,
            target: update.target
        });

        return res.map((v) => v.pipeline);
    }
}