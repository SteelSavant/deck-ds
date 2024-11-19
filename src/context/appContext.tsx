// Adapted from https://github.com/OMGDuke/SDH-GameThemeMusic/blob/main/src/state/ShortAppDetailsState.tsx

import { createContext, FC, useContext, useEffect, useState } from 'react';
import {
    ApiError,
    AppProfile,
    getAppProfile,
    getDefaultAppOverrideForProfileRequest,
    getProfile,
    getProfiles,
    getSettings,
    PipelineDefinition,
    PipelineTarget,
    reifyPipeline,
    ReifyPipelineResponse,
    RuntimeSelection,
    setAppProfileOverride,
    setAppProfileSettings,
    setProfile,
} from '../backend';
import { PipelineActionLookup, TopLevelDefinition } from '../types/backend_api';
import { MaybeString } from '../types/short';
import { Loading } from '../util/loading';
import { logger } from '../util/log';
import { patchPipeline, PipelineUpdate } from '../util/patchPipeline';
import { Result } from '../util/result';

// When setting the app
// - Load all profiles
// - Find matching profiles for app
// - Load all overrides (including defaults)
// - Expose launch actions for profiles
// - Subscribe to category changes, to allow dynamically updating the overrides

export interface StateAction {
    externalProfile: MaybeString;
    update: PipelineUpdate;
}

export interface ShortAppDetails {
    appId: number;
    gameId: string;
    userId64: string;
    sortAs: string;
    isSteamGame: boolean;
    selected_clientid: string;
}

export interface AppTargetSelection {
    primaryTarget: PipelineTarget | null;
    secondaryTarget: PipelineTarget | null;
}

interface PublicAppState {
    appDetails: ShortAppDetails | null;
    appProfile: Loading<AppProfile>;
    reifiedPipelines: { [k: string]: Result<ReifyPipelineResponse, ApiError> };
    openViews: { [k: string]: { [k: string]: boolean } };
}

// The localThemeEntry interface refers to the theme data as given by the python function, the Theme class refers to a theme after it has been formatted and the generate function has been added

interface PublicAppStateContext extends PublicAppState {
    setOnAppPage(appDetails: ShortAppDetails): void;
    setAppProfileDefault(
        appDetails: ShortAppDetails,
        defaultProfileId: string | null,
    ): Promise<void>;
    setAppViewOpen(
        profileId: string,
        view: PipelineTarget,
        isOpen: boolean,
    ): void;
    dispatchUpdate(profileId: string, action: StateAction): Promise<void>;
    ensureSelectedClientUpdated(): Promise<void>;
    useAppTarget(props: {
        profileId: string | null;
        isPrimary: boolean;
    }): PipelineTarget | null;
}

// This class creates the getter and setter functions for all of the global state data.
export class ShortAppDetailsState {
    private readonly delayMs = 1000;
    private appDetails: ShortAppDetails | null = null;
    private appProfile: Loading<AppProfile>;
    private reifiedPipelines: {
        [k: string]: Result<ReifyPipelineResponse, ApiError>;
    } = {};
    private openViews: { [k: string]: { [k: string]: boolean } } = {};
    private lastOnAppPageTime: number = 0;
    private appTargets: { [profileId: string]: AppTargetSelection } = {};

    // You can listen to this eventBus' 'stateUpdate' event and use that to trigger a useState or other function that causes a re-render
    public readonly eventBus = new EventTarget();

    getPublicState(): PublicAppState {
        return {
            appDetails: this.appDetails ? { ...this.appDetails } : null,
            appProfile: this.appProfile ? { ...this.appProfile } : null,
            reifiedPipelines: { ...this.reifiedPipelines },
            openViews: { ...this.openViews },
        };
    }

    getAppTarget(profileId: string, isPrimary: boolean) {
        const target = this.appTargets[profileId];
        return isPrimary ? target?.primaryTarget : target?.secondaryTarget;
    }

    setOnAppPage(appDetails: ShortAppDetails | null) {
        const time = Date.now();

        logger.trace('trying to set app to', appDetails?.sortAs, '@', time);

        setTimeout(
            () => {
                this.setOnAppPageInternal(appDetails, time);
            },
            appDetails ? 0 : this.delayMs,
        );
    }

    setAppViewOpen(profileId: string, view: PipelineTarget, isOpen: boolean) {
        this.openViews[profileId] ??= {};
        this.openViews[profileId][view] = isOpen;
        this.forceUpdate();
    }

