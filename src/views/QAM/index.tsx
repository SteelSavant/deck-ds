import {
    DialogButton,
    Field,
    Navigation,
    PanelSection,
    Router,
    showModal,
} from '@decky/ui';
import { Fragment, ReactElement } from 'react';
import { FaGear, FaPlus } from 'react-icons/fa6';
import { RiArrowDownSFill, RiArrowRightSFill } from 'react-icons/ri';
import {
    PipelineTarget,
    ReifyPipelineResponse,
    getTemplates,
} from '../../backend';
import { CreateProfileModal } from '../../components/CreateProfileModal';
import FocusableRow from '../../components/FocusableRow';
import HandleLoading from '../../components/HandleLoading';
import { IconForTarget } from '../../components/IconForTarget';
import { ShortAppDetails, useAppState } from '../../context/appContext';
import { ConfigErrorContext } from '../../context/configErrorContext';
import useLaunchActions, { LaunchActions } from '../../hooks/useLaunchActions';
import { logger } from '../../util/log';
import AppDefaultProfileDropdown from './AppDefaultProfileDropdown';
import QAMPipelineTargetDisplay, {
    ProfileContext,
} from './QAMPipelineTargetDisplay';

export default function QAM(): ReactElement {
    const { appDetails, appProfile } = useAppState();
    const launchActions = useLaunchActions(appDetails);

    return appDetails ? (
        <HandleLoading
            value={appProfile}
            onOk={(appProfile) => (
                <>
                    <AppDefaultProfileDropdown
                        appDetails={appDetails}
                        appProfile={appProfile}
                        launchActions={launchActions}
                    />
                    <DeckDSProfilesForApp
                        appDetails={appDetails}
                        launchActions={launchActions}
                    />
                </>
            )}
            onErr={(err) => <p>{err.err}</p>}
        />
    ) : (
        <PanelSection>
            <p>Welcome to DeckDS!</p>
            <p>
                To set up profiles or edit settings, go to settings (top right).
            </p>
            <p>
                Launch actions and per-app settings will appear here on the app
                page of configured titles.
            </p>
        </PanelSection>
    );
}

function DeckDSProfilesForApp({
    appDetails,
    launchActions,
}: {
    appDetails: ShortAppDetails;
    launchActions: LaunchActions[];
}): ReactElement {
    return launchActions.length > 0 ? (
        <>
            {launchActions.map((a) => {
                return (
                    <ProfileContext.Provider value={a.profileId}>
                        <AppProfileSection launchActions={a} />
                    </ProfileContext.Provider>
                );
            })}
        </>
    ) : (
        // TODO::horizonal line at end of fragment
        <PanelSection>
            <p>No profiles configured for this title.</p>
            <p>
                To set one, add one of the following collections to an existing
                profile, or use the "+" button to create a new profile using
                that collection:{' '}
            </p>
            {collectionStore.userCollections
                .filter((uc) => uc.apps.has(appDetails.appId))
                .map((c) => {
                    const createNewProfile = async () => {
                        const templates = await getTemplates();
                        if (templates.isOk) {
                            showModal(
                                <CreateProfileModal
                                    templates={templates.data.templates}
                                    collection={c}
                                />,
                            );
                        } else {
                            logger.toastError(
                                'Failed to load profile templates:',
                                templates.err,
                            );
                        }
                    };

                    return (
                        <Field
                            focusable={false}
                            label={`  ${c.displayName}`}
                            inlineWrap="keep-inline"
                            bottomSeparator="none"
                        >
                            <DialogButton
                                onClick={createNewProfile}
                                onOKButton={createNewProfile}
                                style={{
                                    height: '40px',
                                    width: '40px',
                                    padding: '10px 12px',
                                    minWidth: '40px',
                                    display: 'flex',
                                    flexDirection: 'column',
                                    justifyContent: 'center',
                                    marginRight: '10px',
                                }}
                            >
                                <FaPlus />
                            </DialogButton>
                        </Field>
                    );
                })}
        </PanelSection>
    );
}

