import {
    ServerAPI,
    afterPatch,
    appDetailsClasses,
    findInReactTree,
    wrapReactType
} from 'decky-frontend-lib';
import { ReactElement, useEffect } from 'react';
import { ShortAppDetails, ShortAppDetailsState, ShortAppDetailsStateContextProvider, useShortAppDetailsState } from '../context/shortAppDetailsContext';

function patchLibraryApp(
    serverAPI: ServerAPI,
    shortAppDetailsState: ShortAppDetailsState
) {
    return serverAPI.routerHook.addPatch(
        '/library/app/:appid',
        (props?: { path?: string; children?: ReactElement }) => {
            if (!props?.children?.props?.renderFunc) {
                return props
            }

            afterPatch(
                props.children.props,
                'renderFunc',
                (_: Record<string, unknown>[], ret?: ReactElement) => {
                    if (!ret?.props?.children?.type?.type) {
                        return ret
                    }

                    const displayName: string =
                        ret.props.children.props.overview.display_name
                    const appId: number =
                        ret.props.children.props.overview.appid;
                    const gameId: string = ret.props.children.props.overview.m_gameid;

                    console.log('DeckDS:', displayName, '(', appId, ')');

                    wrapReactType(ret.props.children)
                    afterPatch(
                        ret.props.children.type,
                        'type',
                        (_2: Record<string, unknown>[], ret2?: ReactElement) => {
                            const container = findInReactTree(
                                ret2,
                                (x: ReactElement) =>
                                    Array.isArray(x?.props?.children) &&
                                    x?.props?.className?.includes(
                                        appDetailsClasses.InnerContainer
                                    )
                            )
                            if (typeof container !== 'object') {
                                return ret2
                            }

                            container.props.children.push(
                                <ShortAppDetailsStateContextProvider
                                    ShortAppDetailsStateClass={shortAppDetailsState}
                                >
                                    <AppPageLogic appDetails={{
                                        displayName,
                                        appId,
                                        gameId
                                    }} />
                                </ShortAppDetailsStateContextProvider>
                            )

                            return ret2
                        });



                    return ret;
                }
            )
            return props
        }
    )
}

export default patchLibraryApp;

function AppPageLogic({ appDetails }: { appDetails: ShortAppDetails }): ReactElement {
    const state = useShortAppDetailsState();


    useEffect(() => {
        if (state.appDetails?.appId != appDetails?.appId) {
            state.setOnAppPage(appDetails)
        }
        return () => state.setOnAppPage(null);
    },
        [appDetails?.appId]);

    return <div />
}

