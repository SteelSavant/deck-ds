import { DialogBody, DialogControlsSection, Dropdown, Field, Focusable, Toggle } from "decky-frontend-lib";
import { ReactElement } from "react";
import { FaLink } from "react-icons/fa";
import { Action, ActionEnabled, ActionSelection, OneOf, PipelineAction } from "../../backend";

type ActionUpdate = (args: {
    id: string,
    value: Action
}) => void;

type OneOfUpdate = (args:
    {
        id: string,
        selection: string
    }) => void;

interface Updates {
    action: ActionUpdate,
    oneOf: OneOfUpdate,
}

export default function Pipeline({ root, updateAction, updateOneOf, }: {
    root: ActionSelection,
    updateAction: ActionUpdate,
    updateOneOf: OneOfUpdate
}): ReactElement {
    return <DialogBody>
        <DialogControlsSection>
            {buildSelection('root', root, {
                action: updateAction,
                oneOf: updateOneOf
            }, 0)}
        </DialogControlsSection>
    </DialogBody>
}

const paddingIncr = 30;

function buildSelection(id: string, selection: ActionSelection, updates: Updates, depth: number): ReactElement {
    switch (selection.type) {
        case "Action":
            return buildAction(id, selection.value, updates, depth);
        case "OneOf":
            return buildOneOf(id, selection.value, updates, depth);
        case "AllOf":
            return buildAllOf(selection.value, updates, depth);
    }
}

function buildAction(id: string, action: Action, updates: Updates, depth: number): ReactElement {
    return (
        <div style={{
            paddingLeft: `${depth * paddingIncr}px`
        }}>
            <p>Action: {action.type}</p>
        </div>
    )
}

function buildOneOf(id: string, oneOf: OneOf, updates: Updates, depth: number): ReactElement {
    const action = oneOf.actions.find((a) => a.id === oneOf.selection)!;
    return (
        <div style={{
            paddingLeft: `${depth * paddingIncr}px`
        }}>
            <Field focusable={false} label={labelAction(action)} description={action.description} />
            {buildSelection(action.id, action.selection, updates, 0)}
        </div>
    )
}

function buildAllOf(allOf: ActionEnabled[], updates: Updates, depth: number): ReactElement {
    return (
        <div style={{
            paddingLeft: `${depth * paddingIncr}px`
        }}>
            {
                allOf.map((enabled) => {
                    const action = enabled.selection;
                    const isEnabled = enabled.enabled;
                    const selection = action.selection;

                    const forcedEnabled = isEnabled === null || isEnabled === undefined;
                    return (
                        <div style={{ flexDirection: 'row' }}>
                            <Field focusable={forcedEnabled && selection.type !== 'OneOf'} label={labelAction(action)} description={action.description}>
                                <div style={{ paddingRight: '10px' }}>
                                    {
                                        forcedEnabled ? <div />
                                            : <Focusable>
                                                <Toggle value={isEnabled} />
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
                                                })} />
                                            </Focusable>
                                            : <div />
                                    }
                                </div>
                            </Field>
                            {buildSelection(action.id, action.selection, updates, depth + 1)}
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