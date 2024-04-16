// Adapted from https://github.com/OMGDuke/SDH-GameThemeMusic/blob/main/src/state/ShortAppDetailsState.tsx

import _ from 'lodash';
import {
    createContext,
    FC,
    useContext,
    useEffect,
    useState
} from 'react';
import { ApiError, AppProfile, getAppProfile, getDefaultAppOverrideForProfileRequest, getProfile, PipelineDefinition, PipelineTarget, reifyPipeline, ReifyPipelineResponse, RuntimeSelection, setAppProfileOverride, setAppProfileSettings, setProfile } from '../backend';
import { PipelineActionLookup } from '../types/backend_api';
import { MaybeString } from '../types/short';
import { Loading } from '../util/loading';
import { patchPipeline, PipelineUpdate } from "../util/patchPipeline";
import { Result } from '../util/result';


export interface StateAction {
    externalProfile: MaybeString
    update: PipelineUpdate,
}

export type ShortAppDetails = {
    appId: number,
    gameId: string,
    displayName: string,
};


interface PublicAppState {
    appDetails: ShortAppDetails | null
    appProfile: Loading<AppProfile>
    reifiedPipelines: { [k: string]: Result<ReifyPipelineResponse, ApiError> }
    openViews: { [k: string]: { [k: string]: boolean } },
}

// The localThemeEntry interface refers to the theme data as given by the python function, the Theme class refers to a theme after it has been formatted and the generate function has been added

interface PublicAppStateContext
    extends PublicAppState {
    setOnAppPage(appDetails: ShortAppDetails): void,
    setAppProfileDefault(appDetails: ShortAppDetails, defaultProfileId: string | null): Promise<void>
    setAppViewOpen(profileId: string, view: PipelineTarget, isOpen: boolean): void
    dispatchUpdate(profileId: string, action: StateAction): Promise<void>
    loadProfileOverride(appId: number, profileId: string): Promise<void>
}

// This class creates the getter and setter functions for all of the global state data.
export class ShortAppDetailsState {
    private readonly delayMs = 1000
    private appDetails: ShortAppDetails | null = null;
    private appProfile: Loading<AppProfile>;
    private reifiedPipelines: { [k: string]: Result<ReifyPipelineResponse, ApiError> } = {};
    private openViews: { [k: string]: { [k: string]: boolean } } = {};
    private lastOnAppPageTime: number = 0

    // You can listen to this eventBus' 'stateUpdate' event and use that to trigger a useState or other function that causes a re-render
    public readonly eventBus = new EventTarget()

    getPublicState(): PublicAppState {
        return {
            appDetails: this.appDetails ? { ...this.appDetails } : null,
            appProfile: this.appProfile ? { ... this.appProfile } : null,
            reifiedPipelines: { ...this.reifiedPipelines },
            openViews: { ...this.openViews }
        }
    }

    setOnAppPage(appDetails: ShortAppDetails | null) {
        const time = Date.now()

        setTimeout(
            () => {
                this.setOnAppPageInternal(appDetails, time)
            },
            appDetails ? 0 : this.delayMs
        )
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
            console.log('is external profile update');
            await this.updateExternalProfile(action.externalProfile, action.update);
        } else {
            if (this.appProfile?.isOk) {
                console.log(profileId, 'is pipeline update; current state:', this.appProfile);

                const pipeline = this.appProfile.data.overrides[profileId];
                if (pipeline) {
                    let res = (await patchPipeline(pipeline, action.update)).map(async (newPipeline) => {
                        return await this.setAppProfileOverride(
                            appId,
                            profileId,
                            newPipeline
                        )
                    });

                    if (res.isOk) {
                        await res.data;
                    } else {
                        console.log("error updating profile", res.err);
                    }
                } else {
                    console.log('pipeline should already be loaded before updating', pipeline);
                }
            }
        }

