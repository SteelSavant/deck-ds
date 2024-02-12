import _ from "lodash";
import { Action, PipelineActionSettings, PipelineDefinition, PipelineTarget } from "../backend";
import { MaybeString } from "../types/short";

export type PipelineUpdate = {
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


export function patchPipeline(pipeline: PipelineDefinition, update: PipelineUpdate): PipelineDefinition {
    if (update.type === 'updatePipelineInfo') {
        const info = update.info;

        return {
            ...pipeline,
            description: info.description ?? pipeline.description,
            name: info.name ?? pipeline.name,
            register_exit_hooks: info.register_exit_hooks ?? pipeline.register_exit_hooks,
            primary_target_override: info.primary_target_override === undefined
                ? pipeline.primary_target_override
                : info.primary_target_override
        };
    } else {
        let updatedActions: { [k: string]: PipelineActionSettings } = {};
        let currentActions = pipeline.actions.actions;
        for (let key in currentActions) {
            if (key === update.id) {
                let cloned = _.cloneDeep(currentActions[key]);

                const type = update.type;

                switch (type) {
                    case 'updateEnabled':
                        cloned.enabled = update.isEnabled;
                        break;
                    case 'updateAction':
                        if (cloned.selection.type !== 'Action') {
                            throw 'Invalid selection type for updateAction';
                        }

                        const id = cloned.selection.value.value.id;

                        cloned.selection = {
                            type: 'Action',
                            value: {
                                ...update.action
                            }
                        };

                        cloned.selection.value.value.id = id;
                        break;
                    case 'updateOneOf':
                        if (cloned.selection.type != 'OneOf') {
                            throw 'Invalid selection type for updateOneOf';
                        }

                        cloned.selection.value.selection = update.selection;
                        break;
                    case 'updateProfileOverride':
                        cloned.profile_override = update.profileOverride
                        break;
                    case 'updateVisibleOnQAM':
                        cloned.is_visible_on_qam = update.visible;
                        break;
                    default:
                        const typecheck: never = type;
                        throw typecheck ?? 'action update failed to typecheck';
                }

                console.log('updated action at', key, 'to', cloned, 'with update', update);

                updatedActions[key] = cloned;
            } else {
                updatedActions[key] = currentActions[key];
            }
        }

        return {
            ...pipeline,
            actions: {
                actions: updatedActions
            }
        }
    }
}