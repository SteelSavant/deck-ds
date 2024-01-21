import { DialogBody, DialogControlsSection, Dropdown, Field, Focusable, Toggle } from "decky-frontend-lib";
import { Fragment, ReactElement } from "react";
import { Action, ActionOneOf, ActionSelection, PipelineAction, } from "../../backend";
import ActionIcon from "../../components/ActionIcon";
import { useModifiablePipelineContainer } from "../../context/modifiablePipelineContext";
import QAMEditAction from "./QAMEditAction";

export default function QAMPipelineTargetDisplay({ root }: {
    root: ActionSelection,
}): ReactElement {

    return (
        <DialogBody style={{ marginBottom: '10px' }}>
            <DialogControlsSection>
                <Field focusable={false} />
                {buildSelection('root', root) ?? <div />}
            </DialogControlsSection>
        </DialogBody>
    )
}

function buildSelection(id: string, selection: ActionSelection): ReactElement | null {
    switch (selection.type) {
        case "Action":
            return buildAction(id, selection.value);
        case "OneOf":
            return buildOneOf(selection.value);
        case "AllOf":
            return buildAllOf(selection.value);
    }
}

function buildAction(id: string, action: Action): ReactElement | null {
    const { dispatch } = useModifiablePipelineContainer();

    // invoked as a function to allow seeing if the component returns null,
    // so we can ignore rendering things that aren't configurable in the QAM
    const component = QAMEditAction({
        action, onChange: (updatedAction) => {
            dispatch({
                type: 'updateAction',
                id: id,
                action: updatedAction,
            });
        }
    });

    return component;
}

function buildOneOf(oneOf: ActionOneOf): ReactElement {
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
    const type = selection.type;

    const forcedEnabled = action.enabled === null || action.enabled === undefined;
    const isEnabled = action.enabled || forcedEnabled;

    const displayName = action.name.toLocaleUpperCase();

    const fromProfileComponent = <Field focusable={false} label="Use per-game profile">
        <Focusable>
            <Toggle value={!!action.profile_override} onChange={(value) => null // TODO::This
                // dispatch({
                //     type: 'updateEnabled',
                //     id: action.id,
                //     isEnabled: value,
                // })
            } />
        </Focusable>
    </Field>;

    const enabledComponent = forcedEnabled
        ? <div />
        : <Field focusable={false} label="Enabled">
            <Focusable>
                <Toggle value={isEnabled} onChange={(value) =>
                    dispatch({
                        type: 'updateEnabled',
                        id: action.id,
                        isEnabled: value,
                    })
                } />
            </Focusable>
        </Field>;

    function Header(): ReactElement {
        return (
            <Fragment>
                <Field
                    focusable={false}
                    label={displayName}
                    icon={<ActionIcon action={action} />}
                />
                {fromProfileComponent}
                {enabledComponent}
            </Fragment>
        );
    }

    switch (type) {
        case 'AllOf':
            if (forcedEnabled || selection.value.length == 0) {
                return buildAllOf(selection.value);
            } else {
                return (
                    <Fragment>
                        <Header />
                        {
                            isEnabled
                                ? buildAllOf(selection.value)
                                : <div />
                        }
                    </Fragment>
                )
            }

        case 'OneOf':
            return (
                <Fragment>
                    <Header />
                    {
                        isEnabled
                            ? <Fragment>
                                <Field focusable={false} childrenContainerWidth="max">
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
                                </Field>
                                {buildOneOf(selection.value)}
                            </Fragment>
                            : <div />
                    }
                </Fragment>
            )
        case 'Action':
            const actionComponent = buildAction(action.id, selection.value);

            console.log(displayName, actionComponent);

            if (actionComponent) {
                return (
                    <Fragment>
                        <Header />
                        {isEnabled ? actionComponent : <div />}
                    </Fragment>
                );
            } else {
                return <Fragment />
            }


        default:
            const typecheck: never = type;
            throw typecheck ?? 'action type failed to typecheck'
    }
}




