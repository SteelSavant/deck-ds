import { ButtonItem, DialogButton, PanelSection, PanelSectionRow, Router } from "decky-frontend-lib";
import { Fragment, ReactElement, useState } from "react";
import { RiArrowDownSFill, RiArrowRightSFill } from "react-icons/ri";
import { AppProfileOveride, PipelineTarget, setAppProfileOverride } from "../../backend";
import HandleLoading from "../../components/HandleLoading";
import { IconForTarget } from "../../components/IconForTarget";
import { ShortAppDetails, useAppState } from "../../context/appContext";
import { ModifiablePipelineContainerProvider, useModifiablePipelineContainer } from "../../context/modifiablePipelineContext";
import useLaunchActions, { LaunchActions } from "../../hooks/useLaunchActions";
import useReifiedPipeline from "../../hooks/useReifiedPipeline";
import AppDefaultProfileDropdown from "./AppDefaultProfileDropdown";
import QAMPipelineTargetDisplay from "./QAMPipelineTargetDisplay";


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
                        onOk={(appProfile) => < AppDefaultProfileDropdown
                            appDetails={appDetails}
                            appProfile={appProfile}
                            launchActions={launchActions}
                        />}
                        onErr={(err) => <p>{err.err}</p>}
                    />
                    <DeckDSProfilesForApp appDetails={appDetails} launchActions={launchActions} />
                </Fragment>
                : <div />
            }
        </Fragment>
    )
}

function DeckDSProfilesForApp({ appDetails, launchActions }: { appDetails: ShortAppDetails, launchActions: LaunchActions[] }): ReactElement {
    return <Fragment >
        {
            launchActions.map((a) => {
                // TODO::display icon next to target

                const initial: AppProfileOveride = {
                    appId: appDetails.appId.toString(),
                    profileId: a.profile.id,
                    pipeline: a.profile.pipeline
                };

                return (
                    <ModifiablePipelineContainerProvider
                        initialContainer={initial}
                        onUpdate={(pipelineSettings) => {
                            const typed = pipelineSettings as AppProfileOveride
                            setAppProfileOverride({
                                app_id: typed.appId,
                                pipeline: typed.pipeline,
                                profile_id: typed.profileId
                            })
                        }}
                    >
                        <AppProfileSection launchActions={a} />
                    </ModifiablePipelineContainerProvider>
                )
            })
        }
    </Fragment>
    // TODO::horizonal line at end of fragment
}

function AppProfileSection({ launchActions }: { launchActions: LaunchActions }): ReactElement {
    const [view, setView] = useState<{ [k: string]: boolean }>({});

    const height = '40px';
    const margin = '5px';
    return <PanelSection title={launchActions.profile.pipeline.name} >
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
                                backgroundColor: view[t.target]
                                    ? 'lightgreen'
                                    : undefined
                            }}
                            onClick={() => {
                                const newView = { ...view };
                                newView[t.target] = !newView[t.target];
                                setView(newView)
                            }}
                        >
                            {
                                view[t.target]
                                    ? <RiArrowDownSFill style={{ padding: 0, margin: 0, minWidth: 0, objectFit: 'fill', }} />
                                    : <RiArrowRightSFill style={{ padding: 0, margin: 0, minWidth: 0, objectFit: 'fill' }} />
                            }
                        </DialogButton>
                    </div>
                    {
                        view[t.target] ?
                            <QAMTarget target={t.target
                            } />
                            : <div />
                    }
                </Fragment>
            ))
        }
    </PanelSection>;
}

function QAMTarget({ target }: { target: PipelineTarget }): ReactElement {
    const { state } = useModifiablePipelineContainer();

    const reified = useReifiedPipeline(state.container.pipeline);

    return <HandleLoading
        value={reified}
        onOk={(reified) => {
            const selection = reified.targets[target];
            return <QAMPipelineTargetDisplay root={selection} />
        }}
    />

}