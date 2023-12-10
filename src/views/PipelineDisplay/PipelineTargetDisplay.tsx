import { DialogBody, DialogControlsSection, Dropdown, Field, Focusable, Toggle } from "decky-frontend-lib";
import { ReactElement } from "react";
import { FaLink } from "react-icons/fa";
import { Action, ActionOneOf, ActionSelection, } from "../../backend";
import EditAction from "../../components/EditAction";
import { useModifiablePipelineDefinition } from "../../context/modifiablePipelineContext";
import { PipelineAction } from "../../types/backend_api";

type ActionUpdate = (args: {
    id: string,
    value: Action
}) => void;

type OneOfUpdate = (args:
    {
        id: string,
        selection: string
    }) => void;



export default function PipelineTargetDisplay({ root }: {
    root: ActionSelection,
    updateAction: ActionUpdate,
    updateOneOf: OneOfUpdate
}): ReactElement {
    return <DialogBody>
        <DialogControlsSection>
            {buildSelection('root', root, 0)}
        </DialogControlsSection>
    </DialogBody>
}

const paddingIncr = 30;

function buildSelection(id: string, selection: ActionSelection, depth: number): ReactElement {
    switch (selection.type) {
        case "Action":
            return buildAction(id, selection.value, depth);
        case "OneOf":
            return buildOneOf(selection.value, depth);
        case "AllOf":
            return buildAllOf(selection.value, depth);
    }
}

function buildAction(id: string, action: Action, depth: number): ReactElement {
    const { dispatch } = useModifiablePipelineDefinition();

    return (
        <div style={{
            paddingLeft: `${depth * paddingIncr}px`
        }}>
            <EditAction action={action} onChange={(action) => {
                dispatch({
                    type: 'updateAction',
                    id: id,
                    action: action
                });
            }} />
        </div>
    )
}

function buildOneOf(oneOf: ActionOneOf, depth: number): ReactElement {
    const action = oneOf.actions.find((a) => a.id === oneOf.selection)!;
    return (
        <div style={{
            paddingLeft: `${depth * paddingIncr}px`
        }}>
            <Field focusable={false} label={labelAction(action)} description={action.description} />
            {buildSelection(action.id, action.selection, 0)}
        </div>
    )
}

function buildAllOf(allOf: PipelineAction[], depth: number): ReactElement {
    const { dispatch } = useModifiablePipelineDefinition();

    return (
        <div style={{
            paddingLeft: `${depth * paddingIncr}px`
        }}>
            {

                allOf.map((action) => {
                    const selection = action.selection;
                    const isEnabled = action.enabled;


                    const forcedEnabled = isEnabled === null || isEnabled === undefined;
                    return (
                        <div style={{ flexDirection: 'row' }}>
                            <Field focusable={forcedEnabled && selection.type !== 'OneOf'} label={labelAction(action)} description={action.description}>
                                <div style={{ paddingRight: '10px' }}>
                                    {
                                        forcedEnabled ? <div />
                                            : <Focusable>
                                                <Toggle value={isEnabled} onChange={(value) =>
                                                    dispatch({
                                                        type: 'updateEnabled',
                                                        id: action.id,
                                                        isEnabled: value,
                                                    })
                                                } />
                                            </Focusable>
                                    }
                                    {
                                        selection.type === 'OneOf' ?
                                            <Focusable >
                                                <Dropdown selectedOption={selection.value.selection} rgOptions={selection.value.actions.map((a) => {
                                                    return {
                                                        label: a.name,
                                                        data: a.id
                                                    }
                                                })} onChange={(option) => {
                                                    dispatch({
                                                        type: 'updateOneOf',
                                                        id: action.id,
                                                        selection: option.data,
                                                    })
                                                }} />
                                            </Focusable>
                                            : <div />
                                    }
                                </div>
                            </Field>
                            {buildSelection(action.id, action.selection, depth + 1)}
                        </div>
                    )
                })}
        </div>
    );
}


function labelAction(action: PipelineAction): ReactElement {
    return action.id.split(':').length === 3 && action.selection.type !== 'AllOf' ? <div>
        {action.name}
        <FaLink style={{
            paddingLeft: '10px',
            paddingRight: '10px'
        }} />
    </div>
        : <p>{action.name}</p>
}