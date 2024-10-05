import { showModal } from '@decky/ui';
import {
    CategoryProfile,
    DependencyError,
    PipelineTarget,
    autoStart,
    getProfile,
    reifyPipeline,
} from '../backend';
import ConfigErrorModal from '../components/ConfigErrorModal';
import { ShortAppDetails } from '../context/appContext';
import { setupClientPipeline } from '../pipeline/client_pipeline';
import { logger } from '../util/log';
import useProfiles from './useProfiles';

export interface LaunchActions {
    profile: CategoryProfile;
    targets: LaunchTarget[];
}

type LaunchTarget = {
    target: PipelineTarget;
    action: () => Promise<void>;
};

const useLaunchActions = (
    appDetails: ShortAppDetails | null,
): LaunchActions[] => {
    let { profiles } = useProfiles();

    if (appDetails && profiles?.isOk) {
        const loadedProfiles = profiles.data;
        const includedProfiles = new Set<string>();
        const validProfiles = collectionStore.userCollections.flatMap((uc) => {
            const containsApp = uc.apps.has(appDetails.appId);

            if (containsApp) {
                const matchedProfiles = loadedProfiles
                    .filter((p) => !includedProfiles.has(p.id))
                    .filter((p) => p.tags.includes(uc.id));

                for (const p of matchedProfiles) {
                    includedProfiles.add(p.id);
                }
                return matchedProfiles;
            } else {
                return [];
            }
        });

        return validProfiles.map((p) => {
            const defaultTargets: LaunchTarget[] = [];
            const pipelineTargets: PipelineTarget[] = ['Desktop', 'Gamemode'];

            for (const target of pipelineTargets) {
                const action = async () => {
                    // HACK: QAM does weird caching that means the profile can be outdated,
                    // so we reload the profile in the action to ensure it is current
                    const currentPipeline = await getProfile({
                        profile_id: p.id,
                    });

                    p =
                        (currentPipeline?.isOk
                            ? currentPipeline.data.profile
                            : null) ?? p;

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
                                        is_steam_game: appDetails.isSteamGame,
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
                }
            }

            const res = {
                profile: p,
                targets: defaultTargets,
            };

            return res;
        });
    }

    return [];
};

export default useLaunchActions;