        await this.refetchProfile(profileId, appId)
    }

    async setAppProfileDefault(appDetails: ShortAppDetails, defaultProfileId: string) {
        const res = await setAppProfileSettings({
            app_id: appDetails.appId.toString(),
            default_profile: defaultProfileId
        });

        if (res?.isOk) {
            this.refetchProfile(defaultProfileId, appDetails.appId)
        } else {
            console.log('failed to set app(', appDetails.appId, ') default to', defaultProfileId);
        }
        // TODO::error handling
    }

    async loadProfileOverride(appId: number, profileId: string) {
        console.log("loading app profile");
        let shouldUpdate = false;
        if (this.appDetails?.appId === appId && this.appProfile?.isOk) {
            const overrides = this.appProfile.data.overrides;
            if (!overrides[profileId]) {
                const res = await getDefaultAppOverrideForProfileRequest({
                    profile_id: profileId
                });
                if (res.isOk && res.data.pipeline) {
                    overrides[profileId] = res.data.pipeline;


                    console.log('set override for', profileId, 'to', overrides[profileId]);
                    shouldUpdate = true;
                }
                // TODO::error handling
            } else {
                console.log('existing override found:', overrides[profileId]);
            }

            if (overrides[profileId]) {
                const res = (await reifyPipeline({ pipeline: overrides[profileId] })).map((res) =>
                    patchProfileOverridesForMissing(profileId, overrides[profileId], res));
                this.reifiedPipelines[profileId] = res
                console.log('load reified to:', this.reifiedPipelines[profileId]);
                shouldUpdate = true;
            }
        }

        if (shouldUpdate) {
            this.forceUpdate();
        }
    }

    private async setAppProfileOverride(appId: number, profileId: string, pipeline: PipelineDefinition) {
        const res = await setAppProfileOverride({
            app_id: appId.toString(),
            profile_id: profileId,
            pipeline,
        });

        if (res?.isOk && this.appDetails?.appId === appId) {
            this.appProfile = this.appProfile?.map((p) => {
                const overrides = {
                    ...p.overrides
                };
                overrides[profileId] = pipeline;

                return {
                    ...p,
                    overrides,
                }
            })
            this.refetchProfile(profileId, appId)
        } else {
            console.log('failed to set app(', appId, ') override for', profileId);
        }
        // TODO::error handling
    }

    private async updateExternalProfile(profileId: string, update: PipelineUpdate) {
        const profileResponse = await getProfile({
            profile_id: profileId,
        });

        if (profileResponse?.isOk) {
            const profile = profileResponse.data.profile;
            const pipeline = profile?.pipeline;
            if (pipeline) {
                const res = await (await patchPipeline(pipeline, update)).andThenAsync(async (res) => {
                    return await setProfile({
                        profile: {
                            ...profile,
                            pipeline: res
                        }
                    })
                });


                if (res?.isOk) {
                    this.refetchProfile(profileId)
                } else {
                    console.log('failed to set external profile', profileId)
                }
            } else {
                console.log('external profile', profileId, 'not found');
            }
            // TODO::error handling
        } else {
            console.log('failed to fetch external profile', profileId);
        }

        // TODO::error handling
    }

    private async refetchProfile(externalProfileId: string, appIdToMatch?: number) {
        const internal = async () => {
            if (this.appDetails && (!appIdToMatch || this.appDetails?.appId == appIdToMatch)) {
                const newProfile = (await getAppProfile({
                    app_id: this.appDetails.appId.toString()
                }))
                    .map((a) => a.app ?? null);

                if (this.appProfile?.isOk && newProfile.isOk) {
                    for (const key in this.appProfile.data.overrides) {
                        newProfile.data.overrides[key] ??= this.appProfile.data.overrides[key];
                    }
                }

                if (!this.appProfile?.isOk) {
                    console.log('failed to refetch app(', appIdToMatch, ')', this.appProfile?.err);
                } else {
                    const overrides = this.appProfile.data.overrides;
                    for (const k in overrides) {
                        let reified = (await reifyPipeline({
                            pipeline: overrides[k]
                        })).map((response) => patchProfileOverridesForMissing(externalProfileId, overrides[k], response));

                        this.reifiedPipelines[k] = reified;
                    }

                    console.log('refetched; updating to', this.appProfile.data?.overrides);
                }

                this.forceUpdate();
            }
        };
        await internal();
    }


    private setOnAppPageInternal(appDetails: ShortAppDetails | null, time: number) {
        const areEqual = _.isEqual(appDetails, this.appDetails);
        console.log('trying to set app to', appDetails?.displayName);
        if (time < this.lastOnAppPageTime || areEqual) {
            return;
        }

        console.log('setting app to ', appDetails?.displayName)

        this.appDetails = appDetails;
        this.appProfile = null;
        this.openViews = {};
        this.reifiedPipelines = {};
        this.lastOnAppPageTime = time;
        this.fetchProfile();
        this.forceUpdate();
    }

    private async fetchProfile() {
        const appDetails = this.appDetails;
        if (appDetails) {
            const profile = (await getAppProfile({ app_id: appDetails.appId.toString() })).map((v) => v.app ?? null)
            if (this.appDetails?.appId == appDetails.appId) {
                this.appProfile = profile;
                this.forceUpdate();
            }
        }
    }

    private forceUpdate() {
        this.eventBus.dispatchEvent(new Event('stateUpdate'))
    }
}

