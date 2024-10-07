import { routerHook } from '@decky/api';
import {
    afterPatch,
    appDetailsClasses,
    basicAppDetailsSectionStylerClasses,
    findInReactTree,
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

function patchLibraryApp(route: string, appDetailsState: ShortAppDetailsState) {
    // debugPrintStyles();

    return routerHook.addPatch(
        route,
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

                            const appId = overview.appid;

                            const children = container.props.children;
                            const child = children.find(
                                (c: any) => c?.type?.render,
                            );

                            console.log('ret2', ret2);

                            // wrapReactType(child.type);
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

                                    const overview =
                                        appStore.GetAppOverviewByAppID(appId);

                                    const status =
                                        overview.per_client_data.find(
                                            (d: any) =>
                                                d.clientid ===
                                                overview.selected_clientid,
                                        );

                                    const streaming = status.clientid !== '0';
                                    const installed: boolean =
                                        !streaming &&
                                        status.status_percentage == 100 &&
                                        status.installed;

                                    const appButtons = findInReactTree(
                                        ret3,
                                        (x: ReactElement) =>
                                            Array.isArray(x?.props?.children) &&
                                            x?.props?.className?.includes(
                                                basicAppDetailsSectionStylerClasses.AppButtons,
                                            ),
                                    );

                                    // const playButtonStatusPanel = findInReactTree(
                                    //     ret3,
                                    //     (x: ReactElement) => Array.isArray(x?.props?.children) &&
                                    //         x?.props?.className?.includes(
                                    //             basicAppDetailsSectionStylerClasses.ActionButtonAndStatusPanel
                                    //         )

                                    // )

                                    const playButton = findInReactTree(
                                        ret3,
                                        (x: ReactElement) => {
                                            return (
                                                Array.isArray(
                                                    x?.props?.children,
                                                ) &&
                                                x?.props?.className?.includes(
                                                    basicAppDetailsSectionStylerClasses.AppActionButton,
                                                )
                                            );
                                        },
                                    );
                                    const missingAppButtons =
                                        typeof appButtons !== 'object';
                                    const missingPlayButton =
                                        typeof playButton !== 'object';

                                    if (
                                        !missingAppButtons ||
                                        !missingPlayButton
                                    ) {
                                        console.log('ret3', ret3);
                                    }

                                    if (!missingAppButtons) {
                                        console.log('appButtons', appButtons);

                                        const children =
                                            appButtons?.props?.children;

                                        if (
                                            installed &&
                                            children &&
                                            !children.find(
                                                (c: any) =>
                                                    c?.props?.children?.props
                                                        ?.deckDSDesktopSentinel ===
                                                    'sentinel',
                                            )
                                        ) {
                                            console.log(
                                                'appbutton children',
                                                children,
                                            );

                                            children.splice(
                                                0,
                                                0,
                                                <ShortAppDetailsStateContextProvider
                                                    ShortAppDetailsStateClass={
                                                        appDetailsState
                                                    }
                                                >
                                                    <SecondaryPlayButton deckDSDesktopSentinel="sentinel" />
                                                </ShortAppDetailsStateContextProvider>,
                                            );
                                        } else {
                                            const sentinel = children.findIndex(
                                                (c: any) =>
                                                    c?.props?.children?.props
                                                        ?.deckDSDesktopSentinel ===
                                                    'sentinel',
                                            );
                                            console.log(
                                                'appbutton sentinel index: ',
                                                sentinel,
                                                'in',
                                                children,
                                            );
                                            console.log(
                                                'appbutton overview',
                                                overview.selected_clientid,
                                                'status:',
                                                status,
                                                'installed:',
                                                installed,
                                            );

                                            console.log(
                                                'appbutton alt overview',
                                                children.slice(-1)[0]?.props
                                                    ?.overview
                                                    ?.selected_clientid,
                                            );

                                            if (!installed && sentinel >= 0) {
                                                console.log(
                                                    'splicing out appbutton',
                                                );

                                                children.splice(sentinel, 1);
                                            }
                                        }
                                    }

                                    if (!missingPlayButton) {
                                        const children =
                                            playButton?.props?.children;

                                        if (
                                            installed &&
                                            children &&
                                            !children.find(
                                                (c: any) =>
                                                    c?.props?.children?.props
                                                        ?.deckDSGameModeSentinel ===
                                                    'sentinel',
                                            )
                                        ) {
                                            console.log(
                                                'playbutton children',
                                                children,
                                            );

                                            const actualPlayButton =
                                                children[0];
                                            cachedPlayButton = actualPlayButton;

                                            console.log(
                                                'playButton',
                                                playButton,
                                            );
                                            console.log(
                                                'actualPlayButton',
                                                actualPlayButton,
                                            );

                                            children.splice(
                                                0,
                                                1,
                                                <ShortAppDetailsStateContextProvider
                                                    ShortAppDetailsStateClass={
                                                        appDetailsState
                                                    }
                                                >
                                                    <PrimaryPlayButton
                                                        playButton={
                                                            actualPlayButton
                                                        }
                                                        deckDSGameModeSentinel="sentinel"
                                                    />
                                                </ShortAppDetailsStateContextProvider>,
                                            );
                                        } else {
                                            const sentinel = children.findIndex(
                                                (c: any) =>
                                                    c?.props?.children?.props
                                                        ?.deckDSGameModeSentinel ===
                                                    'sentinel',
                                            );
                                            console.log(
                                                'playbutton sentinel index: ',
                                                sentinel,
                                                'in',
                                                children,
                                            );

                                            console.log(
                                                'playbutton overview',
                                                overview.selected_clientid,
                                                'status:',
                                                status,
                                                'installed:',
                                                installed,
                                            );

                                            if (
                                                sentinel >= 0 &&
                                                !installed &&
                                                cachedPlayButton
                                            ) {
                                                children.splice(
                                                    sentinel,
                                                    1,
                                                    cachedPlayButton,
                                                );
                                            }
                                        }
                                    }

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

export default patchLibraryApp;
