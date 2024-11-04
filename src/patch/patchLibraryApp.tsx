import { routerHook } from '@decky/api';
import {
    afterPatch,
    appDetailsClasses,
    basicAppDetailsSectionStylerClasses,
    findInReactTree,
    replacePatch,
    wrapReactClass,
    wrapReactType,
} from '@decky/ui';
import { ReactElement } from 'react';
import {
    ShortAppDetailsState,
    ShortAppDetailsStateContextProvider,
} from '../context/appContext';
import { debugPrintStyles } from '../util/debugPrint';
import { isSteamGame } from '../util/util';
import PrimaryPlayButton from './components/PrimaryPlayButton';
import SecondaryPlayButton from './components/SecondaryPlayButton';

let cachedPlayButton: ReactElement | null = null;
let argCache: any = {};
let lastOnNavTime = 0;
const onNavDebounceTime = 5000;

function deepCompareKeys(obj1: any, obj2: any, cache: Set<any>) {
    if (cache.has(obj1) || cache.has(obj2)) {
        const ret = cache.has(obj1) && cache.has(obj2);
        if (!ret) {
            console.log(
                'cache mismatch?:',
                cache.has(obj1),
                'vs',
                cache.has(obj2),
            );
        }

        return ret;
    }

    cache.add(obj1);
    cache.add(obj2);

    if (typeof obj1 !== 'object' || typeof obj2 !== 'object') {
        const ret = obj1 === obj2;
        if (!ret) {
            console.log('primitive mismatch:', obj1, 'vs', obj2);
        }

        return ret;
    }

    if (obj1 === null || obj2 === null) {
        const ret = obj1 === obj2;
        if (!ret) {
            console.log('null mismatch:', obj1, 'vs', obj2);
        }

        return ret;
    }

    const keys1 = Object.keys(obj1);
    const keys2 = Object.keys(obj2);

    if (keys1.length !== keys2.length) {
        console.log('key length mismatch:', obj1, 'vs', obj2);
        return false;
    }

    for (const key of keys1) {
        if (!keys2.includes(key)) {
            console.log('missing key', key, ':', obj1, 'vs', obj2);

            return false;
        }
        if (!deepCompareKeys(obj1[key], obj2[key], cache)) {
            console.log('value mismatch:', obj1, 'vs', obj2);

            return false;
        }
    }

    return true;
}

