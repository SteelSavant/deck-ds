import { ButtonItem, PanelSection, PanelSectionRow, Router } from "decky-frontend-lib";
import { Fragment, ReactElement } from "react";
import { PipelineTarget, autoStart, reifyPipeline } from "../backend";
import HandleLoading from "../components/HandleLoading";
import { ShortAppDetails, useShortAppDetailsState } from "../context/shortAppDetailsContext";
import useProfiles from "../hooks/useProfiles";

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
    const { profiles } = useProfiles();

    return <HandleLoading value={profiles}
        onOk={(profiles) => {
            const validProfiles = profiles
                .flatMap((p) =>
                    collectionStore.userCollections
                        .map((uc) => {
                            const containsApp = uc.apps.get(appDetails.appId);
                            const isMatch = containsApp && p.pipeline.tags.includes(uc.id);
                            if (isMatch) {
                                console.log('Collection', uc.displayName, 'matches profile', p.pipeline.name, 'tags', p.pipeline.tags);
                                return p;
                            } else {
                                return null;
                            }
                        })
                        .filter((p) => p)
                        .map((p) => p!)
                )             // not efficient, don't care right now

            return <Fragment >
                {validProfiles.map((p) => {
                    const targets = p.pipeline.targets
                    const defaultTargets = [];

                    for (const key in targets) {
                        const value = {
                            target: key,
                            action: async () => {
                                const reified = (await reifyPipeline({
                                    pipeline: p.pipeline
                                }));


                                if (reified.isOk) {
                                    const res = await autoStart({
                                        app: appDetails.gameId,
                                        pipeline: reified.data.pipeline,
                                        target: key as PipelineTarget
                                    });

                                    if (!res.isOk) {
                                        // TODO::handle error
                                    }
                                } else {
                                    // TODO::handle error
                                }

                            }
                        };

                        if (key === 'Gamemode') {
                            defaultTargets.push(value);
                        } else if (key === 'Desktop') {
                            defaultTargets.splice(0, 0, value);
                        } else {
                            // extra targets not planned or handled
                        }
                    }

                    // TODO::display icon next to target

                    return <PanelSection title={p.pipeline.name}>
                        {
                            defaultTargets.map((t) => {
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
        }}
    />

}