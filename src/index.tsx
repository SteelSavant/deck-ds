import {
  ButtonItem,
  definePlugin,

  PanelSection,
  PanelSectionRow,
  Router,
  ServerAPI,
  staticClasses,
} from "decky-frontend-lib";
import { VFC } from "react";
import { FaShip } from "react-icons/fa";

import * as backend from "./backend";
import SettingsRouter from "./views/Settings/SettingsRouter";
import TemplatePreviewRoute from "./views/TemplatePreviewRoute";

var usdplReady = false;


(async function () {
  await backend.initBackend();
  usdplReady = true;
})()

const Content: VFC<{ serverAPI: ServerAPI }> = ({ serverAPI }) => {
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
    <PanelSection title="Panel Section">
      <PanelSectionRow>
        <ButtonItem
          layout="below"
          onClick={() => {
            Router.CloseSideMenus();
            Router.Navigate("/deck-ds/settings/profiles");
          }}
        >
          Configuration
        </ButtonItem>
        <ButtonItem
          layout="below"
          onClick={async () => {
            let res = await backend.getTemplates();
            if (res.isOk) {
              serverAPI.toaster.toast({
                title: "Error",
                body: ["Got ", res.data.templates.length, " templates."].join(''),
              });
            } else {
              serverAPI.toaster.toast({
                title: "Error",
                body: ["Err: ", res.err.code, ": ", res.err].join('')
              });
            }
          }}
        >
          Template Count
        </ButtonItem>
        <ButtonItem
          layout="below"
          onClick={async () => {
            await backend.autoStart({
              app: "17589260553245687808",
              pipeline: {
                name: "test",
                description: "test",
                tags: [],
                targets: {
                  'Desktop': {
                    type: "AllOf",
                    value: []
                  }
                },
              },
              target: "Desktop"
            })
          }}
        >
          Desktop Test
        </ButtonItem>
        <ButtonItem
          layout="below"
          onClick={async () => {
            await backend.autoStart({
              app: "17589260553245687808",
              pipeline: {
                name: "test",
                description: "test",
                tags: [],
                targets: {
                  'Gamemode': {
                    type: "AllOf",
                    value: []
                  }
                },
              },
              target: "Gamemode"
            })
          }}
        >
          Gamemode Test
        </ButtonItem>
      </PanelSectionRow>
    </PanelSection >
  );
}

export default definePlugin((serverApi: ServerAPI) => {
  // Template Preview Route
  serverApi.routerHook.addRoute("/deck-ds/settings/templates/:templateid", () => <TemplatePreviewRoute />, {
    exact: true
  });

  // Settings Route
  serverApi.routerHook.addRoute("/deck-ds/settings/:setting", SettingsRouter, {
    exact: true,
  });

  return {
    title: <div className={staticClasses.Title}>DeckDS</div>,
    content: <Content serverAPI={serverApi} />,
    icon: <FaShip />,
    onDismount() {
      backend.log(backend.LogLevel.Debug, "DeckDS shutting down");
      serverApi.routerHook.removeRoute("/deck-ds/settings/templates/:templateid");
      serverApi.routerHook.removeRoute("/deck-ds/settings/:setting");
    },
  };
});
