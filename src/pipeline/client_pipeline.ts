import { sleep } from 'decky-frontend-lib';
import { uniqueId } from 'lodash';
import {
    Action,
    Pipeline,
    PipelineTarget,
    RuntimeSelection,
    addClientTeardownAction,
    getClientTeardownActions,
    getDisplayInfo,
    log as logBackend,
    removeClientTeardownActions,
} from '../backend';
import { ClientTeardownAction } from '../types/backend_api';
import { LogLevel, logger } from '../util/log';
import { Err, Ok, Result } from '../util/result';

const gamescopeLaunchOptionsRxp = /$gamescope(.*)\s--\s(.*)/g;

export async function setupClientPipeline(
    appId: number,
    pipeline: Pipeline,
    target: PipelineTarget,
): Promise<Result<null, string>> {
    const selection = pipeline.targets[target];

    function flattenSelection(s: RuntimeSelection): Action[] {
        const type = s.type;
        switch (type) {
            case 'Action':
                return [s.value];
            case 'OneOf':
                const found = s.value.actions.find(
                    (a) => a.id === s.value.selection && a.enabled !== false,
                );
                return found ? flattenSelection(found.selection) : [];

            case 'AllOf':
                return s.value
                    .filter((v) => v.enabled !== false)
                    .flatMap((v) => flattenSelection(v.selection));
            default:
                const typecheck: never = type;
                throw `Failed to typecheck runtime selection: ${typecheck}`;
        }
    }

    const actions = flattenSelection(selection);
    await logBackend(
        LogLevel.Debug,
        `setting up client pipeline with ${actions.length} actions`,
    );

    const promises: Promise<any>[] = [];

    for (const action of actions) {
        const res = await execAction(appId, action);

        if (res.isOk) {
            if (res.data !== null) {
                promises.push(
                    addClientTeardownAction({
                        action: res.data,
                    }),
                );
            }
        } else {
            return res.map((_) => {
                return null;
            });
        }
    }

    // sleep to ensure all steam client calls have time to take effect
    await sleep(500);
    await Promise.all(promises);

    return Ok(null);
}

async function execAction(
    appId: number,
    action: Action,
): Promise<Result<ClientTeardownAction | null, string>> {
    // TODO::maybe return teardown action instead of null
    const type = action.type;
    switch (type) {
        case 'MainAppAutomaticWindowing':
            const gamescope = action.value.gamescope;
            const general = action.value.general;
            if (gamescope.use_gamescope) {
                await logBackend(
                    LogLevel.Debug,
                    'configuring gamescope launch actions',
                );

                const dislayRes = await getPrimaryDisplayResolution(
                    general.swap_screens,
                );
                if (!dislayRes.isOk) {
                    return Err(dislayRes.err);
                }

                const previousLaunchOptions =
                    appDetailsStore.GetAppDetails(appId).strLaunchOptions;

                // strip existing gamescope prefix to ensure no duplicates
                const strippedLaunchOptions = gamescopeLaunchOptionsRxp.exec(
                    previousLaunchOptions,
                );

                const command = strippedLaunchOptions
                    ? strippedLaunchOptions[2]
                    : '%command%';

                const gamescopeOptions = [
                    `gamescope -e -W ${dislayRes.data.width | 0} -H ${
                        dislayRes.data.height | 0
                    } -F ${gamescope.filter.toLowerCase()} -S ${gamescope.scaler.toLowerCase()}`,
                ];

                if (gamescope.filter === 'FSR') {
                    gamescopeOptions.push(
                        `--sharpness ${gamescope.fsr_sharpness}`,
                    );
                } else if (gamescope.filter === 'NIS') {
                    gamescopeOptions.push(
                        `--sharpness ${gamescope.nis_sharpness}`,
                    );
                }

                switch (gamescope.fullscreen_option) {
                    case 'Borderless':
                        gamescopeOptions.push('-b');
                    case 'Fullscreen':
                        gamescopeOptions.push('-f');
                }

                if (gamescope.game_resolution) {
                    gamescopeOptions.push(
                        `-w ${gamescope.game_resolution.w | 0} -h ${
                            gamescope.game_resolution.h | 0
                        }`,
                    );
                }

                if (gamescope.game_refresh) {
                    gamescopeOptions.push(`-r ${gamescope.game_refresh | 0}`);
                }

                gamescopeOptions.push('--', command);

                const newLaunchOptions = gamescopeOptions.join(' ');
                logBackend(
                    LogLevel.Debug,
                    `configuring gamescope for ${appId} with launch options ${newLaunchOptions}`,
                );

                SteamClient.Apps.SetAppLaunchOptions(appId, newLaunchOptions);

                return Ok({
                    type: 'MainAppAutomaticWindowing',
                    app_id: appId,
                    action_id: uniqueId(),
                    previous_launch_options: previousLaunchOptions,
                });
            }
        default:
            return Ok(null);
    }
}

async function getPrimaryDisplayResolution(
    useInternal: boolean,
): Promise<Result<{ width: number; height: number }, string>> {
    if (useInternal) {
        return Ok({ width: 1280, height: 800 }); // TODO::don't hardcode this
    }

    const res = await getDisplayInfo();
    return res
        .map((v) =>
            v.available_values.reduce((acc, c) =>
                acc.height * acc.width > c.height * c.width ? acc : c,
            ),
        )
        .mapErr((err) => err.err);
}

/// Tears down client pipeline. If [appId] is undefined, will tear down all existing actions,
/// otherwise will only tear down actions for the specified [appId].
export async function teardownClientPipeline(appId?: number): Promise<void> {
    const res = await getClientTeardownActions();

    if (res.isOk) {
        const actions = res.data.actions.filter((v) =>
            appId ? v.app_id === appId : true,
        );
        const resolved: string[] = [];

        while (actions.length > 0) {
            const action = actions.pop()!;
            try {
                const res = await teardownClientAction(action);
                if (!res.isOk) {
                    logger.warn(
                        'failed to teardown client action',
                        action,
                        ':',
                        res.err,
                    );
                } else {
                    resolved.push(action.action_id);
                }
            } catch (ex) {
                logger.error(
                    'failed to teardown client action',
                    action,
                    ':',
                    ex,
                );
            }
        }

        await removeClientTeardownActions({
            ids: resolved,
        });
    }
}

async function teardownClientAction(
    action: ClientTeardownAction,
): Promise<Result<null, string>> {
    const type = action.type;
    switch (type) {
        case 'MainAppAutomaticWindowing':
            SteamClient.Apps.SetAppLaunchOptions(
                action.app_id,
                action.previous_launch_options,
            );
            return Ok(null);
        default:
            const typecheck: never = type;
            throw `ClientTeardownAction failed to typecheck: ${typecheck}`;
    }
}
