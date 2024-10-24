import { routerHook } from '@decky/api';
import {
    afterPatch,
    appDetailsClasses,
    basicAppDetailsSectionStylerClasses,
    beforePatch,
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
let argCache: any = {};

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
    if (argCache[name] !== args) {
        console.log(name, argCache[name], 'vs', args);
        if (Array.isArray(args)) {
            console.log('array is different array');
        }
        if (deepCompareKeys(argCache[name], args, new Set())) {
            console.log(name, 'are actually the same though...');
            return argCache[name];
        }
    }
    argCache[name] = args;
    return args;
}

function patchLibraryApp(route: string, appDetailsState: ShortAppDetailsState) {
    return routerHook.addPatch(
        route,
        (props?: { path?: string; children?: ReactElement }) => {
            if (!props?.children?.props?.renderFunc) {
                return props;
            }

            console.log('props', props);

            const p1 = afterPatch(
                props.children.props,
                'renderFunc',
                (_1: Record<string, unknown>[], ret?: ReactElement) => {
                    if (!ret?.props?.children?.type?.type) {
                        return ret;
                    }
                    debugPrintStyles();

                    console.log('ret', ret);
                    checkCachedArg('_1', _1);

                    wrapReactType(ret.props.children);
                    const p2 = afterPatch(
                        ret.props.children.type,
                        'type',
                        (
                            _2: Record<string, unknown>[],
                            ret2?: ReactElement,
                        ) => {
                            checkCachedArg('_2', _2);

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
                            const p3 = afterPatch(
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
                                    checkCachedArg('_3', _3);

                                    const child = findInReactTree(
                                        ret3,
                                        (x: ReactElement) => x?.props?.overview,
                                    );

                                    console.log('ret3 child', child);
                                    child.key = 'ret3_child';

                                    wrapReactClass(child);
                                    const p4 = afterPatch(
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
                                            checkCachedArg('_4', _4);

                                            ret4.key = 'ret4';

                                            const child = findInReactTree(
                                                ret4,
                                                (x: ReactElement) =>
                                                    x?.props?.overview,
                                            );

                                            console.log('ret4 child', child);
                                            child.key = 'ret4_child';
                                            const p5 = afterPatch(
                                                child,
                                                'type',
                                                (_5, ret5) => {
                                                    console.log('ret5', ret5);
                                                    checkCachedArg('_5', _5);

                                                    ret5.key = 'ret5';

                                                    beforePatch(
                                                        ret5,
                                                        'type',
                                                        (args) => {
                                                            for (
                                                                let i = 0;
                                                                i < args.length;
                                                                i++
                                                            ) {
                                                                console.log(
                                                                    '_5 beforepatch @',
                                                                    i,
                                                                );
                                                                args[i] =
                                                                    checkCachedArg(
                                                                        `_5.${i}`,
                                                                        args[i],
                                                                    );
                                                            }
                                                            return ret5;
                                                        },
                                                    );

                                                    const p6 = afterPatch(
                                                        ret5,
                                                        'type',
                                                        (_6, ret6) => {
                                                            console.log(
                                                                'ret6',
                                                                ret6,
                                                            );
                                                            checkCachedArg(
                                                                '_6',
                                                                _6,
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

                                                            const p7 =
                                                                afterPatch(
                                                                    ret6Child.type,
                                                                    'render',
                                                                    (
                                                                        _7,
                                                                        ret7,
                                                                    ) => {
                                                                        console.log(
                                                                            'ret7',
                                                                            ret7,
                                                                        );
                                                                        checkCachedArg(
                                                                            '_7',
                                                                            _7,
                                                                        );

                                                                        ret7.key =
                                                                            'ret7';
                                                                        return ret7;
                                                                    },
                                                                );

                                                            console.log(
                                                                'p7',
                                                                p7,
                                                            );

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
                                                    console.log('p6', p6);

                                                    return ret5;
                                                },
                                            );
                                            console.log('p5', p5);

                                            return ret4;
                                        },
                                    );

                                    console.log('p4', p4);

                                    return ret3;
                                },
                            );
                            console.log('p3', p3);

                            return ret2;
                        },
                    );
                    console.log('p2', p2);

                    return ret;
                },
            );
            console.log('p1', p1);

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

// function pe(e) {
//     const t = (0, C.iE)(),
//         [r, n] = (0, ae.SP)('AppDetailsTabsActive', !1),
//         i = A.useRef(),
//         a = A.useRef(),
//         s = A.useRef(!0),
//         o = (0, u.q3)(
//             () =>
//                 B.TS.ON_DECK &&
//                 0 == oe.rV.storePreferences.provide_deck_feedback,
//         ),
//         l = (0, u.q3)(() =>
//             me.yX.BShouldPromptForDeckCompatibilityFeedback(e.overview.appid),
//         ),
//         c = A.useCallback(() => {
//             n(!1), t?.ScrollToTop();
//         }, [t, n]),
//         m = A.useCallback(() => {
//             a.current.FocusActionButton();
//         }, []),
//         d = A.useCallback(
//             (e) => {
//                 e && n(e);
//             },
//             [n],
//         );
//     return (
//         A.useEffect(() => {
//             const e = s.current;
//             s.current = !1;
//             let n = i.current;
//             if (!r || !t || !n) return;
//             const a = function (e) {
//                 let r =
//                     n.getBoundingClientRect().top +
//                     t.scrollTop -
//                     parseInt(O().headerPadding);
//                 t.ScrollTo(r, e);
//             };
//             e ? window.setTimeout(() => a('auto'), 1) : a('smooth');
//         }, [t, i, r]),
//         A.createElement(
//             _.Z,
//             {
//                 className: O().AppDetailsRoot,
//             },
//             A.createElement(be, {
//                 ...e,
//                 onNav: c,
//                 ref: a,
//             }),
//             A.createElement(Q.sD, {
//                 ...e,
//                 onFocus: c,
//             }),
//             A.createElement(
//                 _.Z,
//                 {
//                     onFocusWithin: c,
//                 },
//                 o && A.createElement(ge, null),
//                 !o &&
//                     l &&
//                     A.createElement(he, {
//                         ...e,
//                     }),
//             ),
//             A.createElement(
//                 _.Z,
//                 {
//                     ref: i,
//                     className: O().AppDetailsContainer,
//                     onFocusWithin: d,
//                 },
//                 A.createElement(_e, {
//                     fnOnCancelFromTabHeader: m,
//                     details: e.details,
//                     overview: e.overview,
//                     setSections: e.setSections,
//                     bSuppressTransition: e.bSuppressTransition,
//                     parentComponent: e.parentComponent,
//                 }),
//             ),
//         )
//     );
// }
