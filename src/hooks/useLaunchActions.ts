import { CategoryProfile, PipelineTarget, autoStart, reifyPipeline } from "../backend";
import { ShortAppDetails } from "../context/shortAppDetailsContext";
import useProfiles from "./useProfiles";


interface LaunchActions {
    profile: CategoryProfile,
    targets: LaunchTarget[]
};

type LaunchTarget = {
    target: PipelineTarget,
    action: () => Promise<void>
}

const useLaunchActions = (appDetails: ShortAppDetails): LaunchActions[] => {
    let { profiles } = useProfiles();

    console.log('getting launch actions for', appDetails);

    if (profiles?.isOk) {
        const loadedProfiles = profiles.data;
        const includedProfiles = new Set<string>();
        const validProfiles = collectionStore.userCollections.flatMap((uc) => {
            const containsApp = uc.apps.get(appDetails.appId);

            console.log(uc.displayName, 'contains app', appDetails.appId, ":", containsApp);

            if (containsApp) {
                const matchedProfiles = loadedProfiles
                    .filter((p) => !includedProfiles.has(p.id))
                    .filter((p) => p.tags.includes(uc.id));

                for (const p of matchedProfiles) {
                    console.log(uc.displayName, 'adding profile');

                    includedProfiles.add(p.id);
                }
                return matchedProfiles;
            } else {
                return []
            }
        });

        console.log('found', validProfiles.length, 'valid profiles for launch');

        return validProfiles.map((p) => {
            const targets = p.pipeline.targets

            const defaultTargets: LaunchTarget[] = []

            for (const key in targets) {
                const action = async () => {
                    const reified = (await reifyPipeline({
                        pipeline: p.pipeline
                    }));


                    if (reified.isOk) {
                        const res = await autoStart({
                            app: appDetails.gameId,
                            pipeline: reified.data.pipeline,
                            target: key as PipelineTarget
                        });

                        if (!res.isOk) {
                            // TODO::handle error
                        }
                    } else {
                        // TODO::handle error
                    }
                };

                const value = {
                    action,
                    target: key as PipelineTarget
                }

                if (key === 'Gamemode') {
                    defaultTargets.push(value);
                } else if (key === 'Desktop') {
                    defaultTargets.splice(0, 0, value);
                } else {
                    // extra targets not planned or handled
                }
            }

            return {
                profile: p,
                targets: defaultTargets
            };
        });
    }

    return [];
}

export default useLaunchActions;