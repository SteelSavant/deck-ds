// Adapted from https://github.com/OMGDuke/SDH-GameThemeMusic/blob/main/src/state/ShortAppDetailsState.tsx

import _ from 'lodash';
import {
    createContext,
    FC,
    useContext,
    useEffect,
    useState
} from 'react';
import { getAppProfile, getProfile, PipelineDefinition, setAppProfileOverride, setAppProfileSettings, setProfile } from '../backend';
import { AppProfile } from '../types/backend_api';
import { Loading } from '../util/loading';
import { patchPipeline, PipelineUpdate } from '../util/patch';

export type ShortAppDetails = {
    appId: number,
    gameId: string,
    displayName: string,
};

interface PublicShortAppDetailsState {
    appDetails: ShortAppDetails | null
    appProfile: Loading<AppProfile | null>
}

// The localThemeEntry interface refers to the theme data as given by the python function, the Theme class refers to a theme after it has been formatted and the generate function has been added

interface PublicAppStateContext
    extends PublicShortAppDetailsState {
    setOnAppPage(appDetails: ShortAppDetails): void,
    setAppProfileDefault(appDetails: ShortAppDetails, defaultProfileId: string | null): Promise<void>
    setAppProfileOverride(appDetails: ShortAppDetails, profileId: string, pipeline: PipelineDefinition): Promise<void>
    updateExternalProfile(profileId: string, update: PipelineUpdate): Promise<void>
}

// This class creates the getter and setter functions for all of the global state data.
export class ShortAppDetailsState {
    private delayMs = 1000
    private appDetails: ShortAppDetails | null = null;
    private appProfile: Loading<AppProfile | null>;
    private lastOnAppPageTime: number = 0

    // You can listen to this eventBus' 'stateUpdate' event and use that to trigger a useState or other function that causes a re-render
    public eventBus = new EventTarget()

    getPublicState(): PublicShortAppDetailsState {
        return {
            appDetails: this.appDetails ? { ...this.appDetails } : null,
            appProfile: this.appProfile ? { ...this.appProfile } : null
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

    async setAppProfileDefault(appDetails: ShortAppDetails, defaultProfileId: string | null) {
        const res = await setAppProfileSettings({
            app_id: appDetails.appId.toString(),
            default_profile: defaultProfileId
        });

        if (res?.isOk) {
            this.refetchProfile(appDetails.appId)
        } else {
            console.log('failed to set app(', appDetails.appId, ') default to', defaultProfileId);
        }
        // TODO::error handling
    }

    async setAppProfileOverride(appDetails: ShortAppDetails, profileId: string, pipeline: PipelineDefinition) {
        const res = await setAppProfileOverride({
            app_id: appDetails.appId.toString(),
            profile_id: profileId,
            pipeline,
        });

        if (res?.isOk) {
            this.refetchProfile(appDetails.appId)
        } else {
            console.log('failed to set app(', appDetails.appId, ') override for', profileId);
        }
        // TODO::error handling
    }

    async updateExternalProfile(profileId: string, update: PipelineUpdate) {
        const profileResponse = await getProfile({
            profile_id: profileId,
        });

        if (profileResponse?.isOk) {
            const profile = profileResponse.data.profile;
            const pipeline = profile?.pipeline;
            if (pipeline) {
                const newPipeline = patchPipeline(pipeline, update);
                const res = await setProfile({
                    profile: {
                        ...profile,
                        pipeline: newPipeline
                    }
                });

                if (res?.isOk) {
                    this.refetchProfile()
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

    private async refetchProfile(appIdToMatch?: number) {
        if (this.appDetails && (!appIdToMatch || this.appDetails?.appId == appIdToMatch)) {
            this.appProfile = (await getAppProfile({
                app_id: this.appDetails.appId.toString()
            }))
                .map((a) => a.app ?? null);

            if (!this.appProfile?.isOk) {
                console.log('failed to refetch app(', appIdToMatch, ')');
            }

            this.forceUpdate();
        }
    }


    private setOnAppPageInternal(appDetails: ShortAppDetails | null, time: number) {
        const areEqual = _.isEqual(appDetails, this.appDetails);
        if (time < this.lastOnAppPageTime || areEqual) {
            return;
        }

        this.appDetails = appDetails;
        this.appProfile = null;
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
    const [publicState, setPublicState] = useState<PublicShortAppDetailsState>({
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

    const setAppProfileDefault = async (appDetails: ShortAppDetails, defaultProfileId: string | null) => {
        ShortAppDetailsStateClass.setAppProfileDefault(appDetails, defaultProfileId);
    }

    const setAppProfileOverride = async (appDetails: ShortAppDetails, profileId: string, pipeline: PipelineDefinition) => {
        ShortAppDetailsStateClass.setAppProfileOverride(appDetails, profileId, pipeline);
    }

    const updateExternalProfile = async (profileId: string, update: PipelineUpdate) => {
        ShortAppDetailsStateClass.updateExternalProfile(profileId, update);
    }

    return (
        <AppContext.Provider
            value={{
                ...publicState,
                setOnAppPage,
                setAppProfileDefault,
                setAppProfileOverride,
                updateExternalProfile
            }}
        >
            {children}
        </AppContext.Provider>
    )
}