import { routerHook } from '@decky/api';
import {
    afterPatch,
    appDetailsClasses,
    basicAppDetailsSectionStylerClasses,
    findInReactTree,
    wrapReactClass,
    wrapReactType,
} from '@decky/ui';
import { ReactElement } from 'react';
import {
    ShortAppDetailsState,
    ShortAppDetailsStateContextProvider,
} from '../context/appContext';
import { debugPrintStyles } from '../util/debugPrint';
import PrimaryPlayButton from './components/PrimaryPlayButton';
import SecondaryPlayButton from './components/SecondaryPlayButton';

let cachedPlayButton: ReactElement | null = null;

function patchLibraryApp(route: string, appDetailsState: ShortAppDetailsState) {
    return routerHook.addPatch(
        route,
        (props?: { path?: string; children?: ReactElement }) => {
            if (!props?.children?.props?.renderFunc) {
                return props;
            }

            console.log('props', props);

            afterPatch(
                props.children.props,
                'renderFunc',
                (_: Record<string, unknown>[], ret?: ReactElement) => {
                    if (!ret?.props?.children?.type?.type) {
                        return ret;
                    }
                    debugPrintStyles();

                    console.log('ret', ret);

                    wrapReactType(ret.props.children);
                    afterPatch(
                        ret.props.children.type,
                        'type',
                        (
                            _2: Record<string, unknown>[],
                            ret2?: ReactElement,
                        ) => {
                            const container = findInReactTree(
                                ret2,
                                (x: ReactElement) =>
                                    Array.isArray(x?.props?.children) &&
                                    x?.props?.className?.includes(
                                        appDetailsClasses.InnerContainer,
                                    ),
                            );

                            const overview =
                                ret.props?.children?.props?.overview;

                            if (typeof container !== 'object' || !overview) {
                                return ret2;
                            }

                            console.log('ret2', ret2);
                            console.log('ret2 container', container);

                            const appId = overview.appid;

                            const children = container.props.children;
                            const child = children.find((c: any) =>
                                c?.props?.className?.includes(
                                    appDetailsClasses.AppDetailsOverviewPanel,
                                ),
                            );

                            console.log('ret2 child', child);

                            wrapReactType(child);
                            afterPatch(
                                child.type,
                                'render',
                                (
                                    _3: Record<string, unknown>[],
                                    ret3?: ReactElement,
                                ) => {
                                    if (!ret3) {
                                        return ret3;
                                    }

                                    console.log('ret3', ret3);
                                    ret3.key = 'ret3';

                                    const child = findInReactTree(
                                        ret3,
                                        (x: ReactElement) => x?.props?.overview,
                                    );

                                    console.log('ret3 child', child);
                                    child.key = 'ret3_child';

                                    wrapReactClass(child);
                                    afterPatch(
                                        child.type.prototype,
                                        'render',
                                        (
                                            _4: Record<string, unknown>[],
                                            ret4?: ReactElement,
                                        ) => {
                                            if (!ret4) {
                                                return ret4;
                                            }

                                            console.log('ret4', ret4);
                                            ret4.key = 'ret4';

                                            const child = findInReactTree(
                                                ret4,
                                                (x: ReactElement) =>
                                                    x?.props?.overview,
                                            );

                                            console.log('ret4 child', child);
                                            child.key = 'ret4_child';
                                            afterPatch(
                                                child,
                                                'type',
                                                (_5, ret5) => {
                                                    console.log('ret5', ret5);
                                                    ret5.key = 'ret5';

                                                    afterPatch(
                                                        ret5,
                                                        'type',
                                                        (_6, ret6) => {
                                                            console.log(
                                                                'ret6',
                                                                ret6,
                                                            );
                                                            ret6.key = 'ret6';

                                                            const ret6Child =
                                                                findInReactTree(
                                                                    ret6,
                                                                    (x) =>
                                                                        x?.props
                                                                            ?.overview &&
                                                                        x?.type
                                                                            ?.render,
                                                                );

                                                            console.log(
                                                                'ret6 child',
                                                                ret6Child,
                                                            );

                                                            if (!ret6Child) {
                                                                return ret6;
                                                            }

                                                            ret6Child.key =
                                                                'ret6child';
                                                            wrapReactType(
                                                                ret6Child,
                                                            );

                                                            afterPatch(
                                                                ret6Child.type,
                                                                'render',
                                                                (_7, ret7) => {
                                                                    console.log(
                                                                        'ret7',
                                                                        ret7,
                                                                    );
                                                                    ret7.key =
                                                                        'ret7';
                                                                    return ret7;
                                                                },
                                                            );

                                                            return ret6;

                                                            wrapReactType(ret6);
                                                            afterPatch(
                                                                ret6.type,
                                                                'render',
                                                                (_7, ret7) => {
                                                                    console.log(
                                                                        'ret7',
                                                                        ret7,
                                                                    );
                                                                    ret7.key =
                                                                        'ret7';

                                                                    const child =
                                                                        findInReactTree(
                                                                            ret7,
                                                                            (
                                                                                x,
                                                                            ) =>
                                                                                x
                                                                                    ?.type
                                                                                    ?.render &&
                                                                                x
                                                                                    ?.props
                                                                                    ?.overview,
                                                                        );
                                                                    console.log(
                                                                        'ret7 child',
                                                                        child,
                                                                    );

                                                                    return ret7;

                                                                    if (
                                                                        !child
                                                                    ) {
                                                                        return ret7;
                                                                    }

                                                                    child.key =
                                                                        'ret7 child';

                                                                    wrapReactType(
                                                                        child,
                                                                    );
                                                                    afterPatch(
                                                                        child.type,
                                                                        'render',
                                                                        (
                                                                            _8,
                                                                            ret8,
                                                                        ) => {
                                                                            console.log(
                                                                                'ret8',
                                                                                ret8,
                                                                            );
                                                                            ret8.key =
                                                                                'ret8';

                                                                            const overview =
                                                                                appStore.GetAppOverviewByAppID(
                                                                                    appId,
                                                                                );

                                                                            patchFinalElement(
                                                                                ret8,
                                                                                overview,
                                                                                appDetailsState,
                                                                            );

                                                                            return ret8;
                                                                        },
                                                                    );

                                                                    return ret7;
                                                                },
                                                            );

                                                            return ret6;
                                                        },
                                                    );

                                                    return ret5;
                                                },
                                            );

                                            return ret4;
                                        },
                                    );

                                    return ret3;
                                },
                            );

                            return ret2;
                        },
                    );

                    return ret;
                },
            );

            return props;
        },
    );
}

