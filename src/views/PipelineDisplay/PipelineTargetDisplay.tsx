import { DialogBody, DialogControlsSection, Dropdown, Field, Focusable, Toggle } from "decky-frontend-lib";
import { Fragment, ReactElement } from "react";
import { Action, ActionOneOf, ActionSelection, PipelineAction, } from "../../backend";
import ActionIcon from "../../components/ActionIcon";
import EditAction from "../../components/EditAction";
import { useModifiablePipelineContainer } from "../../context/modifiablePipelineContext";
import { MaybeString } from "../../types/short";

export default function PipelineTargetDisplay({ root, description }: {
    root: ActionSelection,
    description: string
}): ReactElement {
    return (
        <DialogBody>
            <DialogControlsSection>
                <Field focusable={false} description={description} />
                {buildSelection('root', null, root, root.type === 'AllOf' ? -1 : 0)}
            </DialogControlsSection>
        </DialogBody>
    )
}

function buildSelection(id: string, externalProfile: MaybeString, selection: ActionSelection, indentLevel: number): ReactElement {
    switch (selection.type) {
        case "Action":
            return buildAction(id, externalProfile, selection.value, indentLevel);
        case "OneOf":
            return buildOneOf(selection.value, indentLevel);
        case "AllOf":
            return buildAllOf(selection.value, indentLevel);
    }
}

function buildAction(id: string, externalProfile: MaybeString, action: Action, indentLevel: number): ReactElement {
    const { dispatch } = useModifiablePipelineContainer();

    return (
        <EditAction action={action} indentLevel={indentLevel + 1} onChange={(updatedAction) => {
            dispatch(
                {
                    externalProfile: externalProfile,
                    update: {
                        type: 'updateAction',
                        id: id,
                        action: updatedAction,
                    }
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
            <Field
                indentLevel={indentLevel}
                focusable={forcedEnabled && selection.type !== 'OneOf'}
                label={action.name}
                description={action.description}
                icon={<ActionIcon action={action} />}
            >
                <div style={{ paddingRight: '10px' }}>
                    {
                        forcedEnabled ? <div />
                            : <Focusable>
                                <Toggle value={isEnabled} onChange={(value) =>
                                    dispatch({
                                        externalProfile: action.profile_override,
                                        update: {
                                            type: 'updateEnabled',
                                            id: action.id,
                                            isEnabled: value,
                                        }
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
                                        externalProfile: action.profile_override,
                                        update: {
                                            type: 'updateOneOf',
                                            id: action.id,
                                            selection: option.data,
                                            actions: selection.value.actions.map((a) => a.id)
                                        }
                                    })
                                }} />
                            </Focusable>
                            : <div />
                    }
                </div>
            </Field>
            {forcedEnabled || isEnabled ? buildSelection(action.id, action.profile_override, action.selection, selection.type === 'OneOf' ? indentLevel = + 1 : indentLevel) : <div />}
        </div>
    )
}
