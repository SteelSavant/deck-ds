import { Focusable, Tabs } from '@decky/ui';
import { ReactElement, useEffect, useRef, useState } from 'react';
import { PipelineContainer, RuntimeSelection } from '../../backend';
import HandleLoading from '../../components/HandleLoading';
import { ConfigErrorContext } from '../../context/configErrorContext';
import { useModifiablePipelineContainer } from '../../context/modifiablePipelineContext';
import useReifiedPipeline from '../../hooks/useReifiedPipeline';
import PipelineHeader from './PipelineHeader';
import PipelineTargetDisplay from './PipelineTargetDisplay';

interface PipelineDisplayProps {
    header: (container: PipelineContainer) => ReactElement;
    general: (container: PipelineContainer) => ReactElement;
    secondaryAction?: () => void;
    secondaryActionDescription?: string;
}

export default function PipelineDisplay({
    header,
    general,
    secondaryAction,
    secondaryActionDescription,
}: PipelineDisplayProps): ReactElement {
    const [currentTabRoute, setCurrentTabRoute] = useState<string>('general');

    const { state } = useModifiablePipelineContainer();
    const result = useReifiedPipeline(state.container.pipeline);

    let containerRef = useRef<HTMLDivElement>(null);
    let [headerHeight, setHeaderHeight] = useState<number | null>(null);

    useEffect(() => setHeaderHeight(containerRef?.current?.offsetHeight ?? 0));

    return (
        <HandleLoading
            value={result}
            onOk={({ pipeline, config_errors }) => {
                interface TargetDescriptor {
                    target: string;
                    root: RuntimeSelection;
                }

                const defaultTargets: TargetDescriptor[] = [];
                const extraTargets: TargetDescriptor[] = []; // no real intention of actually supporting extra targets, but...

                for (const key in pipeline.targets) {
                    const value: TargetDescriptor = {
                        target: key,
                        root: pipeline.targets[key],
                    };

                    if (key === 'Gamemode') {
                        defaultTargets.push(value);
                    } else if (key === 'Desktop') {
                        defaultTargets.splice(0, 0, value);
                    } else {
                        extraTargets.push(value);
                    }
                }

                const allTargets = defaultTargets.concat(extraTargets);
                const tabs = [
                    {
                        title: 'General',
                        content: general(state.container),
                        id: 'general',
                    },
                    ...allTargets.map((kv) => {
                        return {
                            title: kv.target,
                            content: (
                                <ConfigErrorContext.Provider
                                    value={config_errors}
                                >
                                    <PipelineTargetDisplay root={kv.root} />
                                </ConfigErrorContext.Provider>
                            ),
                            id: kv.target.toLowerCase(),
                        };
                    }),
                ];

                return (
                    <Focusable
                        style={{ minWidth: '100%', minHeight: '100%' }}
                        onSecondaryActionDescription={
                            secondaryActionDescription
                        }
                        onSecondaryButton={secondaryAction}
                    >
                        <div
                            style={{
                                marginTop: '40px',
                                height: 'calc(100% - 40px)',
                                display: 'flex',
                                flexDirection: 'column',
                            }}
                        >
                            <PipelineHeader
                                containerRef={containerRef}
                                children={header(state.container)}
                            />

                            <div
                                style={{
                                    // marginTop: "160px",
                                    height: `calc(100% - 40px - ${headerHeight}px)`,
                                    maxHeight: `calc(100% - 40px - ${headerHeight}px)`,
                                }}
                            >
                                <Tabs
                                    activeTab={currentTabRoute}
                                    onShowTab={(tabID: string) => {
                                        setCurrentTabRoute(tabID);
                                    }}
                                    tabs={tabs}
                                    // TODO::look into footer on tab for "Save as new profile" Action on templates page
                                />
                            </div>
                        </div>
                    </Focusable>
                );
            }}
        />
    );
}
