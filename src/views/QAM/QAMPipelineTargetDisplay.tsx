import { DialogBody, DialogControlsSection, Dropdown, Field, Focusable, Toggle } from "decky-frontend-lib";
import { Fragment, ReactElement, useContext } from "react";
import { ProfileContext } from ".";
import { Action, ActionOneOf, ActionSelection, DependencyError, PipelineAction } from "../../backend";
import ActionIcon from "../../components/ActionIcon";
import ConfigErrorWarning from "../../components/ConfigErrorWarning";
import { useAppState } from "../../context/appContext";
import { ConfigErrorContext } from "../../context/configErrorContext";
import { MaybeString } from "../../types/short";
import QAMEditAction from "./QAMEditAction";

export default function QAMPipelineTargetDisplay({ root }: {
    root: ActionSelection,
}): ReactElement {

    return (
        <DialogBody style={{ marginBottom: '10px' }}>
            <DialogControlsSection>
                <Field focusable={false} />
                {buildRootSelection('root', root) ?? <div />}
            </DialogControlsSection>
        </DialogBody>
    )
}

function buildRootSelection(id: string, selection: ActionSelection): ReactElement | null {
    switch (selection.type) {
        case "Action":
            return buildAction(id, null, selection.value);
        case "OneOf":
            return buildOneOf(selection.value);
        case "AllOf":
            return buildAllOf(selection.value);
    }
}

function buildAction(id: string, externalProfile: MaybeString, action: Action): ReactElement | null {
    const { dispatchUpdate } = useAppState();
    const profileId = useContext(ProfileContext);

    // invoked as a function to allow seeing if the component returns null,
    // so we can ignore rendering things that aren't configurable in the QAM
    const component = QAMEditAction({
        action, onChange: (updatedAction) => {

            console.log('dispatching action edit', {
                type: 'updateAction',
                id: id,
                action: updatedAction,
            });

            dispatchUpdate(profileId, {
                externalProfile: externalProfile,
                update: {
                    type: 'updateAction',
                    id: id,
                    action: updatedAction,
                }
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
    const { dispatchUpdate } = useAppState();

    const profileBeingOverridden = useContext(ProfileContext);
    const configErrors = useContext(ConfigErrorContext);

    if (!action.is_visible_on_qam) {
        return <div />
    }

    const selection = action.selection;
    const type = selection.type;

    const forcedEnabled = action.enabled === null || action.enabled === undefined;
    const isEnabled = action.enabled || forcedEnabled;

    const props: HeaderProps = {
        isEnabled,
        forcedEnabled,
        action,
        configErrors: configErrors[action.id]
    }

    switch (type) {
        case 'AllOf':
            if (forcedEnabled || selection.value.length == 0) {
                return buildAllOf(selection.value);
            } else {
                return (
                    <Fragment>
                        <Header {...props} />
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
                    <Header {...props} />
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
                                            dispatchUpdate(profileBeingOverridden, {
                                                externalProfile: action.profile_override,
                                                update: {
                                                    type: 'updateOneOf',
                                                    id: action.id,
                                                    selection: option.data,
                                                }
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
            const actionComponent = buildAction(action.id, action.profile_override, selection.value);

            if (actionComponent) {
                return (
                    <Fragment>
                        <Header {...props} />
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

interface HeaderProps {
    isEnabled: boolean,
    forcedEnabled: boolean,
    action: PipelineAction,
    configErrors?: DependencyError[] | undefined
}


function FromProfileComponent({ action }: { action: any }) {
    const profileBeingOverridden = useContext(ProfileContext);
    const { dispatchUpdate } = useAppState();

    return <Field focusable={false} label="Use per-game profile">
        <Focusable>
            <Toggle value={!action.profile_override} onChange={(value) => {
                dispatchUpdate(profileBeingOverridden, {
                    externalProfile: null,
                    update: {
                        type: 'updateProfileOverride',
                        id: action.id,
                        profileOverride: value
                            ? null
                            : profileBeingOverridden
                    }
                })
            }} />
        </Focusable>
    </Field>
};

function EnabledComponent({ isEnabled, forcedEnabled, action }: HeaderProps): ReactElement {
    const profileBeingOverridden = useContext(ProfileContext);
    const { dispatchUpdate } = useAppState();

    return forcedEnabled
        ? <div />
        : <Field focusable={false} label="Enabled">
            <Focusable>
                <Toggle value={isEnabled} onChange={(value) =>
                    dispatchUpdate(profileBeingOverridden, {
                        externalProfile: action.profile_override,
                        update: {
                            type: 'updateEnabled',
                            id: action.id,
                            isEnabled: value,
                        }
                    })
                } />
            </Focusable>
        </Field>
}

function Header(props: HeaderProps): ReactElement {
    const action = props.action;
    const displayName = action.name.toLocaleUpperCase();
    const errors = props.configErrors ?? [];

    console.log('QAM Header got errors:', errors);

    return (
        <Fragment>
            <Field
                focusable={false}
                label={displayName}
                icon={<ActionIcon action={action} />}
            >
                {
                    errors.length === 0
                        ? <div />
                        : <ConfigErrorWarning errors={errors} />
                }
            </Field>
            <FromProfileComponent action={action} />
            <EnabledComponent {...props} />
        </Fragment>
    );
}





