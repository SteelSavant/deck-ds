import { Action, ApiError, PipelineDefinition, PipelineTarget, patchPipelineAction } from "../backend";
import { ExitHooks, PipelineActionUpdate } from "../types/backend_api";
import { MaybeString } from "../types/short";
import { Ok, Result } from "./result";

export type PipelineUpdate = {
    type: 'updatePlatform',
    platform: string,
} | {
    type: 'addTopLevel',
    action_id: string,
} | {
    type: 'removeTopLevel',
    id: string,
} | {
    type: 'updatePipelineInfo',
    info: PipelineInfo,
} | {
    type: 'updateProfileOverride',
    action_id: string,
    toplevel_id: string,
    target: PipelineTarget,
    profileOverride: MaybeString
} | {
    type: 'updateEnabled',
    action_id: string,
    toplevel_id: string,
    target: PipelineTarget,
    isEnabled: boolean
} | {
    type: 'updateOneOf',
    action_id: string,
    toplevel_id: string,
    target: PipelineTarget,
    selection: string,
} | {
    type: 'updateAction',
    action_id: string,
    toplevel_id: string,
    target: PipelineTarget,
    action: Action
} | {
    type: 'updateVisibleOnQAM',
    action_id: string,
    toplevel_id: string,
    target: PipelineTarget,
    visible: boolean,
};

export interface PipelineInfo {
    description?: string | undefined,
    name?: string | undefined,
    exit_hooks_override?: ExitHooks | undefined,
    register_exit_hooks?: boolean | undefined,
    primary_target_override?: PipelineTarget | null | undefined,
}


export async function patchPipeline(pipeline: PipelineDefinition, update: PipelineUpdate): Promise<Result<PipelineDefinition, ApiError>> {

    if (update.type === 'updatePipelineInfo') {
        const info = update.info;

        return Ok({
            ...pipeline,
            name: info.name ?? pipeline.name,
            register_exit_hooks: info.register_exit_hooks ?? pipeline.should_register_exit_hooks,
            exit_hooks_override: info.exit_hooks_override ?? pipeline.exit_hooks_override,
            primary_target_override: info.primary_target_override === undefined
                ? pipeline.primary_target_override
                : info.primary_target_override
        });
    } else if (update.type === 'updatePlatform') {
        return Ok({
            ...pipeline,
            platform: {
                ...pipeline.platform,
                root: update.platform
            }
        })
    } else if (update.type === 'addTopLevel') {
        return Ok({
            ...pipeline,
            toplevel: pipeline.toplevel.concat([{
                id: '00000000-0000-0000-0000-000000000000',
                root: update.action_id,
                actions: { actions: {} }
            }])
        })
    } else if (update.type === 'removeTopLevel') {
        return Ok({
            ...pipeline,
            toplevel: pipeline.toplevel.filter((v) => v.id != update.id)
        });
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
            action_id: update.action_id,
            toplevel_id: update.toplevel_id,
            pipeline,
            update: u,
            target: update.target
        });

        return res.map((v) => v.pipeline);
    }
}