function patchFinalElement(
    ret: ReactElement,
    overview: any,
    appDetailsState: ShortAppDetailsState,
) {
    const status = overview.per_client_data.find(
        (d: any) => d.clientid === overview.selected_clientid,
    );

    const streaming = status.clientid !== '0';
    const installed: boolean =
        !streaming && status.status_percentage == 100 && status.installed;

    const appButtons = findInReactTree(
        ret,
        (x: ReactElement) =>
            Array.isArray(x?.props?.children) &&
            x?.props?.className?.includes(
                basicAppDetailsSectionStylerClasses.AppButtons, // _1thLDT_28YIf6OkgIb6n-4
            ),
    );

    // const playButtonStatusPanel = findInReactTree(
    //     ret,
    //     (x: ReactElement) => Array.isArray(x?.props?.children) &&
    //         x?.props?.className?.includes(
    //             basicAppDetailsSectionStylerClasses.ActionButtonAndStatusPanel
    //         )

    // )

    const playButton = findInReactTree(ret, (x: ReactElement) => {
        return (
            Array.isArray(x?.props?.children) &&
            x?.props?.className?.includes(
                basicAppDetailsSectionStylerClasses.AppActionButton, // QsZdWtHTlIK9KIKbscNTt
            )
        );
    });
    const missingAppButtons = typeof appButtons !== 'object';
    const missingPlayButton = typeof playButton !== 'object';

    console.log('appbuttons', appButtons);

    console.log('playbutton', playButton);

    if (!missingAppButtons) {
        const children = appButtons?.props?.children;

        if (
            installed &&
            children &&
            !children.find(
                (c: any) =>
                    c?.props?.children?.props?.deckDSDesktopSentinel ===
                    'sentinel',
            )
        ) {
            children.splice(
                0,
                0,
                <ShortAppDetailsStateContextProvider
                    ShortAppDetailsStateClass={appDetailsState}
                >
                    <SecondaryPlayButton deckDSDesktopSentinel="sentinel" />
                </ShortAppDetailsStateContextProvider>,
            );
        } else {
            const sentinel = children.findIndex(
                (c: any) =>
                    c?.props?.children?.props?.deckDSDesktopSentinel ===
                    'sentinel',
            );

            if (!installed && sentinel >= 0) {
                children.splice(sentinel, 1);
            }
        }
    }

    if (!missingPlayButton) {
        const children = playButton?.props?.children;

        if (
            installed &&
            children &&
            !children.find(
                (c: any) =>
                    c?.props?.children?.props?.deckDSGameModeSentinel ===
                    'sentinel',
            )
        ) {
            const actualPlayButton = children[0];
            cachedPlayButton = actualPlayButton;

            children.splice(
                0,
                1,
                <ShortAppDetailsStateContextProvider
                    ShortAppDetailsStateClass={appDetailsState}
                >
                    <PrimaryPlayButton
                        playButton={actualPlayButton}
                        deckDSGameModeSentinel="sentinel"
                    />
                </ShortAppDetailsStateContextProvider>,
            );
        } else {
            const sentinel = children.findIndex(
                (c: any) =>
                    c?.props?.children?.props?.deckDSGameModeSentinel ===
                    'sentinel',
            );

            if (sentinel >= 0 && !installed && cachedPlayButton) {
                children.splice(sentinel, 1, cachedPlayButton);
            }
        }
    }
}

export default patchLibraryApp;
