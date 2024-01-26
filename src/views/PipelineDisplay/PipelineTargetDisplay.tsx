import { DialogBody, DialogButton, DialogControlsSection, Dropdown, Field, Focusable, Toggle } from "decky-frontend-lib";
import { Fragment, ReactElement } from "react";
import { FaEye, FaEyeSlash } from "react-icons/fa";
import { Action, ActionOneOf, ActionSelection, PipelineAction, } from "../../backend";
import ActionIcon from "../../components/ActionIcon";
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

function buildSelection(id: string, selection: ActionSelection, indentLevel: number): ReactElement | null {
    switch (selection.type) {
        case "Action":
            return buildAction(id, selection.value, indentLevel);
        case "OneOf":
            return buildOneOf(selection.value, indentLevel);
        case "AllOf":
            return buildAllOf(selection.value, indentLevel);
    }
}

function buildAction(id: string, action: Action, indentLevel: number): ReactElement | null {
    const { dispatch } = useModifiablePipelineContainer();

    return EditAction({
        action: action, indentLevel: indentLevel + 1, onChange: (updatedAction) => {
            dispatch(
                {
                    update: {
                        type: 'updateAction',
                        id: id,
                        action: updatedAction,
                    }
                });
        }
    }
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

    const toggleQAMVisible = (action: PipelineAction) => {
        dispatch({
            update: {
                type: 'updateVisibleOnQAM',
                id: action.id,
                visible: !action.is_visible_on_qam
            }
        })
    }

    const built = forcedEnabled || isEnabled ? buildSelection(action.id, action.selection, selection.type === 'OneOf' ? indentLevel = + 1 : indentLevel) : <div />;
    console.log(built?.props);
    return (
        <Fragment>
            <Field
                indentLevel={indentLevel}
                focusable={forcedEnabled && selection.type !== 'OneOf'}
                label={action.name}
                description={action.description}
                icon={<ActionIcon action={action} />}

            >
                <div style={{ display: 'flex', flexDirection: 'row', justifyContent: 'space-between', alignItems: 'center', }}>
                    {
                        [
                            forcedEnabled ? null
                                : <Focusable>
                                    <Toggle value={isEnabled} onChange={(value) =>
                                        dispatch({
                                            update: {
                                                type: 'updateEnabled',
                                                id: action.id,
                                                isEnabled: value,
                                            }
                                        })
                                    } />
                                </Focusable>,
                            selection.type === 'OneOf'
                                ? <Focusable >
                                    <Dropdown selectedOption={selection.value.selection} rgOptions={selection.value.actions.map((a) => {
                                        return {
                                            label: a.name,
                                            data: a.id
                                        }
                                    })} onChange={(option) => {
                                        dispatch({
                                            update: {
                                                type: 'updateOneOf',
                                                id: action.id,
                                                selection: option.data,
                                            }
                                        })
                                    }} />
                                </Focusable>
                                : null,
                            selection.type !== 'AllOf' && built
                                ? < Focusable >
                                    <DialogButton style={{
                                        width: 'fit-content',
                                        minWidth: 'fit-content',
                                        height: 'fit-content',
                                        padding: '10px 12px'
                                    }}
                                        onClick={() => toggleQAMVisible(action)}
                                        onOKButton={() => toggleQAMVisible(action)}
                                    >
                                        {
                                            action.is_visible_on_qam
                                                ? <FaEye />
                                                : <FaEyeSlash />
                                        }
                                    </DialogButton>
                                </Focusable>
                                : null,

                        ].filter((x) => x)
                            .map((x) => <div style={{ marginRight: '10px' }}>
                                {x}
                            </div>)
                    }
                </div>
            </Field>
            {built}
        </Fragment >
    )
}
