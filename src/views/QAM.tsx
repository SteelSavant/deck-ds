import { ButtonItem, PanelSection, PanelSectionRow, Router } from "decky-frontend-lib";
import { Fragment, ReactElement } from "react";
import AppDefaultProfileDropdown from "../components/AppDefaultProfileDropdown";
import HandleLoading from "../components/HandleLoading";
import { IconForTarget } from "../components/IconForTarget";
import { useShortAppDetailsState } from "../context/shortAppDetailsContext";
import useAppProfile from "../hooks/useAppSettings";
import useLaunchActions, { LaunchActions } from "../hooks/useLaunchActions";

export default function QAM(): ReactElement {
    const appDetailsState = useShortAppDetailsState();
    const appDetails = appDetailsState.appDetails;

    const launchActions = useLaunchActions(appDetails);
    const appProfile = useAppProfile(appDetails);

    console.log('launchActions:', launchActions);
    console.log('appProfile', appProfile);

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
            </PanelSection>
            {appDetails ?
                <Fragment>
                    <HandleLoading
                        value={appProfile}
                        onOk={(appProfile) => < AppDefaultProfileDropdown
                            appDetails={appDetails}
                            appProfile={appProfile}
                            launchActions={launchActions}
                        />}
                        onErr={(err) => <p>{err.err}</p>}
                    />
                    <DeckDSProfilesForApp launchActions={launchActions} />
                </Fragment>
                : <div />
            }

        </Fragment>
    )
}

function DeckDSProfilesForApp({ launchActions }: { launchActions: LaunchActions[] }): ReactElement {
    return <Fragment >
        {
            launchActions.map((a) => {
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
                                        <div style={{
                                            display: 'flex',
                                            justifyContent: 'space-between'
                                        }}>
                                            <IconForTarget target={t.target} />
                                            {t.target}
                                        </div>
                                    </ButtonItem>
                                </PanelSectionRow>
                            )
                        })
                    }
                </PanelSection>
            })
        }
    </Fragment>
}