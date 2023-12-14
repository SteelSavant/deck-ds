import { DialogBody, DialogControlsSection, Dropdown, Field, Focusable, Toggle } from "decky-frontend-lib";
import { ReactElement } from "react";
import { FaLink } from "react-icons/fa";
import { Action, ActionOneOf, ActionSelection, } from "../../backend";
import EditAction from "../../components/EditAction";
import { useModifiablePipelineDefinition } from "../../context/modifiablePipelineContext";
import { PipelineAction } from "../../types/backend_api";

export default function PipelineTargetDisplay({ root, description }: {
    root: ActionSelection,
    description: string
}): ReactElement {
    return (
        <DialogBody>
            <DialogControlsSection>
                <Field focusable={false} description={description} />
                {buildSelection('root', root, false)}
            </DialogControlsSection>
        </DialogBody>
    )
}

function buildSelection(id: string, selection: ActionSelection, shouldIndent: boolean): ReactElement {
    switch (selection.type) {
        case "Action":
            return buildAction(id, selection.value);
        case "OneOf":
            return buildOneOf(selection.value, shouldIndent);
        case "AllOf":
            return buildAllOf(selection.value, shouldIndent);
    }
}

function buildAction(id: string, action: Action): ReactElement {
    const { dispatch } = useModifiablePipelineDefinition();

    return (
        <div style={{
            paddingLeft: getIndent(true)
        }}>
            <EditAction action={action} onChange={(updatedAction) => {
                console.log('updating action from', action, 'to', updatedAction);
                dispatch({
                    type: 'updateAction',
                    id: id,
                    action: updatedAction,
                });
            }} />
        </div>
    )
}

function buildOneOf(oneOf: ActionOneOf, shouldIndent: boolean): ReactElement {
    const action = oneOf.actions.find((a) => a.id === oneOf.selection)!;
    return buildPipelineAction(action, shouldIndent);
}

function buildAllOf(allOf: PipelineAction[], shouldIndent: boolean): ReactElement {
    return (
        <div style={{
            paddingLeft: getIndent(shouldIndent)
        }}>
            {allOf.map((action) => buildPipelineAction(action, true))}
        </div>
    );
}

function buildPipelineAction(action: PipelineAction, shouldIndent: boolean): ReactElement {
    const { dispatch } = useModifiablePipelineDefinition();

    const selection = action.selection;
    const isEnabled = action.enabled;

    const forcedEnabled = isEnabled === null || isEnabled === undefined;
    return (
        <div style={{ flexDirection: 'row', paddingLeft: getIndent(shouldIndent) }}>
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
                                        actions: selection.value.actions.map((a) => a.id)
                                    })
                                }} />
                            </Focusable>
                            : <div />
                    }
                </div>
            </Field>
            {forcedEnabled || isEnabled ? buildSelection(action.id, action.selection, selection.type === 'OneOf') : <div />}
        </div>
    )
}

function getIndent(shouldIndent: boolean): string {
    return shouldIndent ? '30px' : '0px';
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