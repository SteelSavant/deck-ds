import { DialogButton, PanelSection } from "decky-frontend-lib";
import { Fragment, ReactElement, createContext } from "react";
import { RiArrowDownSFill, RiArrowRightSFill } from "react-icons/ri";
import { PipelineTarget } from "../../backend";
import FocusableRow from "../../components/FocusableRow";
import HandleLoading from "../../components/HandleLoading";
import { IconForTarget } from "../../components/IconForTarget";
import { ShortAppDetails, useAppState } from "../../context/appContext";
import { ConfigErrorContext } from "../../context/configErrorContext";
import useEnsureAppOverridePipeline from "../../hooks/useEnsureAppOverridePipeline";
import useLaunchActions, { LaunchActions } from "../../hooks/useLaunchActions";
import AppDefaultProfileDropdown from "./AppDefaultProfileDropdown";
import QAMPipelineTargetDisplay from "./QAMPipelineTargetDisplay";

export const ProfileContext = createContext("notset");

export default function QAM(): ReactElement {
    const { appDetails, appProfile } = useAppState();
    const launchActions = useLaunchActions(appDetails);

    return appDetails ?
        <HandleLoading
            value={appProfile}
            onOk={(appProfile) =>
                <Fragment>
                    < AppDefaultProfileDropdown
                        appDetails={appDetails}
                        appProfile={appProfile}
                        launchActions={launchActions}
                    />
                    <DeckDSProfilesForApp
                        appDetails={appDetails}
                        launchActions={launchActions}
                    />
                </Fragment>
            }
            onErr={(err) => <p>{err.err}</p>}
        />
        : <PanelSection >
            <p>Welcome to DeckDS!</p>
            <p>To set up profiles or edit settings, go to settings (top right).</p>
            <p>Launch actions and per-app settings will appear here on the app page of configured titles.</p>
        </PanelSection>
}



function DeckDSProfilesForApp({ appDetails, launchActions }: { appDetails: ShortAppDetails, launchActions: LaunchActions[] }): ReactElement {

    return launchActions.length > 0
        ? <Fragment >
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
        : <PanelSection >
            <p>No profiles configured for this title.</p>
            <p>To set one, add one of the following collections to an existing profile: </p>
            {
                collectionStore
                    .userCollections
                    .filter((uc) => uc.apps.get(appDetails.appId))
                    .map((c) => <li>{c.displayName}</li>)
            }
        </PanelSection>
}

function AppProfileSection({ launchActions }: { launchActions: LaunchActions }): ReactElement {
    const height = '40px';
    const margin = '5px';
    const profileId = launchActions.profile.id;

    const { openViews, setAppViewOpen } = useAppState();

    return (
        <PanelSection title={launchActions.profile.pipeline.name} >
            {
                launchActions.targets.map((t) => {
                    const isOpen = openViews[profileId]?.[t.target];
                    return (
                        <Fragment>
                            <FocusableRow>
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
                                        backgroundColor: isOpen
                                            ? 'lightgreen'
                                            : undefined
                                    }}
                                    onClick={() => {
                                        setAppViewOpen(profileId, t.target, !isOpen)
                                    }}
                                >
                                    {
                                        isOpen
                                            ? <RiArrowDownSFill style={{ padding: 0, margin: 0, minWidth: 0, objectFit: 'fill' }} />
                                            : <RiArrowRightSFill style={{ padding: 0, margin: 0, minWidth: 0, objectFit: 'fill' }} />
                                    }
                                </DialogButton>


                            </FocusableRow>
                            {
                                isOpen ?
                                    <QAMTarget profileId={profileId} target={t.target} />
                                    : <div />
                            }
                        </Fragment>
                    )
                })
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
            const selection = reified.pipeline.targets[target];
            console.log('setting QAM errors:', reified.config_errors);
            console.log('setting QAM selection:', selection);

            return (
                <ConfigErrorContext.Provider value={reified.config_errors} >
                    <QAMPipelineTargetDisplay root={selection} />
                </ConfigErrorContext.Provider>
            )
        }}
    />

}