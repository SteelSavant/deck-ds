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
import { logger } from '../util/log';
import { isSteamGame } from '../util/util';
import PrimaryPlayButton from './components/PrimaryPlayButton';
import SecondaryPlayButton from './components/SecondaryPlayButton';

let cachedPlayButton: ReactElement | null = null;

function getOnNavDebounceTime(isNonSteamGame: boolean): number {
    return 200;
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

                    let lastOnNavTime = 0;
                    let lastEnterAppDetailsTime = 0;
                    let appDetailsFalseCount = 1;

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

                            if (typeof container !== 'object') {
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

                                            const child = findInReactTree(
                                                ret4,
                                                (x: ReactElement) =>
                                                    x?.props?.overview,
                                            );

                                            afterPatch(
                                                child,
                                                'type',
                                                (_5, ret5) => {
                                                    afterPatch(
                                                        ret5,
                                                        'type',
                                                        (_6, ret6) => {
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

                                                            const overview =
                                                                playSection
                                                                    ?.props
                                                                    ?.overview;

                                                            const isNonSteamGame =
                                                                overview.app_type ==
                                                                1073741824;

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
                                                                return ret6;
                                                            }

                                                            const onFocusWithin =
                                                                appDetailsSection
                                                                    .props
                                                                    .onFocusWithin;
                                                            wrapReactType(
                                                                appDetailsSection,
                                                                'props',
                                                            );
                                                            replacePatch(
                                                                appDetailsSection.props,
                                                                'onFocusWithin',
                                                                (focusArgs) => {
                                                                    logger.trace(
                                                                        'appDetailsSection focuswithin',

                                                                        focusArgs.toString(),
                                                                        focusArgs,
                                                                    );

                                                                    const elapsedOnNav =
                                                                        Date.now() -
                                                                        lastOnNavTime;
                                                                    const elapsedAppDetails =
                                                                        Date.now() -
                                                                        lastEnterAppDetailsTime;

                                                                    if (
                                                                        elapsedAppDetails <
                                                                            250 ||
                                                                        elapsedOnNav <
                                                                            getOnNavDebounceTime(
                                                                                isNonSteamGame,
                                                                            )
                                                                    ) {
                                                                        logger.trace(
                                                                            'skipping app details false ==',
                                                                            appDetailsFalseCount,
                                                                        );
                                                                        focusArgs[0] =
                                                                            true;
                                                                        onFocusWithin(
                                                                            focusArgs,
                                                                        );
                                                                    }

                                                                    if (
                                                                        !focusArgs[0]
                                                                    ) {
                                                                        appDetailsFalseCount += 1;
                                                                    } else {
                                                                        appDetailsFalseCount = 0;
                                                                        lastEnterAppDetailsTime =
                                                                            Date.now();
                                                                    }

                                                                    return callOriginal;
                                                                },
                                                            );
                                                            replacePatch(
                                                                appDetailsSection
                                                                    .props
                                                                    .children
                                                                    .props,
                                                                'fnOnCancelFromTabHeader',
                                                                (_args) => {
                                                                    lastEnterAppDetailsTime = 0;
                                                                    lastOnNavTime = 0;
                                                                    appDetailsFalseCount = 1;
                                                                    setTimeout(
                                                                        playSection
                                                                            .props
                                                                            .onNav,
                                                                        1,
                                                                    );

                                                                    return callOriginal;
                                                                },
                                                            );

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
                                                                    logger.trace(
                                                                        'ret6child onnav',
                                                                        _args,
                                                                    );

                                                                    const elapsed =
                                                                        Date.now() -
                                                                        lastOnNavTime;

                                                                    if (
                                                                        elapsed <
                                                                            getOnNavDebounceTime(
                                                                                isNonSteamGame,
                                                                            ) ||
                                                                        appDetailsFalseCount ===
                                                                            0
                                                                    ) {
                                                                        logger.trace(
                                                                            'calling onNav debounce elapsed, false==',
                                                                            appDetailsFalseCount,
                                                                        );
                                                                        return;
                                                                    }

                                                                    lastOnNavTime =
                                                                        Date.now();

                                                                    return callOriginal;
                                                                },
                                                            );

                                                            afterPatch(
                                                                playSection.type,
                                                                'render',
                                                                (_7, ret7) => {
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
                                                                            const elapsedAppDetails =
                                                                                Date.now() -
                                                                                lastEnterAppDetailsTime;

                                                                            const shouldAutoFocus =
                                                                                elapsedAppDetails >
                                                                                    250 &&
                                                                                appDetailsFalseCount >
                                                                                    0;
                                                                            patchFinalElement(
                                                                                ret8,
                                                                                overview,
                                                                                appDetailsState,
                                                                                {
                                                                                    shouldAutoFocus,
                                                                                },
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
    flags: {
        shouldAutoFocus: boolean;
    },
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

    logger.trace('appbuttons', appButtons);
    logger.trace('playbutton', playButton);

    if (!missingAppButtons && flags.shouldAutoFocus) {
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
            logger.trace('retPlayButton', retPlayButton);

            wrapReactClass(retPlayButton);
            afterPatch(
                retPlayButton.type.prototype,
                'render',
                (_playClass, classPlayButton) => {
                    logger.trace('classPlayButton', classPlayButton);

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
                                    args[0].autoFocus = flags.shouldAutoFocus;
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
