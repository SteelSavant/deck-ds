import { Focusable, Tabs } from "decky-frontend-lib";
import { ReactElement, useEffect, useRef, useState } from "react";
import { ActionSelection, PipelineContainer, PipelineTarget } from "../../backend";
import HandleLoading from "../../components/HandleLoading";
import { IconForTarget } from "../../components/IconForTarget";
import { ConfigErrorContext } from "../../context/configErrorContext";
import { useModifiablePipelineContainer } from "../../context/modifiablePipelineContext";
import useReifiedPipeline from "../../hooks/useReifiedPipeline";
import PipelineHeader from "./PipelineHeader";
import PipelineTargetDisplay from "./PipelineTargetDisplay";


interface PipelineDisplayProps {
    header: (container: PipelineContainer) => ReactElement,
    general: (container: PipelineContainer) => ReactElement,
    secondaryAction?: () => void
    secondaryActionDescription?: string
}

export default function PipelineDisplay({ header, general, secondaryAction, secondaryActionDescription }: PipelineDisplayProps): ReactElement {
    const [currentTabRoute, setCurrentTabRoute] = useState<string>("general")

    const { state } = useModifiablePipelineContainer();
    const result = useReifiedPipeline(state.container.pipeline);

    let container = useRef<HTMLDivElement>(null);
    let [headerHeight, setHeaderHeight] = useState<number | null>(null);

    useEffect(() => setHeaderHeight(container?.current?.offsetHeight ?? 0));

    return (
        <HandleLoading
            value={result}
            onOk={
                ({ pipeline, config_errors }) => {
                    interface TargetDescriptor {
                        target: string,
                        root: ActionSelection,
                        description: ReactElement,
                    }

                    const defaultTargets: TargetDescriptor[] = [];
                    const extraTargets: TargetDescriptor[] = [] // no real intention of actually supporting extra targets, but...

                    const descriptions: { [k: string]: string } = {
                        ['Gamemode']: 'Game',
                        ['Desktop']: 'Desktop'
                    }

                    for (const key in pipeline.targets) {
                        const value: TargetDescriptor = {
                            target: key,
                            root: pipeline.targets[key],
                            description: <div
                                style={{
                                    display: 'flex',
                                    flexDirection: 'row',
                                    alignItems: 'center'
                                }}
                            >
                                <p>Action that run when launched in {descriptions[key]} (</p>
                                {<IconForTarget target={key as PipelineTarget} />}
                                <p>) mode.</p>
                            </div>
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

                    console.log('config errors to pass: ', config_errors);

                    const tabs = [
                        {
                            title: 'General',
                            content: general(state.container),
                            id: 'general',
                        }
                        ,
                        ...allTargets.map((kv) => {
                            return {
                                title: kv.target,
                                content: (
                                    <ConfigErrorContext.Provider value={config_errors} >
                                        <PipelineTargetDisplay root={kv.root} description={kv.description} />
                                    </ConfigErrorContext.Provider>
                                ),
                                id: kv.target.toLowerCase(),
                            };
                        }),
                    ];


                    return <Focusable
                        style={{ minWidth: "100%", minHeight: "100%" }}
                        onSecondaryActionDescription={secondaryActionDescription}
                        onSecondaryButton={secondaryAction}
                    >
                        <div style={{
                            marginTop: "40px",
                            height: "calc(100% - 40px)",
                            display: 'flex',
                            flexDirection: 'column'
                        }}>
                            <PipelineHeader containerRef={container} children={header(state.container)} />

                            <div style={{
                                // marginTop: "160px",
                                height: `calc(100% - 40px - ${headerHeight}px)`,
                                maxHeight: `calc(100% - 40px - ${headerHeight}px)`
                            }}>
                                <Tabs
                                    activeTab={currentTabRoute}
                                    onShowTab={(tabID: string) => {
                                        setCurrentTabRoute(tabID);
                                    }}
                                    tabs={tabs}
                                />
                            </div>
                        </div>
                    </Focusable >
                }
            }
        />
    );
}