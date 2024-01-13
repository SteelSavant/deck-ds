import { DialogBody, DialogControlsSection, Dropdown, Field, Focusable, Toggle } from "decky-frontend-lib";
import { Fragment, ReactElement } from "react";
import { FaLink } from "react-icons/fa";
import { Action, ActionOneOf, ActionSelection, PipelineAction, } from "../../backend";
import EditAction from "../../components/EditAction";
import { useModifiablePipelineContainer } from "../../context/modifiablePipelineContext";

export default function PipelineTargetDisplay({ root, description }: {
    root: ActionSelection,
    description: string
}): ReactElement {
    return (
        <DialogBody>
            <DialogControlsSection>
                <Field focusable={false} description={description} />
                {buildSelection('root', root, root.type === 'AllOf' ? -1 : 0)}
            </DialogControlsSection>
        </DialogBody>
    )
}

function buildSelection(id: string, selection: ActionSelection, indentLevel: number): ReactElement {
    switch (selection.type) {
        case "Action":
            return buildAction(id, selection.value, indentLevel);
        case "OneOf":
            return buildOneOf(selection.value, indentLevel);
        case "AllOf":
            return buildAllOf(selection.value, indentLevel);
    }
}

function buildAction(id: string, action: Action, indentLevel: number): ReactElement {
    const { dispatch } = useModifiablePipelineContainer();

    return (
        <EditAction action={action} indentLevel={indentLevel + 1} onChange={(updatedAction) => {
            console.log('updating action from', action, 'to', updatedAction);
            dispatch({
                type: 'updateAction',
                id: id,
                action: updatedAction,
            });
        }} />
    )
}

function buildOneOf(oneOf: ActionOneOf, indentLevel: number): ReactElement {
    const action = oneOf.actions.find((a) => a.id === oneOf.selection)!;
    return buildPipelineAction(action, indentLevel + 1);
}

function buildAllOf(allOf: PipelineAction[], indentLevel: number): ReactElement {
    return (
        <Fragment>
            {allOf.map((action) => buildPipelineAction(action, indentLevel + 1))}
        </Fragment>
    );
}

function buildPipelineAction(action: PipelineAction, indentLevel: number): ReactElement {
    const { dispatch } = useModifiablePipelineContainer();

    const selection = action.selection;
    const isEnabled = action.enabled;

    const forcedEnabled = isEnabled === null || isEnabled === undefined;
    return (
        <div style={{ flexDirection: 'row' }}>
            <Field indentLevel={indentLevel} focusable={forcedEnabled && selection.type !== 'OneOf'} label={labelAction(action)} description={action.description}>
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
                                        actions: selection.value.actions.map((a) => a.id)
                                    })
                                }} />
                            </Focusable>
                            : <div />
                    }
                </div>
            </Field>
            {forcedEnabled || isEnabled ? buildSelection(action.id, action.selection, selection.type === 'OneOf' ? indentLevel = + 1 : indentLevel) : <div />}
        </div>
    )
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