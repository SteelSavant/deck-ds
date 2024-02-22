import { DialogBody, DialogButton, DialogControlsSection, Dropdown, Field, Focusable, Toggle } from "decky-frontend-lib";
import { Fragment, ReactElement, useContext } from "react";
import { FaEye, FaEyeSlash } from "react-icons/fa";
import { Action, ActionOneOf, ActionSelection, PipelineAction, } from "../../backend";
import ActionIcon from "../../components/ActionIcon";
import ConfigErrorWarning from "../../components/ConfigErrorWarning";
import { EditAction } from "../../components/EditAction";
import { ConfigErrorContext } from "../../context/configErrorContext";
import { useModifiablePipelineContainer } from "../../context/modifiablePipelineContext";

export default function PipelineTargetDisplay({ root, description }: {
    root: ActionSelection,
    description: ReactElement
}): ReactElement {
    return (
        <DialogBody>
            <DialogControlsSection>
                <Field focusable={false} description={description} />
                {buildSelection('root', root, root.type === 'AllOf' ? -1 : 0, false)}
            </DialogControlsSection>
        </DialogBody>
    )
}

function buildSelection(id: string, selection: ActionSelection, indentLevel: number, qamHiddenByParent: boolean): ReactElement | null {
    switch (selection.type) {
        case "Action":
            return buildAction(id, selection.value, indentLevel);
        case "OneOf":
            return buildOneOf(selection.value, indentLevel, qamHiddenByParent);
        case "AllOf":
            return buildAllOf(selection.value, indentLevel, qamHiddenByParent);
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
    })
}

function buildOneOf(oneOf: ActionOneOf, indentLevel: number, qamHiddenByParent: boolean): ReactElement {
    const action = oneOf.actions.find((a) => a.id === oneOf.selection)!;
    return buildPipelineAction(action, indentLevel + 1, qamHiddenByParent);
}

function buildAllOf(allOf: PipelineAction[], indentLevel: number, qamHiddenByParent: boolean): ReactElement {
    return (
        <Fragment>
            {allOf.map((action) => buildPipelineAction(action, indentLevel + 1, qamHiddenByParent))}
        </Fragment>
    );
}

function buildPipelineAction(action: PipelineAction, indentLevel: number, qamHiddenByParent: boolean): ReactElement {
    const { dispatch } = useModifiablePipelineContainer();
    const configErrors = useContext(ConfigErrorContext);

    console.log('recieved config errors: ', configErrors);


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

    const hideQamForChildren = !action.is_visible_on_qam || qamHiddenByParent;
    const newIndentLevel = selection.type === 'OneOf'
        ? indentLevel = + 1
        : indentLevel;
    const built = forcedEnabled || isEnabled
        ? buildSelection(action.id, action.selection, newIndentLevel, hideQamForChildren)
        : null;
    console.log(built?.props);

    const hasError = configErrors[action.id]?.length ?? 0 > 0;

    return (
        <Fragment>
            <Field
                indentLevel={indentLevel}
                focusable={!hasError && ((!built && forcedEnabled) || (selection.type !== 'AllOf' && forcedEnabled && qamHiddenByParent && !configErrors[action.id]))}
                label={action.name}
                description={action.description}
                icon={<ActionIcon action={action} />}
            >
                <div style={{ display: 'flex', flexDirection: 'row', justifyContent: 'space-between', alignItems: 'center', }}>
                    {
                        [
                            <ConfigErrorWarning errors={configErrors[action.id]} />,
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
                                ?
                                <DialogButton
                                    focusable={!qamHiddenByParent}
                                    style={{
                                        width: 'fit-content',
                                        minWidth: 'fit-content',
                                        height: 'fit-content',
                                        padding: '10px 12px',
                                        opacity: qamHiddenByParent ? '60%' : '100%'
                                    }}
                                    onClick={qamHiddenByParent
                                        ? undefined
                                        : () => toggleQAMVisible(action)}
                                    onOKButton={qamHiddenByParent
                                        ? undefined
                                        : () => toggleQAMVisible(action)}
                                    onOKActionDescription={qamHiddenByParent
                                        ? undefined
                                        : action.is_visible_on_qam
                                            ? 'hide on QAM'
                                            : 'show on QAM'}
                                >
                                    {
                                        built !== null ?
                                            action.is_visible_on_qam && !qamHiddenByParent
                                                ? <FaEye />
                                                : <FaEyeSlash />
                                            : null
                                    }
                                </DialogButton>
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
