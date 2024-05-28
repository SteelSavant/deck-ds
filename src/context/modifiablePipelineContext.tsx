import * as React from 'react';
import { PipelineContainer } from '../backend';
import { logger } from '../util/log';
import { PipelineUpdate, patchPipeline } from '../util/patchPipeline';

interface PipelineContainerState {
    container: PipelineContainer;
}

type ProfileUpdate = {
    type: 'updateTags';
    tags: string[];
};

export interface StateAction {
    update: PipelineUpdate | ProfileUpdate;
}

type Dispatch = (action: StateAction) => Promise<void>;

type UpdatePipeline = (pipelineSettings: PipelineContainer) => void;

type ModifiablePipelineContextProviderProps = {
    children: React.ReactNode;
    initialContainer: PipelineContainer;
    onPipelineUpdate?: UpdatePipeline;
};

const ModifiablePipelineContainerStateContext = React.createContext<{
    state: PipelineContainerState;
    dispatch: Dispatch;
} | null>(null);

function ModifiablePipelineContainerProvider({
    children,
    initialContainer,
    onPipelineUpdate,
}: ModifiablePipelineContextProviderProps) {
    const [state, setState] = React.useState({ container: initialContainer });

    logger.debug('modifiable pipeline', state.container.pipeline);

    async function dispatch(action: StateAction) {
        logger.trace('starting dispatch');

        const newContainer: PipelineContainer = await (async () => {
            const pipeline = state.container.pipeline;

            const updateType = action.update.type;
            if (updateType === 'updateTags') {
                return {
                    ...state.container,
                    tags: action.update.tags,
                };
            } else {
                const newPipeline = await patchPipeline(
                    pipeline,
                    action.update,
                );
                if (newPipeline.isOk) {
                    return {
                        ...state.container,
                        pipeline: newPipeline.data,
                    };
                }
                throw newPipeline.err;
            }
        })();

        logger.debug('got container state', newContainer);

        if (onPipelineUpdate) {
            await onPipelineUpdate(newContainer); // perform arbitrary action, like saving, when the definition changes
        }

        logger.debug('setting container state to', newContainer);

        setState({ container: newContainer });
    }

    const value = { state, dispatch };
    return (
        <ModifiablePipelineContainerStateContext.Provider value={value}>
            {children}
        </ModifiablePipelineContainerStateContext.Provider>
    );
}

function useModifiablePipelineContainer() {
    const context = React.useContext(ModifiablePipelineContainerStateContext);
    if (context === null) {
        throw new Error('useSettings must be used within a SettingsProvider');
    }
    return context;
}

export { ModifiablePipelineContainerProvider, useModifiablePipelineContainer };
