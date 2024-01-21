import { DialogBody, DialogControlsSection, Dropdown, Field, Focusable, PanelSectionRow, Toggle } from "decky-frontend-lib";
import { Fragment, ReactElement } from "react";
import { FaLink } from "react-icons/fa";
import { Action, ActionOneOf, ActionSelection, PipelineAction, } from "../../backend";
import { useModifiablePipelineContainer } from "../../context/modifiablePipelineContext";
import QAMEditAction from "./QAMEditAction";

export default function QAMPipelineTargetDisplay({ root }: {
    root: ActionSelection,
}): ReactElement {
    return (
        <DialogBody>
            <DialogControlsSection>
                <Field focusable={false} />
                {buildSelection('root', root)}
            </DialogControlsSection>
        </DialogBody>
    )
}

function buildSelection(id: string, selection: ActionSelection): ReactElement[] {
    switch (selection.type) {
        case "Action":
            return buildAction(id, selection.value);
        case "OneOf":
            return buildOneOf(selection.value);
        case "AllOf":
            return buildAllOf(selection.value);
    }
}

function buildAction(id: string, action: Action): ReactElement[] {
    const { dispatch } = useModifiablePipelineContainer();

    return (
        <QAMEditAction action={action} onChange={(updatedAction) => {
            dispatch({
                type: 'updateAction',
                id: id,
                action: updatedAction,
            });
        }} />
    )
}

function buildOneOf(oneOf: ActionOneOf): ReactElement[] {
    const action = oneOf.actions.find((a) => a.id === oneOf.selection)!;
    return buildPipelineAction(action);
}

function buildAllOf(allOf: PipelineAction[]): ReactElement {
    return (
        <Fragment>
            {allOf.map((action) => buildPipelineAction(action))}
        </Fragment>
    );
}

function buildPipelineAction(action: PipelineAction): ReactElement {
    const { dispatch } = useModifiablePipelineContainer();

    const selection = action.selection;
    const isEnabled = action.enabled;

    const forcedEnabled = isEnabled === null || isEnabled === undefined;
    return (
        <div style={{ flexDirection: 'row' }}>
            <PanelSectionRow>
                <ActionLabel action={action} />
                {
                    forcedEnabled
                        ? <div />
                        : <Focusable>
                            ENABLED (Add P before):
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
                                // TODO::add "from profile" as option
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
            </PanelSectionRow>
            {forcedEnabled || isEnabled ? buildSelection(action.id, action.selection) : <div />}
        </div>
    )
}


function ActionLabel({ action }: { action: PipelineAction }): ReactElement {
    return action.id.split(':').length === 3 && action.selection.type !== 'AllOf'
        ? <div>
            <h4>{action.name}</h4>
            <FaLink style={{
                paddingLeft: '10px',
                paddingRight: '10px'
            }} />
        </div>
        : <p>{action.name}</p>
}