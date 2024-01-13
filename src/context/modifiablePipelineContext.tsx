import _ from 'lodash';
import * as React from 'react';
import { Action, PipelineActionSettings, PipelineDefinition } from '../backend';

type State = {
    definition: PipelineDefinition,
}

interface PipelineInfo {
    description: string | undefined;
    name: string | undefined;
    tags: string[] | undefined;
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
    type: 'updatePipelineInfo'
    info: PipelineInfo,
};

type Dispatch = (action: StateAction) => void

type ExternalPipelineUpdate = (pipelineSettings: PipelineDefinition) => void;

type ModifiablePipelineContextProviderProps = {
    children: React.ReactNode,
    initialDefinition: PipelineDefinition,
    onUpdate?: ExternalPipelineUpdate
}

const ModifiablePipelineDefinitionStateContext = React.createContext<
    { state: State; dispatch: Dispatch } | undefined
>(undefined)



function modifiablePipelineDefinitionReducerBuilder(onUpdate?: ExternalPipelineUpdate): (state: State, action: StateAction) => State {
    function modifiablePipelineDefinitionReducer(state: State, action: StateAction): State {
        console.log('in pipeline reducer');

        const newDefinition: PipelineDefinition = (() => {
            if (action.type === 'updatePipelineInfo') {
                const newDefinition: PipelineDefinition = {
                    ...state.definition,
                };

                const info = action.info;

                if (info.description) {
                    newDefinition.description = info.description
                }

                if (info.name) {
                    newDefinition.name = info.name
                }


                return newDefinition;
            } else {
                let target = state.definition.targets["thing"];
                switch (target.type) {
                    case 'Action': target.value.type;
                        break;
                    case 'AllOf': target.value[0];
                        break;
                    case 'OneOf': target.value.actions
                }

                let updatedActions: { [k: string]: PipelineActionSettings } = {};
                let currentActions = state.definition.actions.actions;
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

                                console.log('updated pipeline action to', cloned.selection);
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
                    } else {
                        updatedActions[key] = currentActions[key];
                    }

                }

                return {
                    ...state.definition,
                    definition: {
                        ...state.definition,
                        actions: {
                            actions: updatedActions
                        }
                    },
                }
            }
        })();


        console.log('new definition from reducer:', newDefinition);

        if (onUpdate) {
            onUpdate(newDefinition); // perform arbitrary action, like saving, when the definition changes
        }

        return {
            definition: newDefinition
        }
    }

    return modifiablePipelineDefinitionReducer;
}

function ModifiablePipelineDefinitionProvider({ children, initialDefinition, onUpdate, }: ModifiablePipelineContextProviderProps) {
    const [state, dispatch] = React.useReducer(modifiablePipelineDefinitionReducerBuilder(onUpdate), {
        definition: initialDefinition,
    });

    const value = { state, dispatch };
    return (
        <ModifiablePipelineDefinitionStateContext.Provider value={value}>
            {children}
        </ModifiablePipelineDefinitionStateContext.Provider>
    );
}

function useModifiablePipelineDefinition() {
    const context = React.useContext(ModifiablePipelineDefinitionStateContext)
    if (context === undefined) {
        throw new Error('useSettings must be used within a SettingsProvider')
    }
    return context
}

export { ModifiablePipelineDefinitionProvider, useModifiablePipelineDefinition };
