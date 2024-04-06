import { Action, ApiError, PipelineDefinition, PipelineTarget, patchPipelineAction } from "../backend";
import { PipelineActionUpdate } from "../types/backend_api";
import { MaybeString } from "../types/short";
import { Ok, Result } from "./result";

export type PipelineUpdate = {
    type: 'updatePlatform',
    platform: string,
} | {
    type: 'updateEnabled',
    id: string,
    isEnabled: boolean
} | {
    type: 'updateProfileOverride',
    id: string,
    profileOverride: MaybeString
} | {
    type: 'updateOneOf',
    id: string,
    selection: string,
} | {
    type: 'updateAction',
    id: string,
    action: Action
} | {
    type: 'updatePipelineInfo',
    info: PipelineInfo,
} | {
    type: 'updateVisibleOnQAM',
    id: string,
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
        });

        return res.map((v) => v.pipeline);
    }
}