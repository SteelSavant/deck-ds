import { ButtonItem, PanelSection, PanelSectionRow, Router } from "decky-frontend-lib";
import { Fragment, ReactElement } from "react";
import { ShortAppDetails, useShortAppDetailsState } from "../context/shortAppDetailsContext";
import useLaunchActions from "../hooks/useLaunchActions";

export default function QAM(): ReactElement {
    const appDetailsState = useShortAppDetailsState();
    const appDetails = appDetailsState.appDetails;

    return (
        <Fragment>
            <PanelSection>
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
                </PanelSectionRow>
            </PanelSection >
            {appDetails ? <DeckDSProfilesForApp appDetails={appDetails} /> : <div />}
        </Fragment>
    )
}

function DeckDSProfilesForApp({ appDetails }: { appDetails: ShortAppDetails }): ReactElement {
    const launchActions = useLaunchActions(appDetails);



    return <Fragment >
        {launchActions.map((a) => {


            // TODO::display icon next to target

            return <PanelSection title={a.profile.pipeline.name}>
                {
                    a.targets.map((t) => {
                        return (
                            <PanelSectionRow>
                                <ButtonItem
                                    layout="below"
                                    onClick={t.action}
                                >
                                    {t.target}
                                </ButtonItem>
                            </PanelSectionRow>
                        )
                    })
                }
            </PanelSection>
        })}
    </Fragment>
}