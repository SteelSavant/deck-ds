import _ from 'lodash';
import * as React from 'react';
import { PipelineContainer } from '../backend';
import { MaybeString } from '../types/short';
import { PipelineUpdate, patchPipeline } from '../util/patch';

interface PipelineContainerState {
    container: PipelineContainer,
    onPipelineUpdate?: UpdatePipeline,
    onExternalProfileUpdate?: UpdateExternalProfile
}

type ProfileUpdate = {
    type: 'updateTags',
    tags: string[]
};

type ContainerUpdate = {
    type: 'containerUpdate'
    container: PipelineContainer,
}

export interface StateAction {
    ///  if null patch self, else defer to exteral update using provided profile
    externalProfile: MaybeString,
    update: PipelineUpdate | ProfileUpdate | ContainerUpdate,
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


export function modifiablePipelineContainerReducer(state: PipelineContainerState, action: StateAction): PipelineContainerState {
    console.log('handling modifiable pipeline dispatch for', action);

    // defer updates to external profiles, to avoid complexity of local state
    if (action.externalProfile) {
        console.log('is external profile update');
        if (action.update.type !== 'updateTags' && action.update.type !== 'containerUpdate' && state.onExternalProfileUpdate) {
            console.log('performing external profile update');
            state.onExternalProfileUpdate(action.externalProfile, action.update)
        }
        return state;
    }

    console.log('is pipeline update');

    const newContainer: PipelineContainer = (() => {
        const pipeline = state.container.pipeline;

        const updateType = action.update.type;
        if (updateType === 'containerUpdate') {
            console.log('performing container update');
            return {
                ...action.update.container

            }
        } else if (updateType === 'updateTags') {
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

    console.log('should perform pipeline update:', state.onPipelineUpdate)

    if (state.onPipelineUpdate) {
        console.log('performing pipeline update');

        state.onPipelineUpdate(newContainer); // perform arbitrary action, like saving, when the definition changes
    }

    return {
        container: newContainer,
        onPipelineUpdate: state.onPipelineUpdate,
        onExternalProfileUpdate: state.onExternalProfileUpdate
    }
}

function ModifiablePipelineContainerProvider({ children, initialContainer, onPipelineUpdate, onExternalProfileUpdate }: ModifiablePipelineContextProviderProps) {
    const [state, dispatch] = React.useReducer(modifiablePipelineContainerReducer, {
        container: initialContainer,
        onExternalProfileUpdate,
        onPipelineUpdate
    });

    console.log('modifiable pipeline', state.container.pipeline);

    React.useEffect(() => {
        console.log('checking container effect')
        if (!_.isEqual(state.container.pipeline, initialContainer.pipeline)) {
            console.log('running container effect')

            dispatch({
                externalProfile: null,
                update: {
                    type: 'containerUpdate',
                    container: initialContainer
                }
            })
        }
    }, [initialContainer])

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
