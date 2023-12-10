import _ from 'lodash';
import * as React from 'react';
import { DefinitionOneOf } from '../backend';
import { Action, PipelineActionDefinition, PipelineDefinition } from "../types/backend_api";

type State = {
    definition: PipelineDefinition,
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
} | {
    type: 'updateAction',
    id: string,
    action: Action
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
        const cloneFn = (value: any): any => {
            if (value && value.id && value.id === action.id) {
                const type = action.type;
                let cloned = _.cloneDeep(value) as PipelineActionDefinition; // TODO::consider proper type narrowing
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
                                actions: (cloned.selection as unknown as DefinitionOneOf).actions // TODO::consider proper type narrowing
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

                return cloned;
            }
        };
        const newDefinition = _.cloneDeepWith(state.definition, cloneFn);

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
