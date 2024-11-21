import { showModal } from '@decky/ui';
import {
    DependencyError,
    PipelineTarget,
    autoStart,
    getProfile,
    reifyPipeline,
} from '../backend';
import ConfigErrorModal from '../components/ConfigErrorModal';
import { ShortAppDetails, useAppState } from '../context/appContext';
import { setupClientPipeline } from '../pipeline/client_pipeline';
import { logger } from '../util/log';

export interface LaunchActions {
    profileId: string;
    targets: LaunchTarget[];
}

type LaunchTarget = {
    target: PipelineTarget;
    action: () => Promise<void>;
};

const useLaunchActions = (
    appDetails: ShortAppDetails | null,
): LaunchActions[] => {
    let { reifiedPipelines } = useAppState();

    console.log('using reified pipelines', reifiedPipelines);

    if (appDetails) {
        return Object.keys(reifiedPipelines)
            .map((pid) => {
                const reified = reifiedPipelines[pid];

                if (!reified || !reified.isOk) {
                    logger.error(
                        `unable to map actions for ${pid}; pipeline not reified:`,
                        reified?.err,
                    );
                    return null;
                }

                const defaultTargets: LaunchTarget[] = [];
                const pipelineTargets: PipelineTarget[] = Object.keys(
                    reified.data.pipeline.targets,
                ) as PipelineTarget[];

                for (const target of pipelineTargets) {
                    const action = async () => {
                        // HACK: QAM does weird caching that means the profile can be outdated,
                        // so we reload the profile in the action to ensure it is current
                        const currentPipeline = await getProfile({
                            profile_id: pid,
                        });

                        const p = currentPipeline?.isOk
                            ? currentPipeline.data.profile
                            : null;

                        if (!p) {
                            logger.toastWarn(
                                'profile with id',
                                pid,
                                'does not exist for action',
                                target,
                            );
                            return;
                        }

                        // Reify pipeline and run autostart procedure for target

                        const reified = await reifyPipeline({
                            pipeline: p.pipeline,
                        });

                        if (reified.isOk) {
                            const configErrors = reified.data.config_errors;
                            const errors: DependencyError[] = [];
                            const otherTag =
                                target === 'Desktop' ? ':gamemode' : ':desktop';
                            for (const key in configErrors) {
                                // only match config errors for this target
                                if (!key.endsWith(otherTag)) {
                                    errors.push(...configErrors[key]);
                                }
                            }

                            if (errors.length > 0) {
                                showModal(<ConfigErrorModal errors={errors} />);
                            } else {
                                // TODO::teardown pipeline on
                                // - plugin startup
                                // - app close (only possible when launch fails, or for game-mode)
                                const res = await (
                                    await setupClientPipeline(
                                        appDetails.appId,
                                        reified.data.pipeline,
                                        target,
                                    )
                                ).andThenAsync(async () =>
                                    (
                                        await autoStart({
                                            user_id_64: appDetails.userId64,
                                            game_id: appDetails.gameId,
                                            app_id: appDetails.appId.toString(),
                                            profile_id: p.id,
                                            game_title: appDetails.sortAs,
                                            is_steam_game:
                                                appDetails.isSteamGame,
                                            target: target,
                                        })
                                    ).mapErr((v) => v.err),
                                );

                                if (!res.isOk) {
                                    logger.toastError(
                                        'Failed to launch app:',
                                        res.err,
                                    );
                                }
                            }
                        } else {
                            if (!reified.isOk) {
                                logger.toastError(
                                    'Failed to load profile to launch app:',
                                    reified.err.err,
                                );
                            }
                        }
                    };

                    const value = {
                        action,
                        target: target as PipelineTarget,
                    };

                    if (target === 'Gamemode') {
                        defaultTargets.push(value);
                    } else if (target === 'Desktop') {
                        defaultTargets.splice(0, 0, value);
                    } else {
                        // extra targets not planned or handled
                        logger.warn(
                            'unsupported target in launch action:',
                            target,
                        );
                    }
                }

                const res = {
                    profileId: pid,
                    targets: defaultTargets,
                };

                console.log('returning mapped actions:', res);

                return res;
            })
            .filter((v) => v)
            .map((v) => v!);
    } else {
        logger.warn('not building actions; no app details');
    }

    return [];
};

export default useLaunchActions;
