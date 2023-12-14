import {
  ButtonItem,
  definePlugin,

  PanelSection,
  ServerAPI,
  staticClasses
} from "decky-frontend-lib";
import { VFC } from "react";
import { FaShip } from "react-icons/fa";

import * as backend from "./backend";
import { ServerApiProvider } from "./context/serverApiContext";
import { ShortAppDetailsState, ShortAppDetailsStateContextProvider } from "./context/shortAppDetailsContext";
import patchLibraryApp from "./lib/patchLibraryApp";
import QAM from "./views/QAM";
import ProfileRoute from "./views/Settings/Profiles/ProfileRoute";
import SettingsRouter from "./views/Settings/SettingsRouter";
import TemplatePreviewRoute from "./views/Settings/Templates/TemplatePreviewRoute";

const appDetailsState = new ShortAppDetailsState();

var usdplReady = false;


(async function () {
  await backend.initBackend();
  usdplReady = true;
})()

const Content: VFC<{ serverApi: ServerAPI }> = ({ serverApi }) => {
  if (!usdplReady) {
    // Not translated on purpose (to avoid USDPL issues)
    return (

      <PanelSection>
        USDPL or DeckDS's backend did not start correctly!
        <ButtonItem
          layout="below"
          onClick={(_: MouseEvent) => {
            console.log("DeckDS: manual reload after startup failure");
            // reload();
          }}
        >
          Reload
        </ButtonItem>
      </PanelSection>
    )
  }

  return (
    <ShortAppDetailsStateContextProvider ShortAppDetailsStateClass={appDetailsState}>
      <QAM />
    </ShortAppDetailsStateContextProvider>

  );
}

export default definePlugin((serverApi: ServerAPI) => {
  const libraryPatch = patchLibraryApp(serverApi, appDetailsState);

  // Template Preview Route
  serverApi.routerHook.addRoute("/deck-ds/settings/templates/:templateid", () =>
    <ShortAppDetailsStateContextProvider ShortAppDetailsStateClass={appDetailsState} >
      <ServerApiProvider serverApi={serverApi}>
        <TemplatePreviewRoute />
      </ServerApiProvider>
    </ShortAppDetailsStateContextProvider>, {
    exact: true
  });

  serverApi.routerHook.addRoute("/deck-ds/settings/profiles/:profileid", () =>
    <ShortAppDetailsStateContextProvider ShortAppDetailsStateClass={appDetailsState} >
      <ServerApiProvider serverApi={serverApi}>
        <ProfileRoute />
      </ServerApiProvider>
    </ShortAppDetailsStateContextProvider>, {
    exact: true
  });

  // Settings Route
  serverApi.routerHook.addRoute("/deck-ds/settings/:setting", () =>
    <ShortAppDetailsStateContextProvider ShortAppDetailsStateClass={appDetailsState} >
      <ServerApiProvider serverApi={serverApi}>
        <SettingsRouter />
      </ServerApiProvider>
    </ShortAppDetailsStateContextProvider>, {
    exact: true,
  });

  return {
    title: <div className={staticClasses.Title}>DeckDS</div>,
    content: <Content serverApi={serverApi} />,
    icon: <FaShip />,
    onDismount() {

      backend.log(backend.LogLevel.Debug, "DeckDS shutting down");

      appDetailsState.setGamesRunning([]);
      appDetailsState.setOnAppPage(null);

      serverApi.routerHook.removeRoute("/deck-ds/settings/templates/:templateid");
      serverApi.routerHook.removeRoute("/deck-ds/settings/:setting");
      serverApi.routerHook.removePatch('/library/app/:appid', libraryPatch)
    },
  };
});
