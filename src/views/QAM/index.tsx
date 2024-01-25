import { ButtonItem, DialogButton, PanelSection, PanelSectionRow, Router } from "decky-frontend-lib";
import { Fragment, ReactElement, createContext } from "react";
import { RiArrowDownSFill, RiArrowRightSFill } from "react-icons/ri";
import { PipelineTarget } from "../../backend";
import HandleLoading from "../../components/HandleLoading";
import { IconForTarget } from "../../components/IconForTarget";
import { useAppState } from "../../context/appContext";
import useEnsureAppOverridePipeline from "../../hooks/useEnsureAppOverridePipeline";
import useLaunchActions, { LaunchActions } from "../../hooks/useLaunchActions";
import AppDefaultProfileDropdown from "./AppDefaultProfileDropdown";
import QAMPipelineTargetDisplay from "./QAMPipelineTargetDisplay";

export const ProfileContext = createContext("notset");

export default function QAM(): ReactElement {
    const { appDetails, appProfile } = useAppState();
    const launchActions = useLaunchActions(appDetails);

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
                        onOk={(appProfile) =>
                            <Fragment>
                                < AppDefaultProfileDropdown
                                    appDetails={appDetails}
                                    appProfile={appProfile}
                                    launchActions={launchActions}
                                />
                                <DeckDSProfilesForApp launchActions={launchActions} />
                            </Fragment>
                        }
                        onErr={(err) => <p>{err.err}</p>}
                    />
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
                return (
                    <ProfileContext.Provider value={a.profile.id}>
                        <AppProfileSection launchActions={a} />
                    </ProfileContext.Provider>
                )
            })
        }
    </Fragment >
    // TODO::horizonal line at end of fragment
}

function AppProfileSection({ launchActions }: { launchActions: LaunchActions }): ReactElement {
    const height = '40px';
    const margin = '5px';
    const profileId = launchActions.profile.id;

    const { openViews, setAppViewOpen } = useAppState();

    return (<PanelSection title={launchActions.profile.pipeline.name} >
        {
            launchActions.targets.map((t) => (
                <Fragment>
                    <div style={{ display: 'flex', flexDirection: 'row', width: '100%' }}>
                        <DialogButton
                            style={{
                                display: 'flex',
                                flexDirection: 'row',
                                justifyContent: 'space-between',
                                alignItems: 'center',
                                width: "90%",
                                maxWidth: "90%",
                                minWidth: 0,
                                height,
                                marginRight: margin,
                                marginBottom: margin,
                                borderTopRightRadius: 0,
                                borderBottomRightRadius: 0
                            }}
                            onClick={t.action}
                            onOKButton={t.action}
                        >
                            <IconForTarget target={t.target} />
                            {t.target}
                        </DialogButton>
                        <DialogButton
                            style={{
                                alignItems: 'center',
                                justifyItems: 'center',
                                width: "10%",
                                minWidth: 0,
                                height,
                                marginBottom: margin,
                                borderTopLeftRadius: 0,
                                borderBottomLeftRadius: 0,
                                padding: 0,
                                backgroundColor: openViews[t.target]
                                    ? 'lightgreen'
                                    : undefined
                            }}
                            onClick={() => {
                                setAppViewOpen(profileId, t.target, !openViews[profileId]?.[t.target])
                            }}
                        >
                            {
                                openViews[profileId]?.[t.target]
                                    ? <RiArrowDownSFill style={{ padding: 0, margin: 0, minWidth: 0, objectFit: 'fill', }} />
                                    : <RiArrowRightSFill style={{ padding: 0, margin: 0, minWidth: 0, objectFit: 'fill' }} />
                            }
                        </DialogButton>
                    </div>
                    {
                        openViews[profileId]?.[t.target] ?
                            <QAMTarget profileId={profileId} target={t.target
                            } />
                            : <div />
                    }
                </Fragment>
            ))
        }
    </PanelSection>
    );
}

function QAMTarget({ profileId, target }: { profileId: string, target: PipelineTarget }): ReactElement {
    useEnsureAppOverridePipeline(profileId);
    const { reifiedPipelines } = useAppState();

    const reified = reifiedPipelines[profileId];

    return <HandleLoading
        value={reified}
        onOk={(reified) => {
            const selection = reified.targets[target];
            return <QAMPipelineTargetDisplay root={selection} />
        }}
    />

}