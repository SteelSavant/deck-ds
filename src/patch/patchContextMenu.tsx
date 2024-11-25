/* eslint-disable @typescript-eslint/no-explicit-any */

import {
    afterPatch,
    fakeRenderComponent,
    findModuleChild,
    Patch,
} from '@decky/ui';
import { ReactElement, useEffect, useRef, useState } from 'react';
import { IconForTarget } from '../components/IconForTarget';
import {
    ShortAppDetailsState,
    ShortAppDetailsStateContextProvider,
    useAppState,
} from '../context/appContext';
import { logger } from '../util/log';
import { isSteamGame } from '../util/util';
import useActionButtonProps from './hooks/useActionButtonProps';

const appDetailsState = new ShortAppDetailsState();

function PlayBtnMenuItem({
    appId,
    playButton,
}: {
    appId: number;
    playButton: any;
}): ReactElement {
    console.log('building play btn menu item');
    const { appDetails } = useAppState();
    const { target, onLaunch } = useActionButtonProps({
        isPrimary: true,
    });
    const [patch, setPatch] = useState(!!(onLaunch && target)); // hack to force rerenders when necessary
    const ref = useRef<any>();

    // Store the original button onclick/icon
    const buttonRef = useRef(playButton.props.children[0]);
    const launchRef = useRef(playButton.props.onSelected);
    const keyRef = useRef(playButton.key);
    const refRef = useRef(playButton.ref);

    useEffect(() => {
        // Ensure we're set to the right page
        if (appDetails?.appId !== appId) {
            const overview = appStore.GetAppOverviewByAppID(appId);

            appDetailsState.setOnAppPage({
                appId: appId,
                gameId: overview.m_gameid,
                sortAs: overview.sort_as,
                userId64: App.m_CurrentUser.strSteamID,
                isSteamGame: isSteamGame(overview),
                selected_clientid: overview.selected_clientid,
            });
        }
    }, [appDetails?.appId, appId]);

    useEffect(() => {
        const children = playButton.props.children as any[];
        const shouldPatch = !!(target && onLaunch);

        console.log(
            'playbutton',
            playButton,
            'shouldPatch',
            shouldPatch,
            'target',
            target,
            'onLaunch',
            onLaunch,
        );

        if (shouldPatch) {
            logger.trace('Using play target');
            children[0] = <IconForTarget target={target} />;
            playButton.props.onSelected = onLaunch;
            playButton.ref = ref;
        } else {
            logger.trace('Using play original');
            children[0] = buttonRef.current;
            playButton.props.onSelected = launchRef.current;
            playButton.ref = refRef.current;
        }

        if (patch !== shouldPatch) {
            logger.trace('forcing primary play button rebuild...');
            playButton.key = shouldPatch
                ? 'deckds-ctx-play-btn'
                : keyRef.current;
            setPatch(shouldPatch);

            setTimeout(() => {
                ref.current?.Focus();
            }, 1);
        }

        return () => {
            children[0] = buttonRef.current;
            playButton.props.onSelected = launchRef.current;
            playButton.key = keyRef.current;
        };
    }, [target, onLaunch]);

    return playButton;
}

const splicePlayButton = (children: any[], appid: number) => {
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
                <PlayBtnMenuItem appId={appid} playButton={children[0]} />
            </ShortAppDetailsStateContextProvider>,
        );
    }
};

/**
 * Patches the game context menu.
 * @param LibraryContextMenu The game context menu.
 * @returns A patch to remove when the plugin dismounts.
 */
const contextMenuPatch = (LibraryContextMenu: any) => {
    const patches: {
        outer?: Patch;
        inner?: Patch;
        unpatch: () => void;
    } = {
        unpatch: () => {
            return null;
        },
    };
    console.log('patching context menu', patches);
    patches.outer = afterPatch(
        LibraryContextMenu.prototype,
        'render',
        (args: Record<string, unknown>[], component: any) => {
            const appid: number = component._owner.pendingProps.overview.appid;
            console.log('patching context menu afterpatch', patches, args);

            if (!patches.inner) {
                console.log('no inner patch');

                patches.inner = afterPatch(
                    component.type.prototype,
                    'shouldComponentUpdate',
                    ([nextProps]: any, shouldUpdate: any) => {
                        console.log('patching context menu afterpatch 2');

                        try {
                            const ddsIdx = nextProps.children.findIndex(
                                (x: any) => x?.key === 'deckds-ctx-play-btn',
                            );
                            if (ddsIdx != -1)
                                nextProps.children.splice(ddsIdx, 1);
                        } catch (e) {
                            console.log('wrong component?');
                            // wrong context menu (probably)
                            return component;
                        }

                        console.log(
                            'actually patching context menu afterpatch 2',
                        );

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
                            console.log('definitely splice');
                            splicePlayButton(nextProps.children, updatedAppid);
                        } else {
                            console.log('dont splice?');
                        }

                        return shouldUpdate;
                    },
                );
            }

            splicePlayButton(component.props.children, appid);
            console.log('returning component', component);

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
                console.log('returning fake obj');
                return Object.values(m).find(
                    (sibling) =>
                        sibling?.toString().includes('createElement') &&
                        sibling?.toString().includes('navigator:'),
                );
            }
        }
        console.log('not returning fake obj');
        return;
    }),
).type;

export default contextMenuPatch;
