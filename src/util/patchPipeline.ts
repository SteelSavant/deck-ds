import { v4 } from 'uuid';
import {
    Action,
    ApiError,
    PipelineDefinition,
    PipelineTarget,
    patchPipelineAction,
} from '../backend';
import {
    DesktopControllerLayoutHack,
    PipelineActionUpdate,
} from '../types/backend_api';
import { MaybeString } from '../types/short';
import { Ok, Result } from './result';

export type PipelineUpdate =
    | {
          type: 'updatePlatform';
          platform: string;
      }
    | {
          type: 'addTopLevel';
          action_id: string;
      }
    | {
          type: 'removeTopLevel';
          id: string;
      }
    | {
          type: 'updatePipelineInfo';
          info: PipelineInfo;
      }
    | {
          type: 'updateProfileOverride';
          action_id: string;
          toplevel_id: string;
          target: PipelineTarget;
          profileOverride: MaybeString;
      }
    | {
          type: 'updateEnabled';
          action_id: string;
          toplevel_id: string;
          target: PipelineTarget;
          isEnabled: boolean;
      }
    | {
          type: 'updateOneOf';
          action_id: string;
          toplevel_id: string;
          target: PipelineTarget;
          selection: string;
      }
    | {
          type: 'updateAction';
          action_id: string;
          toplevel_id: string;
          target: PipelineTarget;
          action: Action;
      }
    | {
          type: 'updateVisibleOnQAM';
          action_id: string;
          toplevel_id: string;
          target: PipelineTarget;
          visible: boolean;
      };

export interface PipelineInfo {
    description?: string;
    name?: string;
    // exit_hooks_override?: BtnChord | null;
    // next_window_hooks_override?: BtnChord | null;
    // register_exit_hooks?: boolean;
    primary_target_override?: PipelineTarget | null;
    steam_desktop_layout_config_hack_override?: boolean | null;
    nonsteam_desktop_layout_config_hack_override?: boolean | null;
}

export async function patchPipeline(
    pipeline: PipelineDefinition,
    update: PipelineUpdate,
): Promise<Result<PipelineDefinition, ApiError>> {
    if (update.type === 'updatePipelineInfo') {
        const info = update.info;

        const hack: DesktopControllerLayoutHack = {
            ...pipeline.desktop_controller_layout_hack,
            steam_override:
                update.info.steam_desktop_layout_config_hack_override ===
                undefined
                    ? pipeline.desktop_controller_layout_hack.steam_override
                    : info.steam_desktop_layout_config_hack_override,
            nonsteam_override:
                update.info.nonsteam_desktop_layout_config_hack_override ===
                undefined
                    ? pipeline.desktop_controller_layout_hack.nonsteam_override
                    : info.nonsteam_desktop_layout_config_hack_override,
        };

        return Ok({
            ...pipeline,
            name: info.name ?? pipeline.name,
            // register_exit_hooks:
            //     info.register_exit_hooks ?? pipeline.should_register_exit_hooks,
            // exit_hooks_override:
            //     info.exit_hooks_override === undefined
            //         ? pipeline.exit_hooks_override
            //         : info.exit_hooks_override,
            primary_target_override:
                info.primary_target_override === undefined
                    ? pipeline.primary_target_override
                    : info.primary_target_override,
            desktop_controller_layout_hack: hack,
        });
    } else if (update.type === 'updatePlatform') {
        return Ok({
            ...pipeline,
            platform: {
                ...pipeline.platform,
                root: update.platform,
            },
        });
    } else if (update.type === 'addTopLevel') {
        return Ok({
            ...pipeline,
            toplevel: pipeline.toplevel.concat([
                {
                    id: v4(),
                    root: update.action_id,
                    actions: { actions: {} },
                },
            ]),
        });
    } else if (update.type === 'removeTopLevel') {
        return Ok({
            ...pipeline,
            toplevel: pipeline.toplevel.filter((v) => v.id != update.id),
        });
    } else {
        const u: PipelineActionUpdate = (function () {
            const type = update.type;
            switch (type) {
                case 'updateEnabled':
                    return {
                        type: 'UpdateEnabled',
                        value: {
                            is_enabled: update.isEnabled,
                        },
                    };
                case 'updateAction':
                    return {
                        type: 'UpdateAction',
                        value: {
                            action: update.action,
                        },
                    };
                case 'updateOneOf':
                    return {
                        type: 'UpdateOneOf',
                        value: {
                            selection: update.selection,
                        },
                    };
                case 'updateProfileOverride':
                    return {
                        type: 'UpdateProfileOverride',
                        value: {
                            profile_override: update.profileOverride,
                        },
                    };
                case 'updateVisibleOnQAM':
                    return {
                        type: 'UpdateVisibleOnQAM',
                        value: {
                            is_visible: update.visible,
                        },
                    };
                default:
                    const typecheck: never = type;
                    throw `failed to typecheck PipelineActionUpdate: ${typecheck}`;
            }
        })();

        const res = await patchPipelineAction({
            action_id: update.action_id,
            toplevel_id: update.toplevel_id,
            pipeline,
            update: u,
            target: update.target,
        });

        return res.map((v) => v.pipeline);
    }
}
