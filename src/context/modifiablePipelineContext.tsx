import * as React from 'react';
import { PipelineContainer } from '../backend';
import { PipelineUpdate, patchPipeline } from '../util/patch_pipeline';

interface PipelineContainerState {
    container: PipelineContainer,
}

type ProfileUpdate = {
    type: 'updateTags',
    tags: string[]
};


export interface StateAction {
    update: PipelineUpdate | ProfileUpdate,
}

type Dispatch = (action: StateAction) => void

type UpdatePipeline = (pipelineSettings: PipelineContainer) => void;

type ModifiablePipelineContextProviderProps = {
    children: React.ReactNode,
    initialContainer: PipelineContainer,
    onPipelineUpdate?: UpdatePipeline,
}

const ModifiablePipelineContainerStateContext = React.createContext<
    { state: PipelineContainerState; dispatch: Dispatch } | undefined
>(undefined)


function modifiablePipelineContainerReducerBuilder(onUpdate?: UpdatePipeline): (state: PipelineContainerState, action: StateAction) => PipelineContainerState {
    function modifiablePipelineContainerReducer(state: PipelineContainerState, action: StateAction): PipelineContainerState {
        console.log('handling modifiable pipeline dispatch for', action);


        const newContainer: PipelineContainer = (() => {
            const pipeline = state.container.pipeline;

            const updateType = action.update.type;
            if (updateType === 'updateTags') {
                return {
                    ...state.container,
                    tags: action.update.tags
                }
            } else {
                const newPipeline = patchPipeline(pipeline, action.update);
                return {
                    ...state.container,
                    pipeline: newPipeline,
                }
            }
        })();

        console.log('should perform pipeline update:', onUpdate)

        if (onUpdate) {
            console.log('performing pipeline update');

            onUpdate(newContainer); // perform arbitrary action, like saving, when the definition changes
        }

        return {
            container: newContainer,
        }
    }
    return modifiablePipelineContainerReducer;
}

function ModifiablePipelineContainerProvider({ children, initialContainer, onPipelineUpdate }: ModifiablePipelineContextProviderProps) {
    const [state, dispatch] = React.useReducer(modifiablePipelineContainerReducerBuilder(onPipelineUpdate), {
        container: initialContainer,
    });

    console.log('modifiable pipeline', state.container.pipeline);

    const value = { state, dispatch };
    return (
        <ModifiablePipelineContainerStateContext.Provider value={value}>
            {children}
        </ModifiablePipelineContainerStateContext.Provider>
    );
}

function useModifiablePipelineContainer() {
    const context = React.useContext(ModifiablePipelineContainerStateContext)
    if (context === undefined) {
        throw new Error('useSettings must be used within a SettingsProvider')
    }
    return context
}

export { ModifiablePipelineContainerProvider, useModifiablePipelineContainer };


