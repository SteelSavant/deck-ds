import {
    ServerAPI,
    afterPatch,
    appDetailsClasses,
    basicAppDetailsSectionStylerClasses,
    findInReactTree,
    wrapReactType
} from 'decky-frontend-lib';
import { ReactElement } from 'react';
import { ShortAppDetailsState, ShortAppDetailsStateContextProvider } from '../context/shortAppDetailsContext';
import DesktopPlayButton from './DesktopPlayButton';
import GameModePlayButton from './GameModePlayButton';

// TODO::don't patch if appid doesn't have pipeline
// TODO::patch in real button

function patchLibraryApp(serverAPI: ServerAPI, appDetailsState: ShortAppDetailsState) {
    return serverAPI.routerHook.addPatch(
        '/library/app/:appid',
        (props?: { path?: string; children?: ReactElement }) => {
            if (!props?.children?.props?.renderFunc) {
                return props;
            }

            afterPatch(
                props.children.props,
                'renderFunc',
                (_: Record<string, unknown>[], ret?: ReactElement) => {
                    if (!ret?.props?.children?.type?.type) {
                        return ret;
                    }


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
                                return ret2;
                            }

                            const children = container.props.children;
                            const child = children.find((c: any) => c?.type?.render);

                            wrapReactType(child.type);
                            afterPatch(child.type, 'render', (_3: Record<string, unknown>[], ret3?: ReactElement) => {
                                if (!ret3) {
                                    return ret3;
                                }

                                const appButtons = findInReactTree(
                                    ret3,
                                    (x: ReactElement) =>
                                        Array.isArray(x?.props?.children) &&
                                        x?.props?.className?.includes(
                                            basicAppDetailsSectionStylerClasses.AppButtons
                                        )
                                );

                                const playButtonStatusPanel = findInReactTree(
                                    ret3,
                                    (x: ReactElement) =>
                                        Array.isArray(x?.props?.children) &&
                                        x?.props?.className?.includes(
                                            basicAppDetailsSectionStylerClasses.ActionButtonAndStatusPanel
                                        )
                                )

                                const missingAppButtons = typeof appButtons !== 'object';
                                const missingPlayButtonStatusPanel = typeof playButtonStatusPanel !== 'object';

                                if (!missingAppButtons) {
                                    const children = appButtons?.props?.children;

                                    console.log('appButtons:', children);

                                    if (!children.find((c: any) => c?.props?.children?.props?.deckDSDesktopSentinel === 'sentinel')) {
                                        children?.splice(0, 0,
                                            <ShortAppDetailsStateContextProvider ShortAppDetailsStateClass={appDetailsState}>
                                                <DesktopPlayButton
                                                    deckDSDesktopSentinel='sentinel'
                                                />
                                            </ShortAppDetailsStateContextProvider>
                                        )
                                    }
                                }

                                if (!missingPlayButtonStatusPanel) {
                                    console.log('ret3 playButton', playButtonStatusPanel);
                                    const children = playButtonStatusPanel?.props?.children;
                                    if (children && !children.find((c: any) => c?.props?.children?.props?.deckDSGameModeSentinel === 'sentinel')) {
                                        const actualPlayButton = children[0];
                                        children?.splice(0, 1,
                                            <ShortAppDetailsStateContextProvider ShortAppDetailsStateClass={appDetailsState}>
                                                <GameModePlayButton
                                                    playButton={actualPlayButton}
                                                    deckDSGameModeSentinel='sentinel'
                                                />
                                            </ShortAppDetailsStateContextProvider>);
                                    }
                                }

                                return ret3;
                            });

                            return ret2;
                        })

                    return ret;
                })

            return props;
        },
    )
}

export default patchLibraryApp;