function AppProfileSection({
    launchActions,
}: {
    launchActions: LaunchActions;
}): ReactElement {
    const height = '40px';
    const margin = '5px';
    const profileId = launchActions.profileId;

    const { reifiedPipelines } = useAppState();
    const reified = reifiedPipelines[profileId];

    const { openViews, setAppViewOpen } = useAppState();

    const openProfileSettings = () => {
        Navigation.Navigate(`/deck-ds/settings/profiles/${profileId}`);
        Router.CloseSideMenus();
    };

    return (
        <HandleLoading
            value={reified}
            onOk={(reified) => {
                const title: any = (
                    <div
                        style={{
                            display: 'flex',
                            flexDirection: 'row',
                            justifyContent: 'space-between',
                            alignItems: 'center',
                        }}
                    >
                        <p>{reified.pipeline.name}</p>
                        <DialogButton
                            style={{
                                width: 'fit-content',
                                minWidth: 'fit-content',
                                height: 'fit-content',
                                minHeight: 'fit-content',
                                paddingLeft: 10,
                                paddingRight: 10,
                                paddingTop: 5,
                                paddingBottom: 5,
                            }}
                            onClick={openProfileSettings}
                            onOKButton={openProfileSettings}
                        >
                            <FaGear />
                        </DialogButton>
                    </div>
                );

                return (
                    // TODO::settings button inline with title that takes you to the settings for that app
                    <PanelSection title={title}>
                        {launchActions.targets
                            .filter((v) => reified.pipeline.targets[v.target])
                            .map((t) => {
                                const isOpen = openViews[profileId]?.[t.target];
                                return (
                                    <>
                                        <FocusableRow>
                                            <DialogButton
                                                style={{
                                                    display: 'flex',
                                                    flexDirection: 'row',
                                                    justifyContent:
                                                        'space-between',
                                                    alignItems: 'center',
                                                    width: '90%',
                                                    maxWidth: '90%',
                                                    minWidth: 0,
                                                    height,
                                                    marginRight: margin,
                                                    marginBottom: margin,
                                                    borderTopRightRadius: 0,
                                                    borderBottomRightRadius: 0,
                                                }}
                                                onClick={t.action}
                                                onOKButton={t.action}
                                            >
                                                <IconForTarget
                                                    target={t.target}
                                                />
                                                {t.target}
                                            </DialogButton>
                                            <DialogButton
                                                style={{
                                                    alignItems: 'center',
                                                    justifyItems: 'center',
                                                    width: '10%',
                                                    minWidth: 0,
                                                    height,
                                                    marginBottom: margin,
                                                    borderTopLeftRadius: 0,
                                                    borderBottomLeftRadius: 0,
                                                    padding: 0,
                                                    backgroundColor: isOpen
                                                        ? 'lightgreen'
                                                        : undefined,
                                                }}
                                                onClick={() => {
                                                    setAppViewOpen(
                                                        profileId,
                                                        t.target,
                                                        !isOpen,
                                                    );
                                                }}
                                            >
                                                {isOpen ? (
                                                    <RiArrowDownSFill
                                                        style={{
                                                            padding: 0,
                                                            margin: 0,
                                                            minWidth: 0,
                                                            objectFit: 'fill',
                                                        }}
                                                    />
                                                ) : (
                                                    <RiArrowRightSFill
                                                        style={{
                                                            padding: 0,
                                                            margin: 0,
                                                            minWidth: 0,
                                                            objectFit: 'fill',
                                                        }}
                                                    />
                                                )}
                                            </DialogButton>
                                        </FocusableRow>
                                        {isOpen ? (
                                            <QAMTarget
                                                reified={reified}
                                                target={t.target}
                                            />
                                        ) : null}
                                    </>
                                );
                            })}
                    </PanelSection>
                );
            }}
        />
    );
}

function QAMTarget({
    reified,
    target,
}: {
    reified: ReifyPipelineResponse;
    target: PipelineTarget;
}): ReactElement {
    const selection = reified.pipeline.targets[target];

    return selection ? (
        <ConfigErrorContext.Provider value={reified.config_errors}>
            <QAMPipelineTargetDisplay root={selection} target={target} />
        </ConfigErrorContext.Provider>
    ) : (
        <Fragment />
    );
}
