import _ from 'lodash';
import * as React from 'react';
import { Action, PipelineActionSettings, PipelineContainer, PipelineDefinition } from '../backend';

type PipelineContainerState = {
    container: PipelineContainer,
}

interface PipelineInfo {
    description: string | undefined;
    name: string | undefined;
}

type StateAction = {
    type: 'updateEnabled',
    id: string,
    isEnabled: boolean
} | {
    type: 'updateProfileOverride',
    id: string,
    profileOverride: string | null | undefined
} | {
    type: 'updateOneOf',
    id: string,
    selection: string,
    actions: string[],
} | {
    type: 'updateAction',
    id: string,
    action: Action
} | {
    type: 'updatePipelineInfo',
    info: PipelineInfo,
} | {
    type: 'updateTags',
    tags: string[]
};

type Dispatch = (action: StateAction) => void

type ExternalPipelineUpdate = (pipelineSettings: PipelineContainer) => void;

type ModifiablePipelineContextProviderProps = {
    children: React.ReactNode,
    initialContainer: PipelineContainer,
    onUpdate?: ExternalPipelineUpdate
}

const ModifiablePipelineContainerStateContext = React.createContext<
    { state: PipelineContainerState; dispatch: Dispatch } | undefined
>(undefined)



function modifiablePipelineContainerReducerBuilder(onUpdate?: ExternalPipelineUpdate): (state: PipelineContainerState, action: StateAction) => PipelineContainerState {
    function modifiablePipelineContainerReducer(state: PipelineContainerState, action: StateAction): PipelineContainerState {
        const newContainer: PipelineContainer = (() => {
            const pipeline = state.container.pipeline;
            if (action.type === 'updatePipelineInfo') {
                const newDefinition: PipelineDefinition = {
                    ...pipeline,
                };

                const info = action.info;

                if (info.description) {
                    newDefinition.description = info.description
                }

                if (info.name) {
                    newDefinition.name = info.name
                }


                return {
                    ...state.container,
                    pipeline: newDefinition
                };
            } else if (action.type === 'updateTags') {
                return {
                    ...state.container,
                    tags: action.tags
                }
            } else {
                let updatedActions: { [k: string]: PipelineActionSettings } = {};
                let currentActions = pipeline.actions.actions;
                for (let key in currentActions) {
                    if (key === action.id) {
                        let cloned = _.cloneDeep(currentActions[key]);
                        const type = action.type;

                        switch (type) {
                            case 'updateEnabled':
                                cloned.enabled = action.isEnabled;
                                break;
                            case 'updateAction':
                                cloned.selection = {
                                    type: 'Action',
                                    value: action.action
                                };
                                break;
                            case 'updateOneOf':
                                if (cloned.selection.type != 'OneOf') {
                                    throw 'Invalid selection type for updateOneOf';
                                }

                                cloned.selection = {
                                    type: 'OneOf',
                                    value: {
                                        selection: action.selection,
                                        actions: action.actions,
                                    }
                                }
                                break;
                            case 'updateProfileOverride':
                                cloned.profile_override = action.profileOverride
                                break;
                            default:
                                const typecheck: never = type;
                                throw typecheck ?? 'action update failed to typecheck';
                        }

                        updatedActions[key] = cloned;
                    } else {
                        updatedActions[key] = currentActions[key];
                    }

                }
                let result: PipelineContainer = {
                    ...state.container,
                    pipeline: {
                        ...state.container.pipeline,
                        actions: {
                            actions: updatedActions
                        }
                    },
                };
                return result;
            }
        })();

        if (onUpdate) {
            onUpdate(newContainer); // perform arbitrary action, like saving, when the definition changes
        }

        return {
            container: newContainer
        }
    }

    return modifiablePipelineContainerReducer;
}

function ModifiablePipelineContainerProvider({ children, initialContainer, onUpdate, }: ModifiablePipelineContextProviderProps) {
    const [state, dispatch] = React.useReducer(modifiablePipelineContainerReducerBuilder(onUpdate), {
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