    async dispatchUpdate(profileId: string, action: StateAction) {
        const appId = this.appDetails?.appId;

        if (appId === null || appId === undefined) {
            return;
        }

        // defer updates to external profiles, to avoid complexity of local state
        if (action.externalProfile) {
            logger.debug('is external profile update');
            await this.updateExternalProfile(
                action.externalProfile,
                action.update,
            );
        } else {
            if (this.appProfile?.isOk) {
                logger.debug(
                    profileId,
                    'is pipeline update; current state:',
                    this.appProfile,
                );

                const pipeline = this.appProfile.data.overrides[profileId];
                if (pipeline) {
                    let res = (
                        await patchPipeline(pipeline, action.update)
                    ).map(async (newPipeline) => {
                        return await this.setAppProfileOverrideInternal(
                            appId,
                            profileId,
                            newPipeline,
                        );
                    });

                    if (res.isOk) {
                        await res.data;
                    } else {
                        logger.toastWarn('error updating profile', res.err);
                    }
                } else {
                    logger.toastError(
                        'pipeline should already be loaded before updating',
                        pipeline,
                    );
                }
            }
        }

        await this.refetchProfile(profileId, appId);
    }

    async setAppProfileDefault(
        appDetails: ShortAppDetails,
        defaultProfileId: string,
    ) {
        const res = await setAppProfileSettings({
            app_id: appDetails.appId.toString(),
            default_profile: defaultProfileId,
        });

        if (res?.isOk) {
            this.refetchProfile(defaultProfileId, appDetails.appId);
        } else {
            logger.toastError(
                'failed to set app(',
                appDetails.appId,
                ') default to',
                defaultProfileId,
            );
        }
    }

    private async loadProfileOverride(appId: number, profileId: string) {
        logger.debug('loading app profile', appId, ':', profileId);
        let shouldUpdate = false;
        if (this.appDetails?.appId === appId && this.appProfile?.isOk) {
            const overrides = this.appProfile.data.overrides;
            if (!overrides[profileId]) {
                const res = await getDefaultAppOverrideForProfileRequest({
                    profile_id: profileId,
                });
                if (res.isOk && res.data.pipeline) {
                    overrides[profileId] = res.data.pipeline;
                    logger.debug(
                        'set override for',
                        profileId,
                        'to',
                        overrides[profileId],
                    );
                    shouldUpdate = true;
                } else if (!res.isOk) {
                    logger.toastWarn(
                        'Failed to initialize app profile:',
                        res.err.err,
                    );
                }
            } else {
                logger.debug('existing override found:', overrides[profileId]);
            }

            if (overrides[profileId]) {
                const res = (
                    await reifyPipeline({ pipeline: overrides[profileId] })
                ).map((res) =>
                    patchProfileOverridesForMissing(
                        profileId,
                        overrides[profileId],
                        res,
                    ),
                );
                this.reifiedPipelines[profileId] = res;
                this.appTargets[profileId] = await computeAppTargetSelection(
                    overrides[profileId],
                );
                logger.debug(
                    'load reified to:',
                    this.reifiedPipelines[profileId],
                );
                shouldUpdate = true;
            }
        }

        if (shouldUpdate) {
            this.forceUpdate();
        }
    }

    ensureSelectedClientUpdated() {
        if (this.appDetails) {
            const id = appStore.GetAppOverviewByAppID(
                this.appDetails.appId,
            )?.selected_clientid;

            if (id !== this.appDetails.selected_clientid) {
                this.appDetails.selected_clientid = id;
                this.forceUpdate();
            }
        }
    }

    private async setAppProfileOverrideInternal(
        appId: number,
        profileId: string,
        pipeline: PipelineDefinition,
    ) {
        const res = await setAppProfileOverride({
            app_id: appId.toString(),
            profile_id: profileId,
            pipeline,
        });

        if (res?.isOk && this.appDetails?.appId === appId) {
            this.appProfile = this.appProfile?.map((p) => {
                const overrides = {
                    ...p.overrides,
                };
                overrides[profileId] = pipeline;

                return {
                    ...p,
                    overrides,
                };
            });
            this.refetchProfile(profileId, appId);
        } else {
            logger.toastWarn(
                'failed to set app(',
                appId,
                ') override for',
                profileId,
            );
        }
    }

    private async updateExternalProfile(
        profileId: string,
        update: PipelineUpdate,
    ) {
        const profileResponse = await getProfile({
            profile_id: profileId,
        });

        if (profileResponse?.isOk) {
            const profile = profileResponse.data.profile;
            const pipeline = profile?.pipeline;
            if (pipeline) {
                const res = await (
                    await patchPipeline(pipeline, update)
                ).andThenAsync(async (res) => {
                    return await setProfile({
                        profile: {
                            ...profile,
                            pipeline: res,
                        },
                    });
                });

                if (res?.isOk) {
                    this.refetchProfile(profileId);
                } else {
                    logger.warn('failed to set external profile', profileId);
                }
            } else {
                logger.toastWarn('external profile', profileId, 'not found');
            }
        } else {
            logger.toastWarn('failed to fetch external profile', profileId);
        }
    }

