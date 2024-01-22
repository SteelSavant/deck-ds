import * as React from 'react';
import { PipelineContainer } from '../backend';
import { MaybeString } from '../types/short';
import { PipelineUpdate, patchPipeline } from '../util/patch';

interface PipelineContainerState {
    container: PipelineContainer,
}

type ProfileUpdate = {
    type: 'updateTags',
    tags: string[]
};

interface StateAction {
    ///  if null patch self, else defer to exteral update using provided profile
    externalProfile: MaybeString,
    update: PipelineUpdate | ProfileUpdate,
}


type Dispatch = (action: StateAction) => void

type UpdatePipeline = (pipelineSettings: PipelineContainer) => void;
type UpdateExternalProfile = (profileId: string, update: PipelineUpdate) => void;

type ModifiablePipelineContextProviderProps = {
    children: React.ReactNode,
    initialContainer: PipelineContainer,
    onPipelineUpdate?: UpdatePipeline,
    onExternalProfileUpdate?: UpdateExternalProfile,
}

const ModifiablePipelineContainerStateContext = React.createContext<
    { state: PipelineContainerState; dispatch: Dispatch } | undefined
>(undefined)



function modifiablePipelineContainerReducerBuilder(onPipelineUpdate?: UpdatePipeline, onExternalProfileUpdate?: UpdateExternalProfile): (state: PipelineContainerState, action: StateAction) => PipelineContainerState {
    function modifiablePipelineContainerReducer(state: PipelineContainerState, action: StateAction): PipelineContainerState {
        // defer updates to external profiles, to avoid complexity of local state
        if (action.externalProfile) {
            if (action.update.type != 'updateTags' && onExternalProfileUpdate) {
                onExternalProfileUpdate(action.externalProfile, action.update)
            }
            return state;
        }

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

        if (onPipelineUpdate) {
            onPipelineUpdate(newContainer); // perform arbitrary action, like saving, when the definition changes
        }

        return {
            container: newContainer
        }
    }

    return modifiablePipelineContainerReducer;
}

function ModifiablePipelineContainerProvider({ children, initialContainer, onPipelineUpdate, onExternalProfileUpdate }: ModifiablePipelineContextProviderProps) {
    const [state, dispatch] = React.useReducer(modifiablePipelineContainerReducerBuilder(onPipelineUpdate, onExternalProfileUpdate), {
        container: initialContainer,
    });

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
