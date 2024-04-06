import * as React from 'react';
import { PipelineContainer } from '../backend';
import { PipelineUpdate, patchPipeline } from "../util/patchPipeline";

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

type Dispatch = (action: StateAction) => Promise<void>

type UpdatePipeline = (pipelineSettings: PipelineContainer) => void;

type ModifiablePipelineContextProviderProps = {
    children: React.ReactNode,
    initialContainer: PipelineContainer,
    onPipelineUpdate?: UpdatePipeline,
}

const ModifiablePipelineContainerStateContext = React.createContext<
    { state: PipelineContainerState; dispatch: Dispatch } | undefined
>(undefined)


// TODO::as there isn't transient state anymore (no template previews)
// Change this to do all the edits directly on the server.
// Make the patchPipeline function a server call.
function ModifiablePipelineContainerProvider({ children, initialContainer, onPipelineUpdate }: ModifiablePipelineContextProviderProps) {
    const [state, setState] = React.useState({ container: initialContainer });

    console.log('modifiable pipeline', state.container.pipeline);

    async function dispatch(action: StateAction) {
        const newContainer: PipelineContainer = await (async () => {
            const pipeline = state.container.pipeline;

            const updateType = action.update.type;
            if (updateType === 'updateTags') {
                return {
                    ...state.container,
                    tags: action.update.tags
                }
            } else {
                const newPipeline = await patchPipeline(pipeline, action.update);
                if (newPipeline.isOk) {
                    return {
                        ...state.container,
                        pipeline: newPipeline.data,
                    }
                }
                throw newPipeline.err;
            }
        })();

        setState({ container: newContainer })

        if (onPipelineUpdate) {
            console.log('performing pipeline update');

            await onPipelineUpdate(newContainer); // perform arbitrary action, like saving, when the definition changes
        }
    };


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


