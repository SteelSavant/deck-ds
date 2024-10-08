/*
Useful Resources:
- Steam Browser Protocol: https://developer.valvesoftware.com/wiki/Steam_browser_protocol
- Steam Console Commands: https://gist.github.com/davispuh/6600880
*/

import {
    DialogButton,
    findModuleChild,
    Focusable,
    PanelSection,
    quickAccessMenuClasses,
    Router,
} from '@decky/ui';
import { VFC } from 'react';

import { definePlugin, routerHook } from '@decky/api';
import { FaGears, FaWaveSquare } from 'react-icons/fa6';
import * as backend from './backend';
import { IconForTarget } from './components/IconForTarget';
import {
    ShortAppDetailsState,
    ShortAppDetailsStateContextProvider,
} from './context/appContext';
import { PatchHandler } from './patch/patchHandler';
import { teardownClientPipeline } from './pipeline/client_pipeline';
import { logger, LogLevel } from './util/log';
import QAM from './views/QAM';
import ProfileRoute from './views/Settings/Profiles/ProfileRoute';
import SettingsRouter from './views/Settings/SettingsRouter';

declare global {
    let collectionStore: CollectionStore;
    let appStore: AppStore;
    let appDetailsStore: AppDetailsStore;
    let App: App;
}

const appDetailsState = new ShortAppDetailsState();
PatchHandler.init(appDetailsState);

let usdplReady = false;

(async function () {
    // Init backend
    await backend.initBackend();
    // Run cleanup for any previous run
    await teardownClientPipeline();
    // Setup patch handler
    const globalSettings = await backend.getSettings();
    if (globalSettings.isOk) {
        PatchHandler.getInstance().setPatchEnabled(
            globalSettings.data.global_settings.enable_ui_inject,
        );
    }

    usdplReady = true;
})();

const Content: VFC = () => {
    if (!usdplReady) {
        // Not translated on purpose (to avoid USDPL issues)
        return (
            <PanelSection>
                USDPL or DeckDS's backend did not start correctly!
            </PanelSection>
        );
    }

    return (
        <ShortAppDetailsStateContextProvider
            ShortAppDetailsStateClass={appDetailsState}
        >
            <QAM />
        </ShortAppDetailsStateContextProvider>
    );
};

const History = findModuleChild((m) => {
    if (typeof m !== 'object') return undefined;
    for (let prop in m) {
        if (m[prop]?.m_history) return m[prop].m_history;
    }
});

export default definePlugin(() => {
    // console.log('Steam Client:', SteamClient);
    // console.log('collection store:', collectionStore);
    // console.log('collections:', collectionStore.userCollections);

    // Ensure patch handler settings loaded
    let loaded = true;
    setTimeout(async () => {
        if (!loaded || !usdplReady) {
            logger.warn('Not setting patch setting; not ready.');
            return;
        }

        const globalSettings = await backend.getSettings();

        if (globalSettings.isOk && loaded) {
            PatchHandler.getInstance().setPatchEnabled(
                globalSettings.data.global_settings.enable_ui_inject,
            );
        } else if (!globalSettings.isOk) {
            logger.error('Not setting patch setting: ', globalSettings.err);
        }
    }, 1000); // We defer until after the backend should be initialized to avoid potential issues.

    function isSteamGame(overview: any): boolean {
        const hasOwnerAccountId = overview.owner_account_id !== undefined;
        const wasPurchased = !!overview.rt_purchased_time;
        const hasSize = overview.size_on_disk !== '0';

        return hasOwnerAccountId || wasPurchased || hasSize;
    }

    function updateAppDetails(this: any, currentRoute: string): void {
        const re = /^\/library\/app\/(\d+)(\/?.*)/;

        if (re.test(currentRoute)) {
            const appIdStr = re.exec(currentRoute)![1]!;
            const appId = Number.parseInt(appIdStr);
            const overview = appStore.GetAppOverviewByAppID(appId);

            // console.log('steam client app overview:', overview);
            // console.log(
            //     'steam client app details',
            //     appDetailsStore.GetAppDetails(appId),
            // );

            appDetailsState.setOnAppPage({
                appId,
                gameId: overview.m_gameid,
                sortAs: overview.sort_as,
                userId64: App.m_CurrentUser.strSteamID,
                isSteamGame: isSteamGame(overview),
                selected_clientid: overview.selected_clientid,
            });
        } else {
            appDetailsState.setOnAppPage(null);
            appDetailsState.setOnAppPage(null);
        }
    }

    const initialRoute = History.location?.pathname ?? '/library/home';
    updateAppDetails(initialRoute);

    const unlistenHistory = History.listen(async (info: any) => {
        updateAppDetails(info.pathname);
    });

    // Profiles route
    routerHook.addRoute(
        '/deck-ds/settings/profiles/:profileid',
        () => (
            <ShortAppDetailsStateContextProvider
                ShortAppDetailsStateClass={appDetailsState}
            >
                <ProfileRoute />
            </ShortAppDetailsStateContextProvider>
        ),
        {
            exact: true,
        },
    );

    // Settings Route
    routerHook.addRoute(
        '/deck-ds/settings/:setting',
        () => (
            <ShortAppDetailsStateContextProvider
                ShortAppDetailsStateClass={appDetailsState}
            >
                <SettingsRouter />
            </ShortAppDetailsStateContextProvider>
        ),
        {
            exact: true,
        },
    );

    const navigateToSettings = () => {
        Router.CloseSideMenus();
        Router.Navigate('/deck-ds/settings/profiles');
    };

    let appStateRegistrar: any;
    try {
        appStateRegistrar =
            SteamClient.GameSessions.RegisterForAppLifetimeNotifications(
                async (update: any) => {
                    console.log('app lifecycle update:', update);
                    if (!update.bRunning) {
                        await teardownClientPipeline(update.unAppID);
                    }
                },
            );
    } catch (ex) {
        logger.error('failed to register for app lifetime notifications:', ex);
    }

    return {
        name: 'DeckDS',
        alwaysRender: true,
        titleView: (
            <Focusable
                style={{
                    display: 'flex',
                    padding: '0',
                    width: '100%',
                    boxShadow: 'none',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                }}
                className={quickAccessMenuClasses.Title}
            >
                <p>DeckDS</p>
                <DialogButton
                    style={{
                        width: 'fit-content',
                        minWidth: 'fit-content',
                        height: 'fit-content',
                        minHeight: 'fit-content',
                        paddingLeft: 10,
                        paddingRight: 10,
                        paddingTop: 5,
                        paddingBottom: 5,
                    }}
                    onClick={navigateToSettings}
                    onOKButton={navigateToSettings}
                >
                    <FaGears />
                </DialogButton>
            </Focusable>
        ),
        icon: (
            <div>
                <IconForTarget target="Desktop" />
                <FaWaveSquare />
                <IconForTarget target="Gamemode" />
            </div>
        ),
        content: <Content />,

        onDismount: () => {
            loaded = false;

            backend.log(LogLevel.Debug, 'DeckDS shutting down');

            unlistenHistory();
            appDetailsState.setOnAppPage(null);

            PatchHandler.dispose();
            routerHook.removeRoute('/deck-ds/settings/templates/:templateid');
            routerHook.removeRoute('/deck-ds/settings/:setting');

            appStateRegistrar?.unregister();
        },
    };
});
