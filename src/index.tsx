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
import SettingsRouter from "./Settings/SettingsRouter";

var usdplReady = false;


(async function () {
  await backend.initBackend();
  usdplReady = true;
})()


// interface AddMethodArgs {
//   left: number;
//   right: number;
// }

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
            Router.Navigate("/deck-ds");
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
          Template Count (3)
        </ButtonItem>
      </PanelSectionRow>
    </PanelSection>
  );
}

export default definePlugin((serverApi: ServerAPI) => {
  serverApi.routerHook.addRoute("/deck-ds", SettingsRouter, {
    exact: true,
  });

  return {
    title: <div className={staticClasses.Title}>DeckDS</div>,
    content: <Content serverAPI={serverApi} />,
    icon: <FaShip />,
    onDismount() {
      backend.log(backend.LogLevel.Debug, "DeckDS shutting down");

      serverApi.routerHook.removeRoute("/deck-ds");
    },
  };
});
