import { ButtonItem, PanelSection, PanelSectionRow, Router } from "decky-frontend-lib";
import { Fragment, ReactElement } from "react";
import { PipelineTarget, autoStart, reifyPipeline } from "../backend";
import HandleLoading from "../components/HandleLoading";
import { useShortAppDetailsState } from "../context/shortAppDetailsContext";
import useProfiles from "../hooks/useProfiles";

export default function QAM(): ReactElement {
    const appDetailsState = useShortAppDetailsState();

    const appId = appDetailsState.appDetails?.appId;
    // TODO::handle view in game

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
            {appId ? <DeckDSProfilesForApp appId={appId} /> : <div />}
        </Fragment>
    )
}

function DeckDSProfilesForApp({ appId }: { appId: number }): ReactElement {
    const { profiles } = useProfiles();

    return <HandleLoading value={profiles}
        onOk={(profiles) => {
            const validProfiles = profiles.map((p) =>
                collectionStore.userCollections.map((uc) =>
                    p.pipeline.tags.includes(uc.id))
                    ? p
                    : null
            ).filter((p) => p).map((p) => p!); // not efficient, don't care right now


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
                                        app: appId.toString(),
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