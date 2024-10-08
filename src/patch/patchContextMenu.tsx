/* eslint-disable @typescript-eslint/no-explicit-any */

import {
    afterPatch,
    fakeRenderComponent,
    findModuleChild,
    Patch,
} from '@decky/ui';
import { ReactElement } from 'react';
import { IconForTarget } from '../components/IconForTarget';
import {
    ShortAppDetailsState,
    ShortAppDetailsStateContextProvider,
    useAppState,
} from '../context/appContext';
import useAppTarget from '../hooks/useAppTarget';
import useLaunchActions from '../hooks/useLaunchActions';
import { isSteamGame } from '../util/util';

function PlayBtnMenuItem({
    playButton,
}: {
    playButton: ReactElement;
}): ReactElement {
    const { appDetails, appProfile } = useAppState();
    const launchActions = useLaunchActions(appDetails);

    const action = appProfile?.isOk
        ? launchActions.find(
              (a) => a.profile.id == appProfile.data.default_profile,
          ) ?? launchActions[0]
        : null;

    const target = useAppTarget({
        isPrimary: true,
        profileId: action?.profile.id ?? null,
    });

    const onLaunch = action?.targets?.find((t) => t.target === target)?.action;

    const shouldCustomize = target && onLaunch;

    console.log('playbutton', playButton);

    if (!shouldCustomize) {
        return playButton;
    }

    const modifiedPlayButton = {
        ...playButton,
    };

    modifiedPlayButton.props = { ...playButton.props };
    modifiedPlayButton.props.children = [
        IconForTarget({ target }),
        playButton.props.children[1],
    ];
    modifiedPlayButton.props.onSelected = onLaunch;
    modifiedPlayButton.key = 'deckds-play-btn';

    console.log('modified playbutton', modifiedPlayButton);

    return modifiedPlayButton;
}

const splicePlayButton = (
    children: any[],
    appid: number,
    appDetailsState: ShortAppDetailsState,
) => {
    console.log('ctx children', children);
    const overview = appStore.GetAppOverviewByAppID(appid);

    const status = overview.per_client_data.find(
        (d: any) => d.clientid === overview.selected_clientid,
    );

    const streaming = status.clientid !== '0';
    const installed: boolean =
        !streaming && status.status_percentage == 100 && status.installed;

    const missingPlayBtn = !children[0] || streaming || !installed;

    console.log(
        'overview',
        overview,
        'streaming',
        streaming,
        'installed',
        installed,
        'status',
        status,
    );

    if (!missingPlayBtn && !children[0].props.ShortAppDetailsStateClass) {
        children.splice(
            0,
            1,
            <ShortAppDetailsStateContextProvider
                ShortAppDetailsStateClass={appDetailsState}
            >
                <PlayBtnMenuItem playButton={children[0]} />
            </ShortAppDetailsStateContextProvider>,
        );
    }
};

/**
 * Patches the game context menu.
 * @param LibraryContextMenu The game context menu.
 * @returns A patch to remove when the plugin dismounts.
 */
const contextMenuPatch = (
    LibraryContextMenu: any,
    appDetailsState: ShortAppDetailsState,
) => {
    const patches: {
        outer?: Patch;
        inner?: Patch;
        unpatch: () => void;
    } = {
        unpatch: () => {
            return null;
        },
    };
    patches.outer = afterPatch(
        LibraryContextMenu.prototype,
        'render',
        (_: Record<string, unknown>[], component: any) => {
            const appid: number = component._owner.pendingProps.overview.appid;
            const overview = appStore.GetAppOverviewByAppID(appid);

            appDetailsState.setOnAppPage({
                appId: appid,
                gameId: overview.m_gameid,
                sortAs: overview.sort_as,
                userId64: App.m_CurrentUser.strSteamID,
                isSteamGame: isSteamGame(overview),
                selected_clientid: overview.selected_clientid,
            });

            if (!patches.inner) {
                patches.inner = afterPatch(
                    component.type.prototype,
                    'shouldComponentUpdate',
                    ([nextProps]: any, shouldUpdate: any) => {
                        try {
                            const ddsIdx = nextProps.children.findIndex(
                                (x: any) => x?.key === 'deckds-play-btn',
                            );
                            if (ddsIdx != -1)
                                nextProps.children.splice(ddsIdx, 1);
                        } catch (e) {
                            // wrong context menu (probably)
                            return component;
                        }

                        if (shouldUpdate === true) {
                            let updatedAppid: number = appid;

                            // find the first menu component that has the correct appid assigned to _owner
                            const parentOverview = nextProps.children.find(
                                (x: any) =>
                                    x?._owner?.pendingProps?.overview?.appid &&
                                    x._owner.pendingProps.overview.appid !==
                                        appid,
                            );

                            // if found then use that appid
                            if (parentOverview) {
                                updatedAppid =
                                    parentOverview._owner.pendingProps.overview
                                        .appid;
                            }

                            const overview =
                                appStore.GetAppOverviewByAppID(appid);
                            appDetailsState.setOnAppPage({
                                appId: updatedAppid,
                                gameId: overview.m_gameid,
                                sortAs: overview.sort_as,
                                userId64: App.m_CurrentUser.strSteamID,
                                isSteamGame: isSteamGame(overview),
                                selected_clientid: overview.selected_clientid,
                            });

                            splicePlayButton(
                                nextProps.children,
                                updatedAppid,
                                appDetailsState,
                            );
                        }

                        return shouldUpdate;
                    },
                );
            } else {
                splicePlayButton(
                    component.props.children,
                    appid,
                    appDetailsState,
                );
            }

            return component;
        },
    );
    patches.unpatch = () => {
        patches.outer?.unpatch();
        patches.inner?.unpatch();
    };
    return patches;
};

/**
 * Game context menu component.
 */
export const LibraryContextMenu = fakeRenderComponent(
    findModuleChild((m) => {
        if (typeof m !== 'object') return;
        for (const prop in m) {
            if (
                m[prop]?.toString() &&
                m[prop].toString().includes('().LibraryContextMenu')
            ) {
                return Object.values(m).find(
                    (sibling) =>
                        sibling?.toString().includes('createElement') &&
                        sibling?.toString().includes('navigator:'),
                );
            }
        }
        return;
    }),
).type;

export default contextMenuPatch;
