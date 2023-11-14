import {
  ButtonItem,
  definePlugin,
  DialogButton,
  findModule,
  findModuleChild,
  Menu,
  MenuItem,
  Module,
  PanelSection,
  PanelSectionRow,
  Router,
  ServerAPI,
  staticClasses,
} from "decky-frontend-lib";
import { VFC } from "react";
import { FaShip } from "react-icons/fa";

import logo from "../assets/logo.png";

import * as backend from "./backend";
import { tr } from "usdpl-front";
import { set_value, get_value } from "usdpl-front";
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
