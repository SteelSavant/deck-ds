import { ReactElement } from "react";
import { Action, ActionEnabled, ActionSelection, OneOf } from "../../backend";

export default function Pipeline({ root, }: { root: ActionSelection }): ReactElement {
    return buildSelection(root, root.type === "AllOf" ? -1 : 1)
}


function buildSelection(selection: ActionSelection, depth: number): ReactElement {
    if (selection.type === "Action") {
        return buildAction(selection.value, depth);
    } else if (selection.type === "OneOf") {
        return buildOneOf(selection.value, depth);
    }
    else if (selection.type === "AllOf") {
        return buildAllOf(selection.value, depth);
    }
    else {
        const typecheck: never = selection;
        throw typecheck;
    }
}

function buildAction(action: Action, depth: number): ReactElement {
    return (
        <div style={{
            paddingLeft: `${depth * 15}px`
        }}>
            <p>Action: {action}</p>
        </div>
    )
}

function buildOneOf(oneOf: OneOf, depth: number): ReactElement {
    const action = oneOf.actions.find((a) => a.id === oneOf.selection)!;
    const child = buildSelection(action.selection, depth);
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

function buildAllOf(allOf: ActionEnabled[], depth: number): ReactElement {

    return (
        <div style={{
            paddingLeft: `${depth * 15}px`
        }}> {
                allOf.map((enabled) => {
                    const action = enabled.selection;
                    const child = buildSelection(action.selection, depth + 1);
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
