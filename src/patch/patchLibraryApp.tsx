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

            afterPatch(
                props.children.props,
                'renderFunc',
                (_1: Record<string, unknown>[], ret?: ReactElement) => {
                    if (!ret?.props?.children?.type?.type) {
                        return ret;
                    }

                    wrapReactType(ret.props.children);
                    afterPatch(
                        ret.props.children.type,
                        'type',
                        (
                            _2: Record<string, unknown>[],
                            ret2?: ReactElement,
                        ) => {
                            let hasWrappedRet6Child = false;
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

                            const children = container.props.children;
                            const child = children.find((c: any) =>
                                c?.props?.className?.includes(
                                    appDetailsClasses.AppDetailsOverviewPanel,
                                ),
                            );

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

                                    ret3.key = 'ret3';

                                    const child = findInReactTree(
                                        ret3,
                                        (x: ReactElement) => x?.props?.overview,
                                    );

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

                                                            if (
                                                                !ret6Child ||
                                                                !installed
                                                            ) {
                                                                return ret6;
                                                            }

                                                            hasWrappedRet6Child =
                                                                true;

                                                            ret6Child.key =
                                                                'ret6child';

                                                            wrapReactType(
                                                                ret6Child,
                                                            );

                                                            beforePatch(
                                                                ret6Child.type,
                                                                'render',
                                                                (args) => {
                                                                    console.log(
                                                                        'ret6child args',
                                                                        args,
                                                                    );

                                                                    const onNav =
                                                                        args[0]
                                                                            .onNav;
                                                                    args[0].onNav =
                                                                        (
                                                                            ...args: any
                                                                        ) => {
                                                                            if (
                                                                                !installed ||
                                                                                ret6incr ===
                                                                                    0
                                                                            ) {
                                                                                console.log(
                                                                                    'calling onNav',
                                                                                    ret6incr,
                                                                                );
                                                                                onNav(
                                                                                    ...args,
                                                                                );
                                                                            } else {
                                                                                console.log(
                                                                                    'calling onNav debounce',
                                                                                    ret6incr,
                                                                                );
                                                                            }

                                                                            ret6incr += 1;
                                                                            ret6incr %= 3;
                                                                        };
                                                                },
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