const AppContext =
    createContext<PublicAppStateContext>(null as any)
export const useAppState = () =>
    useContext(AppContext)

interface ProviderProps {
    ShortAppDetailsStateClass: ShortAppDetailsState
}

// This is a React Component that you can wrap multiple separate things in, as long as they both have used the same instance of the CssLoaderState class, they will have synced state
export const ShortAppDetailsStateContextProvider: FC<ProviderProps> = ({
    children,
    ShortAppDetailsStateClass
}) => {
    const [publicState, setPublicState] = useState<PublicAppState>({
        ...ShortAppDetailsStateClass.getPublicState()
    })

    useEffect(() => {
        function onUpdate() {
            setPublicState({ ...ShortAppDetailsStateClass.getPublicState() })
        }

        ShortAppDetailsStateClass.eventBus.addEventListener(
            'stateUpdate',
            onUpdate
        )

        return () =>
            ShortAppDetailsStateClass.eventBus.removeEventListener(
                'stateUpdate',
                onUpdate
            )
    }, [])


    const setOnAppPage = (appDetails: ShortAppDetails) =>
        ShortAppDetailsStateClass.setOnAppPage(appDetails)

    const setAppProfileDefault = async (appDetails: ShortAppDetails, defaultProfileId: string) => {
        ShortAppDetailsStateClass.setAppProfileDefault(appDetails, defaultProfileId);
    }

    const setAppViewOpen = (profileId: string, view: PipelineTarget, isOpen: boolean) => {
        ShortAppDetailsStateClass.setAppViewOpen(profileId, view, isOpen);
    }

    const dispatchUpdate = async (profileId: string, action: StateAction) => {
        ShortAppDetailsStateClass.dispatchUpdate(profileId, action)
    }

    const loadProfileOverride = async (appId: number, profileId: string) => {
        ShortAppDetailsStateClass.loadProfileOverride(appId, profileId);
    }

    return (
        <AppContext.Provider
            value={{
                ...publicState,
                setOnAppPage,
                setAppProfileDefault,
                setAppViewOpen,
                dispatchUpdate,
                loadProfileOverride
            }}
        >
            {children}
        </AppContext.Provider>
    )
}

function patchProfileOverridesForMissing(externalProfileId: string, overrides: PipelineDefinition, response: ReifyPipelineResponse): ReifyPipelineResponse {
    const pipeline = response.pipeline;

    const toplevel = [overrides.platform].concat(overrides.toplevel);

    function patch(selection: RuntimeSelection, actions: PipelineActionLookup) {
        const type = selection.type
        switch (type) {
            case 'Action':
                return;
            case 'OneOf':
                for (const v of selection.value.actions) {
                    if (!actions.actions[v.id]) {
                        v.profile_override = externalProfileId
                    }
                    patch(v.selection, actions)
                }
                return;
            case 'AllOf':
                for (const v of selection.value) {
                    if (!actions.actions[v.id]) {
                        v.profile_override = externalProfileId
                    }
                    patch(v.selection, actions)
                }
                return;

            default:
                const typecheck: never = type;
                throw typecheck ?? 'runtime selection failed to typecheck';
        }
    }

    for (const target in pipeline.targets) {
        let selection = pipeline.targets[target];
        if (selection.type === 'AllOf') {
            let skip = 0;
            for (const v in selection.value) {
                const i = parseInt(v);

                // handle case where ids mismatch because a toplevel action is invalid for a target
                while (selection.value[i].id !== toplevel[i + skip].root && i + skip < toplevel.length) {
                    skip += 1;
                }

                patch(selection.value[i].selection, toplevel[i + skip].actions);
            }
        } else {
            throw 'expected toplevel action to be AllOf'
        }
    }

    console.log('reify response after patch: ', response.pipeline);

    return response;
}