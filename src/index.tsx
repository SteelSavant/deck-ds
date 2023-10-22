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
  // const [result, setResult] = useState<number | undefined>();

  // const onClick = async () => {
  //   const result = await serverAPI.callPluginMethod<AddMethodArgs, number>(
  //     "add",
  //     {
  //       left: 2,
  //       right: 2,
  //     }
  //   );
  //   if (result.success) {
  //     setResult(result.result);
  //   }
  // };


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

  //#region Find SteamOS modules
const findModule = (property: string) => {
  return findModuleChild((m: Module) => {
    if (typeof m !== "object") return undefined;
    for (let prop in m) {
      try {
        if (m[prop][property])
          return m[prop]
      } catch {
        return undefined
      }
    }
  })
}

  const NavSoundMap = findModule("ToastMisc");


  return (
    <PanelSection title="Panel Section">
      <PanelSectionRow>
        <ButtonItem
          layout="below"
          onClick={async (_: MouseEvent) => {
            let logged = await backend.log(backend.LogLevel.Info, "Msg from frontend!")
            serverAPI.toaster.toast({
              title: "DeckDS",
              body: logged ? "Log sent successfully!" : "Log failed",
              duration: 5000,
              sound:  NavSoundMap?.ToastMisc,
              playSound: true,
              showToast: true
            });

            let path = await backend.logPath();

            serverAPI.toaster.toast({
              title: "DeckDS",
              body: "Log set at path " + path,
              duration: 5000,
              sound:  NavSoundMap?.ToastMisc,
              playSound: true,
              showToast: true
            })
          }}
        >
          Server says yolo
        </ButtonItem>
      </PanelSectionRow>

      <PanelSectionRow>
        <div style={{ display: "flex", justifyContent: "center" }}>
          <img src={logo} />
        </div>
      </PanelSectionRow>

      {/* <PanelSectionRow>
        <ButtonItem
          layout="below"
          onClick={() => {
            Router.CloseSideMenus();
            Router.Navigate("/deck-ds");
          }}
        >
          Router
        </ButtonItem>
      </PanelSectionRow> */}
    </PanelSection>
  );
};

const DeckyPluginRouterTest: VFC = () => {
  return (
    <div style={{ marginTop: "50px", color: "white" }}>
      Hello World!
      <DialogButton onClick={() => Router.NavigateToLibraryTab()}>
        Go to Library
      </DialogButton>
    </div>
  );
};

export default definePlugin((serverApi: ServerAPI) => {
  serverApi.routerHook.addRoute("/deck-ds", DeckyPluginRouterTest, {
    exact: true,
  });

  return {
    title: <div className={staticClasses.Title}>DeckDS</div>,
    content: <Content serverAPI={serverApi} />,
    icon: <FaShip />,
    onDismount() {
      backend.log(backend.LogLevel.Debug, "DeckDS shutting down");

      // serverApi.routerHook.removeRoute("/deck-ds");
    },
  };
});
