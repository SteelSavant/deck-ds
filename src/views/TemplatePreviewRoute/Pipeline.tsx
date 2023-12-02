import { Field, Toggle } from "decky-frontend-lib";
import { ReactElement } from "react";
import { FaLink } from "react-icons/fa";
import { Action, ActionEnabled, ActionSelection, OneOf, PipelineAction } from "../../backend";

export default function Pipeline({ root, }: { root: ActionSelection }): ReactElement {
    return buildSelection(root, 0)
}

const paddingIncr = 30;

function buildSelection(selection: ActionSelection, depth: number): ReactElement {
    switch (selection.type) {
        case "Action":
            return buildAction(selection.value, depth);
        case "OneOf":
            return buildOneOf(selection.value, depth);
        case "AllOf":
            return buildAllOf(selection.value, depth);
    }
}

function buildAction(action: Action, depth: number): ReactElement {
    return (
        <div style={{
            paddingLeft: `${depth * paddingIncr}px`
        }}>
            <p>Action: {action.type}</p>
        </div>
    )
}

function buildOneOf(oneOf: OneOf, depth: number): ReactElement {
    const action = oneOf.actions.find((a) => a.id === oneOf.selection)!;
    return (
        <div style={{
            paddingLeft: `${depth * paddingIncr}px`
        }}>
            <Field focusable={true} label={action.name} description={action.description} >
                {displayLinked(action)}
            </Field>
            {buildSelection(action.selection, depth)}
        </div>
    )
}

function buildAllOf(allOf: ActionEnabled[], depth: number): ReactElement {
    return (
        <div style={{
            paddingLeft: `${depth * paddingIncr}px`
        }}>
            {
                allOf.map((enabled) => {
                    const action = enabled.selection;
                    const isEnabled = enabled.enabled;
                    return (
                        <div>
                            <Field focusable={true} label={action.name} description={action.description} >
                                {displayLinked(action)}
                                {isEnabled === null || isEnabled === undefined
                                    ? <div />
                                    : <Toggle value={isEnabled} />
                                }
                            </Field>
                            {buildSelection(action.selection, depth + 1)}
                        </div>
                    )
                })}
        </div>
    );
}


function displayLinked(action: PipelineAction): ReactElement {
    return action.id.split(':').length === 3 && action.selection.type !== 'AllOf' ?
        <FaLink style={{
            padding: '15px'
        }} />
        : <div />
}