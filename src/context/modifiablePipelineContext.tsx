import * as React from 'react';
import { ApiError, PipelineContainer } from '../backend';
import { logger } from '../util/log';
import { PipelineUpdate, patchPipeline } from '../util/patchPipeline';
import { Ok, Result } from '../util/result';

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

type Dispatch = (action: StateAction) => Promise<Result<void, ApiError>>;

type UpdatePipeline = (
    pipelineSettings: PipelineContainer,
) => Promise<Result<void, ApiError>>;

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

    async function dispatch(
        action: StateAction,
    ): Promise<Result<void, ApiError>> {
        logger.trace('starting dispatch');

        const newContainer: Result<PipelineContainer, ApiError> =
            await (async () => {
                const pipeline = state.container.pipeline;

                const updateType = action.update.type;
                if (updateType === 'updateTags') {
                    return Ok({
                        ...state.container,
                        tags: action.update.tags,
                    });
                } else {
                    const newPipeline = await patchPipeline(
                        pipeline,
                        action.update,
                    );

                    return newPipeline.map((pipeline) => {
                        return {
                            ...state.container,
                            pipeline,
                        };
                    });
                }
            })();

        logger.debug('got container state', newContainer);

        var v: void;
        let res: Result<void, ApiError> = Ok(v);

        return newContainer.andThenAsync(async (newContainer) => {
            if (onPipelineUpdate) {
                res = await onPipelineUpdate(newContainer); // perform arbitrary action, like saving, when the definition changes
            }

            if (res.isOk) {
                logger.debug('setting container state to', newContainer);

                setState({ container: newContainer });
            }

            return res;
        });
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
