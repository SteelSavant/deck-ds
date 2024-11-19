import { routerHook } from '@decky/api';
import {
    afterPatch,
    appDetailsClasses,
    basicAppDetailsSectionStylerClasses,
    beforePatch,
    callOriginal,
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
import { isSteamGame } from '../util/util';
import PrimaryPlayButton from './components/PrimaryPlayButton';
import SecondaryPlayButton from './components/SecondaryPlayButton';

const onNavDebounceTime = 500;
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
                (_1: Record<string, unknown>[], ret?: ReactElement) => {
                    if (!ret?.props?.children?.type?.type) {
                        return ret;
                    }

                    console.log('ret', ret);
                    let lastOnNavTime = 0;
                    let onNavIncr = 0;
                    let appDetailsFalseCount = 0;

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

                                    const child = findInReactTree(
                                        ret3,
                                        (x: ReactElement) => x?.props?.overview,
                                    );

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

                                                    afterPatch(
                                                        ret5,
                                                        'type',
                                                        (_6, ret6) => {
                                                            console.log(
                                                                'ret6',
                                                                ret6,
                                                            );

                                                            ret6.key = 'ret6';
                                                            lastOnNavTime =
                                                                Date.now(); // prevents nav when rebuilding

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

                                                            playSection.key =
                                                                'ret6child';

                                                            for (const v of [
                                                                // [0, 'onNav'],
                                                                // [1, 'onFocus'],
                                                                // [
                                                                //     2,
                                                                //     'onFocusWithin',
                                                                // ],
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

                                                                wrapReactType(
                                                                    child,
                                                                    'props',
                                                                );
                                                                replacePatch(
                                                                    child.props,
                                                                    v[1] as string,
                                                                    (
                                                                        focusArgs,
                                                                    ) => {
                                                                        console.log(
                                                                            'ret6 focuswithin',
                                                                            v[0],
                                                                            v[1],
                                                                            focusArgs,
                                                                            onFocusWithin,
                                                                        );

                                                                        // console.log(
                                                                        //     'handling focus within...',
                                                                        // );

                                                                        if (
                                                                            v[0] ===
                                                                            3
                                                                        ) {
                                                                            if (
                                                                                !focusArgs[0]
                                                                            ) {
                                                                                appDetailsFalseCount += 1;
                                                                                if (
                                                                                    appDetailsFalseCount >
                                                                                    1
                                                                                ) {
                                                                                    console.log(
                                                                                        'calling onnav from appdetailssection focuswithin',
                                                                                    );

                                                                                    playSection.props.onNav();
                                                                                    playSection.props.onNav();

                                                                                    lastOnNavTime =
                                                                                        Date.now();
                                                                                } else {
                                                                                    console.log(
                                                                                        'setting appdetailssection focuswithin true',
                                                                                    );
                                                                                    focusArgs[0] =
                                                                                        true;
                                                                                }
                                                                                appDetailsFalseCount %= 2;
                                                                            }
                                                                        }

                                                                        return callOriginal;
                                                                    },
                                                                );
                                                            }

                                                            wrapReactType(
                                                                playSection,
                                                            );
                                                            wrapReactType(
                                                                playSection,
                                                                'props',
                                                            );
                                                            replacePatch(
                                                                playSection.props,
                                                                'onNav',
                                                                (_args) => {
                                                                    console.log(
                                                                        'ret6child onnav',
                                                                        _args,
                                                                    );

                                                                    const elapsed =
                                                                        Date.now() -
                                                                        lastOnNavTime;

                                                                    if (
                                                                        elapsed <
                                                                            onNavDebounceTime ||
                                                                        appDetailsFalseCount ===
                                                                            1
                                                                    ) {
                                                                        console.log(
                                                                            'calling onNav debounce elapsed, false==',
                                                                            appDetailsFalseCount,
                                                                        );
                                                                        return;
                                                                    }

                                                                    if (
                                                                        onNavIncr ===
                                                                        0
                                                                    ) {
                                                                        console.log(
                                                                            'calling onNav',
                                                                            onNavIncr,
                                                                        );

                                                                        return callOriginal;
                                                                    } else {
                                                                        console.log(
                                                                            'calling onNav debounce build',
                                                                            onNavIncr,
                                                                        );
                                                                    }

                                                                    // onNavIncr += 1;
                                                                    // onNavIncr %=
                                                                    //     onNavMaxIncr;

                                                                    // console.log(
                                                                    //     'set onnav incr to',
                                                                    //     onNavIncr,
                                                                    // );

                                                                    return;
                                                                },
                                                            );

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

                                                                    if (
                                                                        appDetailsFalseCount >
                                                                        1
                                                                    ) {
                                                                        console.log(
                                                                            'calling onnav from rebuild',
                                                                        );

                                                                        playSection.props.onNav();
                                                                        playSection.props.onNav();

                                                                        lastOnNavTime =
                                                                            Date.now();
                                                                    }

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
            // const ref = retPlayButton.ref;
            // if (ref) {
            //     ref.current = null;

            //     setTimeout(() => {
            //         ref.current = null;
            //     }, 100);
            // }

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

                            wrapReactType(actualPlayButton);
                            beforePatch(
                                actualPlayButton.type,
                                'render',
                                (args) => {
                                    args[0].autoFocus = false;
                                    args[1] = null;
                                },
                            );

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