    private async refetchProfile(
        externalProfileId: string,
        appIdToMatch?: number,
    ) {
        const internal = async () => {
            if (
                this.appDetails &&
                (!appIdToMatch || this.appDetails?.appId == appIdToMatch)
            ) {
                const newProfile = (
                    await getAppProfile({
                        app_id: this.appDetails.appId.toString(),
                    })
                ).map((a) => a.app ?? null);

                if (this.appProfile?.isOk && newProfile.isOk) {
                    for (const key in this.appProfile.data.overrides) {
                        newProfile.data.overrides[key] ??=
                            this.appProfile.data.overrides[key];
                    }
                }

                if (!this.appProfile?.isOk) {
                    logger.toastWarn(
                        'failed to refetch app(',
                        appIdToMatch,
                        ')',
                        this.appProfile?.err,
                    );
                } else {
                    const overrides = this.appProfile.data.overrides;
                    for (const k in overrides) {
                        let reified = (
                            await reifyPipeline({
                                pipeline: overrides[k],
                            })
                        ).map((response) =>
                            patchProfileOverridesForMissing(
                                externalProfileId,
                                overrides[k],
                                response,
                            ),
                        );

                        this.reifiedPipelines[k] = reified;
                        this.appTargets[k] = await computeAppTargetSelection(
                            overrides[k],
                        );
                    }

                    logger.debug(
                        'refetched; updating to',
                        this.appProfile.data?.overrides,
                    );
                }

                this.forceUpdate();
            }
        };
        await internal();
    }

    private setOnAppPageInternal(
        appDetails: ShortAppDetails | null,
        time: number,
    ) {
        const areEqual =
            (appDetails === null && this.appDetails === null) ||
            (appDetails?.appId === this.appDetails?.appId &&
                appDetails?.gameId === this.appDetails?.gameId);

        if (time <= this.lastOnAppPageTime || areEqual) {
            this.ensureSelectedClientUpdated();
            if (areEqual) {
                this.lastOnAppPageTime = time;
            }
            return;
        }

        logger.debug('setting app to ', appDetails?.sortAs, '@', Date.now());

        this.appDetails = appDetails;
        this.appProfile = null;
        this.openViews = {};
        this.reifiedPipelines = {};
        this.lastOnAppPageTime = time;
        this.forceUpdate();
        this.fetchProfile(appDetails);
    }

    private async fetchProfile(appDetails: ShortAppDetails | null) {
        console.log('starting profile fetch for', appDetails);
        if (appDetails) {
            console.log('fetching profile for', appDetails);

            const profile = (
                await getAppProfile({ app_id: appDetails.appId.toString() })
            ).map((v) => v.app ?? null);
            if (this.appDetails?.appId == appDetails.appId) {
                console.log('fetched profile for', appDetails, ':', profile);

                this.appProfile = profile;

                if (profile.isOk) {
                    console.log(
                        'handling profile for',
                        appDetails,
                        ':',
                        profile.data,
                    );

                    const profileIds = await getProfileIdsForAppId(
                        appDetails.appId,
                    );

                    const profiles = profileIds.map((profileId) =>
                        this.loadProfileOverride(appDetails.appId, profileId),
                    );
                    await Promise.all(profiles);
                }
                this.forceUpdate();
            }
        }
    }

    private forceUpdate(updateSelectedClient = true) {
        if (this.appDetails && updateSelectedClient) {
            this.appDetails.selected_clientid = appStore.GetAppOverviewByAppID(
                this.appDetails.appId,
            )?.selected_clientid;
        }

        this.eventBus.dispatchEvent(new Event('stateUpdate'));
    }
}

const AppContext = createContext<PublicAppStateContext>(null as any);
export const useAppState = () => useContext(AppContext);

export interface ShortAppDetailsStateProviderProps {
    ShortAppDetailsStateClass: ShortAppDetailsState;
}

// This is a React Component that you can wrap multiple separate things in, as long as they both have used the same instance of the CssLoaderState class, they will have synced state
export const ShortAppDetailsStateContextProvider: FC<
    ShortAppDetailsStateProviderProps