function checkCachedArg(name: string, args: any) {
    let ret = true;
    if (argCache[name] !== args) {
        console.log(name, argCache[name], 'vs', args);
        if (deepCompareKeys(argCache[name], args, new Set())) {
            console.log(name, 'are actually the same though...');
        } else {
            ret = false;
        }
    }
    argCache[name] = args;
    return ret;
}

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
                (_1: Record<string, unknown>[], ret?: ReactElement) => {
                    if (!ret?.props?.children?.type?.type) {
                        return ret;
                    }

                    console.log('ret', ret);

                    // findModuleExport((e) => {
                    //     if (!e || typeof e === 'string') {
                    //         return false;
                    //     }
                    //     if (!e.toString) {
                    //         console.log('module export:', typeof e, e);
                    //     } else {
                    //         console.log(
                    //             'module export str:',
                    //             typeof e,
                    //             e.toString(),
                    //         );
                    //     }
                    //     return false;
                    // });
                    debugPrintStyles();

                    wrapReactType(ret.props.children);
                    afterPatch(
                        ret.props.children.type,
                        'type',
                        (
                            _2: Record<string, unknown>[],
                            ret2?: ReactElement,
                        ) => {
                            console.log('ret2', ret2);
                            const container = findInReactTree(
                                ret2,
                                (x: ReactElement) =>
                                    Array.isArray(x?.props?.children) &&
                                    x?.props?.className?.includes(
                                        appDetailsClasses.InnerContainer,
                                    ),
                            );

                            if (typeof container !== 'object') {
                                return ret2;
                            }

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
                                    replacePatch(
                                        ret3.props.value,
                                        'm_bAutoFocusChild',
                                        () => false,
                                    );

                                    // replacePatch(
                                    //     ret3.props.value,
                                    //     'm_bAutoFocusChild',
                                    //     (args) => {
                                    //         console.log(
                                    //             'ret2 child args',
                                    //             args,
                                    //         );
                                    //         return false;
                                    //     },
                                    // );

                                    const child = findInReactTree(
                                        ret3,
                                        (x: ReactElement) => x?.props?.overview,
                                    );

                                    // const playButtonStatusPanel = findInReactTree(
                                    //     ret3,
                                    //     (x: ReactElement) => Array.isArray(x?.props?.children) &&
                                    //         x?.props?.className?.includes(
                                    //             basicAppDetailsSectionStylerClasses.ActionButtonAndStatusPanel // _1fHBRg7vFnKszK6EiOdIEY
                                    //         )

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

                                            child.key = 'ret4_child';
                                            afterPatch(
                                                child,
                                                'type',
                                                (_5, ret5) => {
                                                    console.log('ret5', ret5);

                                                    ret5.key = 'ret5';
                                                    let ret6incr = 0;

                                                    afterPatch(
                                                        ret5,
                                                        'type',
                                                        (_6, ret6) => {
                                                            console.log(
                                                                'ret6',
                                                                ret6,
                                                            );

                                                            ret6.key = 'ret6';

                                                            const playSection =
                                                                findInReactTree(
                                                                    ret6,
                                                                    (x) =>
                                                                        x?.props
                                                                            ?.overview &&
                                                                        x?.type
                                                                            ?.render,
                                                                );

                                                            const appDetailsSection =
                                                                findInReactTree(
                                                                    ret6,
                                                                    (x) =>
                                                                        x?.props?.className?.includes(
                                                                            basicAppDetailsSectionStylerClasses.AppDetailsContainer,
                                                                        ) &&
                                                                        x?.type
                                                                            ?.render,
                                                                );

                                                            console.log(
                                                                'ret6 child',
                                                                playSection,
                                                                appDetailsSection,
                                                            );

                                                            const overview =
                                                                playSection
                                                                    ?.props
                                                                    ?.overview;

                                                            const status =
                                                                overview.per_client_data.find(
                                                                    (d: any) =>
                                                                        d.clientid ===
                                                                        overview.selected_clientid,
                                                                );

                                                            const streaming =
                                                                status.clientid !==
                                                                '0';
                                                            const installed: boolean =
                                                                !streaming &&
                                                                status.status_percentage ==
                                                                    100 &&
                                                                status.installed;

                                                            appDetailsState.setOnAppPage(
                                                                {
                                                                    appId: overview.appid,
                                                                    gameId: overview.m_gameid,
                                                                    sortAs: overview.sort_as,
                                                                    userId64:
                                                                        App
                                                                            .m_CurrentUser
                                                                            .strSteamID,
                                                                    isSteamGame:
                                                                        isSteamGame(
                                                                            overview,
                                                                        ),
                                                                    selected_clientid:
                                                                        overview.selected_clientid,
                                                                },
                                                            );

                                                            if (!installed) {
                                                                console.log(
                                                                    'not installed',
                                                                );

                                                                return ret6;
                                                            }

                                                            console.log(
                                                                'have play section, and installed',
                                                                overview,
                                                                status,
                                                                installed,
                                                            );

                                                            playSection.key =
                                                                'ret6child';

                                                            for (const v of [
                                                                [0, 'onNav'],
                                                                [1, 'onFocus'],
                                                                [
                                                                    2,
                                                                    'onFocusWithin',
                                                                ],
                                                                [
                                                                    3,
                                                                    'onFocusWithin',
                                                                ],
                                                            ]) {
                                                                const child =
                                                                    ret6.props
                                                                        .children[
                                                                        v[0]
                                                                    ];

                                                                const onFocusWithin =
                                                                    child.props[
                                                                        v[1]
                                                                    ];
                                                                // wrapReactType(
                                                                //     child,
                                                                // );
                                                                replacePatch(
                                                                    child.props,
                                                                    v[1] as string,
                                                                    (
                                                                        _focusArgs,
                                                                    ) => {
                                                                        console.log(
                                                                            'ret6 focuswithin',
                                                                            v[0],
                                                                            v[1],
                                                                            _focusArgs,
                                                                            onFocusWithin,
                                                                        );

                                                                        return;

                                                                        lastOnNavTime =
                                                                            Date.now();

                                                                        console.log(
                                                                            'handling focus within...',
                                                                        );

                                                                        onFocusWithin(
                                                                            ..._focusArgs,
                                                                        );
                                                                    },
                                                                );
                                                            }

                                                            wrapReactType(
                                                                playSection,
                                                            );

                                                            replacePatch(
                                                                playSection,
                                                                'ref',
                                                                () => {
                                                                    return null;
                                                                },
                                                            );
                                                            // const onNav =
                                                            //     playSection
                                                            //         .props
                                                            //         .onNav;
                                                            // replacePatch(
                                                            //     playSection.props,
                                                            //     'onNav',
                                                            //     (args) => {
                                                            //         console.log(
                                                            //             'ret6child onnav',
                                                            //             args,
                                                            //         );

                                                            //         const elapsed =
                                                            //             Date.now() -
                                                            //             lastOnNavTime;
                                                            //         if (
                                                            //             (!installed ||
                                                            //                 ret6incr ===
                                                            //                     0) &&
                                                            //             elapsed >
                                                            //                 onNavDebounceTime
                                                            //         ) {
                                                            //             console.log(
                                                            //                 'calling onNav',
                                                            //                 ret6incr,
                                                            //             );
                                                            //             onNav(
                                                            //                 ...args,
                                                            //             );
                                                            //         } else {
                                                            //             console.log(
                                                            //                 'calling onNav debounce',
                                                            //                 ret6incr,
                                                            //             );
                                                            //         }

                                                            //         ret6incr += 1;
                                                            //         ret6incr %= 3;
                                                            //     },
                                                            // );

                                                            afterPatch(
                                                                playSection.type,
                                                                'render',
                                                                (_7, ret7) => {
                                                                    console.log(
                                                                        'ret7',
                                                                        ret7,
                                                                    );
                                                                    ret7.key =
                                                                        'ret7';

                                                                    const ret7Child =
                                                                        findInReactTree(
                                                                            ret7,
                                                                            (
                                                                                v,
                                                                            ) =>
                                                                                v !==
                                                                                    ret7 &&
                                                                                v
                                                                                    ?.type
                                                                                    ?.render,
                                                                        );
                                                                    ret7Child.key =
                                                                        'ret7Child';

                                                                    console.log(
                                                                        'ret7Child',
                                                                        ret7Child,
                                                                    );

                                                                    wrapReactType(
                                                                        ret7Child,
                                                                    );

                                                                    afterPatch(
                                                                        ret7Child.type,
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
        return x?.props?.className?.includes(
            basicAppDetailsSectionStylerClasses.AppActionButton, // QsZdWtHTlIK9KIKbscNTt
        );
    });
    const missingAppButtons = typeof appButtons !== 'object';
    const missingPlayButton = typeof playButton !== 'object';

    console.log('appbuttons', appButtons);

    console.log('playbutton', playButton);

    if (!missingAppButtons) {
        const children = appButtons?.props?.children;

        if (children) {
            if (
                installed &&
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
    }

    if (!missingPlayButton) {
        wrapReactType(playButton);
        afterPatch(playButton.type, 'render', (_play, retPlayButton) => {
            console.log('retPlayButton', retPlayButton);
            const ref = retPlayButton.ref;
            if (ref) {
                ref.current = null;

                setTimeout(() => {
                    ref.current = null;
                }, 100);
            }

            wrapReactClass(retPlayButton);
            afterPatch(
                retPlayButton.type.prototype,
                'render',
                (_playClass, classPlayButton) => {
                    console.log('classPlayButton', classPlayButton);

                    const children = classPlayButton?.props?.children;

                    if (children) {
                        if (
                            installed &&
                            !children.find(
                                (c: any) =>
                                    c?.props?.children?.props
                                        ?.deckDSGameModeSentinel === 'sentinel',
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
                                    c?.props?.children?.props
                                        ?.deckDSGameModeSentinel === 'sentinel',
                            );

                            if (
                                sentinel >= 0 &&
                                !installed &&
                                cachedPlayButton
                            ) {
                                children.splice(sentinel, 1, cachedPlayButton);
                            }
                        }
                    }

                    return classPlayButton;
                },
            );
            return retPlayButton;
        });
    }
}

export default patchLibraryApp;
