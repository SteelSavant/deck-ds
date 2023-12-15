// Adapted from https://github.com/OMGDuke/SDH-GameThemeMusic/blob/main/src/state/ShortAppDetailsState.tsx

import {
    createContext,
    FC,
    useContext,
    useEffect,
    useState
} from 'react';

export type ShortAppDetails = {
    appId: number,
    gameId: string,
    displayName: string,
};

interface PublicShortAppDetailsState {
    gamesRunning: number[]
    appDetails: ShortAppDetails | null
}

// The localThemeEntry interface refers to the theme data as given by the python function, the Theme class refers to a theme after it has been formatted and the generate function has been added

interface PublicShortAppDetailsStateContext
    extends PublicShortAppDetailsState {
    setGamesRunning(gamesRunning: number[]): void
    setOnAppPage(appDetails: ShortAppDetails): void
}

// This class creates the getter and setter functions for all of the global state data.
export class ShortAppDetailsState {
    private delayMs = 1000
    private gamesRunning: number[] = []
    private appDetails: ShortAppDetails | null = null;
    private lastOnAppPageTime: number = 0

    // You can listen to this eventBus' 'stateUpdate' event and use that to trigger a useState or other function that causes a re-render
    public eventBus = new EventTarget()

    getPublicState(): PublicShortAppDetailsState {
        return {
            gamesRunning: this.gamesRunning,
            appDetails: this.appDetails ? { ...this.appDetails } : null
        }
    }

    setGamesRunning(gamesRunning: number[]) {
        const noGamesRunning = gamesRunning.length === 0
        this.gamesRunning = gamesRunning

        setTimeout(
            () => {
                this.forceUpdate()
            },
            noGamesRunning ? this.delayMs : 0
        )
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


    private setOnAppPageInternal(appDetails: ShortAppDetails | null, time: number) {
        if (time < this.lastOnAppPageTime) {
            return;
        }
        this.appDetails = appDetails;
        this.lastOnAppPageTime = time;
        this.forceUpdate();
    }

    private forceUpdate() {
        this.eventBus.dispatchEvent(new Event('stateUpdate'))
    }
}

const ShortAppDetailsStateContext =
    createContext<PublicShortAppDetailsStateContext>(null as any)
export const useShortAppDetailsState = () =>
    useContext(ShortAppDetailsStateContext)

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

    const setGamesRunning = (gamesRunning: number[]) =>
        ShortAppDetailsStateClass.setGamesRunning(gamesRunning)
    const setOnAppPage = (appDetails: ShortAppDetails) =>
        ShortAppDetailsStateClass.setOnAppPage(appDetails)

    return (
        <ShortAppDetailsStateContext.Provider
            value={{
                ...publicState,
                setGamesRunning,
                setOnAppPage
            }}
        >
            {children}
        </ShortAppDetailsStateContext.Provider>
    )
}