> = ({ children, ShortAppDetailsStateClass }) => {
    const [publicState, setPublicState] = useState<PublicAppState>({
        ...ShortAppDetailsStateClass.getPublicState(),
    });

    // TODO::get global settings from hook, instead of function call when needed

    useEffect(() => {
        function onUpdate() {
            setPublicState({ ...ShortAppDetailsStateClass.getPublicState() });
        }

        ShortAppDetailsStateClass.eventBus.addEventListener(
            'stateUpdate',
            onUpdate,
        );

        return () =>
            ShortAppDetailsStateClass.eventBus.removeEventListener(
                'stateUpdate',
                onUpdate,
            );
    }, []);

    const setOnAppPage = (appDetails: ShortAppDetails) =>
        ShortAppDetailsStateClass.setOnAppPage(appDetails);

    const setAppProfileDefault = async (
        appDetails: ShortAppDetails,
        defaultProfileId: string,
    ) => {
        ShortAppDetailsStateClass.setAppProfileDefault(
            appDetails,
            defaultProfileId,
        );
    };
    const setAppViewOpen = (
        profileId: string,
        view: PipelineTarget,
        isOpen: boolean,
    ) => {
        ShortAppDetailsStateClass.setAppViewOpen(profileId, view, isOpen);
    };

    const dispatchUpdate = async (profileId: string, action: StateAction) => {
        ShortAppDetailsStateClass.dispatchUpdate(profileId, action);
    };

    const ensureSelectedClientUpdated = async () => {
        ShortAppDetailsStateClass.ensureSelectedClientUpdated();
    };

    const getAppTarget = (props: { profileId: string; isPrimary: boolean }) =>
        ShortAppDetailsStateClass.getAppTarget(
            props.profileId,
            props.isPrimary,
        );
    return (
        <AppContext.Provider
            value={{
                ...publicState,
                setOnAppPage,
                setAppProfileDefault,
                setAppViewOpen,
                dispatchUpdate,
                ensureSelectedClientUpdated,
                useAppTarget: getAppTarget,
            }}
        >
            {children}
        </AppContext.Provider>
    );
};

function patchProfileOverridesForMissing(
    externalProfileId: string,
    overrides: PipelineDefinition,
    response: ReifyPipelineResponse,
): ReifyPipelineResponse {
    const pipeline = response.pipeline;

    const toplevel: { [k: string]: TopLevelDefinition } = {};
    toplevel[overrides.platform.id] = overrides.platform;
    for (const tl of overrides.toplevel) {
        toplevel[tl.id] = tl;
    }

    function patch(selection: RuntimeSelection, actions: PipelineActionLookup) {
        const type = selection.type;
        switch (type) {
            case 'Action':
                return;
            case 'OneOf':
                for (const v of selection.value.actions) {
                    if (!actions.actions[v.id]) {
                        v.profile_override = externalProfileId;
                    }
                    patch(v.selection, actions);
                }
                return;
            case 'AllOf':
                for (const v of selection.value) {
                    if (!actions.actions[v.id]) {
                        v.profile_override = externalProfileId;
                    }
                    patch(v.selection, actions);
                }
                return;

            default:
                const typecheck: never = type;
                throw `runtime selection failed to typecheck: ${typecheck}`;
        }
    }

    for (const target in pipeline.targets) {
        let selection = pipeline.targets[target];
        if (selection.type === 'AllOf') {
            const actions = selection.value;
            for (const a of actions) {
                const toplevel_actions = toplevel[a.toplevel_id].actions;
                patch(a.selection, toplevel_actions);
                if (!toplevel_actions.actions[a.id]) {
                    a.profile_override = externalProfileId;
                }
            }
        } else {
            throw 'expected toplevel action to be AllOf';
        }
    }

    logger.debug('reify response after patch: ', response.pipeline);

    return response;
}

async function getProfileIdsForAppId(appId: number): Promise<string[]> {
    const loadedProfiles = (await getProfiles()).unwrap().profiles;
    const includedProfiles = new Set<string>();
    const profiles = collectionStore.userCollections
        .flatMap((uc) => {
            const containsApp = uc.apps.has(appId);

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
        })
        .map((v) => v.id);
    logger.debug('using profiles', profiles, 'for appid', appId);

    return profiles;
}

async function computeAppTargetSelection(
    pipeline: PipelineDefinition,
): Promise<AppTargetSelection> {
    const settings = await getSettings();
    const defaultTarget = settings.isOk
        ? settings.data.global_settings.primary_ui_target
        : 'Gamemode';
    const primaryTarget = pipeline.primary_target_override ?? defaultTarget;

    return {
        primaryTarget,
        secondaryTarget: primaryTarget === 'Gamemode' ? 'Desktop' : 'Gamemode',
    };
}
