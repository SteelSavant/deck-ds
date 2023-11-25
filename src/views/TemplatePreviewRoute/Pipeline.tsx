import { ReactElement } from "react";
import { ActionSelection, AllOfSelection, OneOfSelection, PipelineActionDefinition, Selection, isAction, isAllOf, isOneOf } from "../../backend";

export default function Pipeline({ root, actions }: { root: Selection, actions: { [key: string]: PipelineActionDefinition } }): ReactElement {
    return buildSelection(root, actions, isAllOf(root) ? -1 : 1)
}


function buildSelection(selection: Selection, actions: { [key: string]: PipelineActionDefinition }, depth: number): ReactElement {
    if (isAction(selection)) {
        return buildAction(selection, depth);
    } else if (isOneOf(selection)) {
        return buildOneOf(selection, actions, depth)
    } else if (isAllOf(selection)) {
        return buildAllOf(selection, actions, depth);
    } else {
        console.log("unknown selection variant", selection);
        return <div />
    }

}

function buildAction(action: ActionSelection, depth: number): ReactElement {
    return (
        <div style={{
            paddingLeft: `${depth * 15}px`
        }}>
            <p>Action: {action.Action}</p>
        </div>
    )
}

function buildOneOf(oneOf: OneOfSelection, actions: { [key: string]: PipelineActionDefinition }, depth: number): ReactElement {
    const action = actions[oneOf.OneOf.selection];
    const child = buildSelection(action.selection, actions, depth);
    return (
        <div style={{
            paddingLeft: `${depth * 15}px`
        }}>
            <h3>{action.name}</h3>
            <p>{action.description}</p>
            {child}
        </div>
    )
}

function buildAllOf(allOf: AllOfSelection, actions: { [key: string]: PipelineActionDefinition }, depth: number): ReactElement {

    return (
        <div style={{
            paddingLeft: `${depth * 15}px`
        }}> {
                allOf.AllOf.map((enabled) => {
                    const action = actions[enabled.selection];
                    const child = buildSelection(action.selection, actions, depth + 1);
                    return (
                        <div>
                            <h3>{action.name}</h3>
                            <p>{action.description}</p>
                            {child}
                        </div>
                    )
                })}
        </div>
    );
}
