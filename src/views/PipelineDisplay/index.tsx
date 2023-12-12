import { Focusable, Tabs } from "decky-frontend-lib";
import { ReactElement, useState } from "react";
import { ActionSelection } from "../../backend";
import HandleLoading from "../../components/HandleLoading";
import { useModifiablePipelineDefinition } from "../../context/modifiablePipelineContext";
import useReifiedPipeline from "../../hooks/useReifiedPipeline";
import PipelineInfo from "./PipelineInfo";
import PipelineTargetDisplay from "./PipelineTargetDisplay";

interface PipelineDisplayProps {
    secondaryAction?: () => void
    secondaryActionDescription?: string
}

export default function PipelineDisplay({ secondaryAction, secondaryActionDescription }: PipelineDisplayProps): ReactElement {
    const [currentTabRoute, setCurrentTabRoute] = useState<string>("info")

    const { state } = useModifiablePipelineDefinition();
    console.log('pipeline display updated with state', state.definition);
    const result = useReifiedPipeline(state.definition);

    return (
        <HandleLoading
            value={result}
            onOk={
                (pipeline) => {
                    interface KeyValue {
                        target: string,
                        root: ActionSelection,
                    }

                    const defaultTargets: KeyValue[] = [];
                    const extraTargets: KeyValue[] = [] // no real intention of actually supporting extra targets, but...

                    for (const key in pipeline.targets) {
                        const value: KeyValue = {
                            target: key,
                            root: pipeline.targets[key],
                        };

                        if (key === 'Gamemode') {
                            defaultTargets.push(value);
                        } else if (key === 'Desktop') {
                            defaultTargets.splice(0, 0, value);
                        } else {
                            extraTargets.push(value)
                        }
                    }


                    const allTargets = defaultTargets.concat(extraTargets);

                    const tabs = [
                        {
                            title: "Info",
                            content: <PipelineInfo pipeline={pipeline} />,
                            id: "info",
                        },
                        ...allTargets.map((kv) => {
                            return {
                                title: kv.target,
                                content: <PipelineTargetDisplay root={kv.root} />,
                                id: kv.target.toLowerCase(),
                            };
                        }),
                    ];

                    return <Focusable
                        style={{ minWidth: "100%", minHeight: "100%" }}
                        onSecondaryActionDescription={secondaryActionDescription}
                        onSecondaryButton={secondaryAction}
                        onClick={secondaryAction}
                    >
                        <div
                            style={{
                                marginTop: "40px",
                                height: "calc(100% - 40px)",
                            }}>
                            <Tabs
                                activeTab={currentTabRoute}
                                onShowTab={(tabID: string) => {
                                    setCurrentTabRoute(tabID);
                                }}
                                tabs={tabs}
                            />
                        </div>
                    </Focusable>
                }
            }
        />
    );
}