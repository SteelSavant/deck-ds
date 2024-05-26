import {
    DialogBody,
    DialogControlsSection,
    Dropdown,
    Field,
    Focusable,
    Toggle,
} from 'decky-frontend-lib';
import { Fragment, ReactElement, createContext, useContext } from 'react';

import {
    Action,
    ActionOneOf,
    DependencyError,
    PipelineAction,
    PipelineTarget,
    RuntimeSelection,
} from '../../backend';
import ActionIcon from '../../components/ActionIcon';
import ConfigErrorWarning from '../../components/ConfigErrorWarning';
import { useAppState } from '../../context/appContext';
import { ConfigErrorContext } from '../../context/configErrorContext';
import { MaybeString } from '../../types/short';
import QAMEditAction from './QAMEditAction';

const PipelineTargetContext = createContext<PipelineTarget>('Desktop');
export const ProfileContext = createContext('notset');

export default function QAMPipelineTargetDisplay({
    root,
    target,
}: {
    root: RuntimeSelection;
    target: PipelineTarget;
}): ReactElement {
    return (
        <DialogBody style={{ marginBottom: '10px' }}>
            <DialogControlsSection>
                <Field focusable={false} />
                <PipelineTargetContext.Provider value={target}>
                    {buildRootSelection(root)}
                </PipelineTargetContext.Provider>
            </DialogControlsSection>
        </DialogBody>
    );
}

function buildRootSelection(selection: RuntimeSelection): ReactElement {
    const type = selection.type;
    switch (type) {
        case 'AllOf':
            return buildAllOf(selection.value);
        default:
            throw 'root selection must be an AllOf';
    }
}

function buildAction(
    action_id: string,
    toplevel_id: string,
    externalProfile: MaybeString,
    action: Action,
): ReactElement | null {
    const { dispatchUpdate } = useAppState();
    const profileId = useContext(ProfileContext);
    const target = useContext(PipelineTargetContext);

    // invoked as a function to allow seeing if the component returns null,
    // so we can ignore rendering things that aren't configurable in the QAM
    const component = QAMEditAction({
        action,
        onChange: (updatedAction) => {
            console.log('dispatching action edit', {
                type: 'updateAction',
                id: action_id,
                action: updatedAction,
            });

            dispatchUpdate(profileId, {
                externalProfile: externalProfile,
                update: {
                    type: 'updateAction',
                    toplevel_id,
                    action_id: action_id,
                    target: target,
                    action: updatedAction,
                },
            });
        },
    });

    return component;
}

function buildOneOf(oneOf: ActionOneOf): ReactElement {
    const action = oneOf.actions.find((a) => a.id === oneOf.selection)!;
    return buildPipelineAction(action);
}

function buildAllOf(allOf: PipelineAction[]): ReactElement {
    return <>{allOf.map((action) => buildPipelineAction(action))}</>;
}

function buildPipelineAction(action: PipelineAction): ReactElement {
    const { dispatchUpdate } = useAppState();

    const profileBeingOverridden = useContext(ProfileContext);
    const configErrors = useContext(ConfigErrorContext);
    const target = useContext(PipelineTargetContext);

    if (!action.is_visible_on_qam) {
        return <div />;
    }

    const selection = action.selection;
    const type = selection.type;

    const forcedEnabled =
        action.enabled === null || action.enabled === undefined;
    const isEnabled = action.enabled || forcedEnabled;

    const props: HeaderProps = {
        isEnabled,
        forcedEnabled,
        action,
        configErrors: configErrors[action.id],
    };

    switch (type) {
        case 'AllOf':
            // TODO::handle userdefined
            if (forcedEnabled || selection.value.length == 0) {
                return buildAllOf(selection.value);
            } else {
                return (
                    <>
                        <Header {...props} />
                        {isEnabled ? buildAllOf(selection.value) : <div />}
                    </>
                );
            }

        case 'OneOf':
            return (
                <>
                    <Header {...props} />
                    {isEnabled ? (
                        <>
                            <Field
                                focusable={false}
                                childrenContainerWidth="max"
                            >
                                <Focusable>
                                    <Dropdown
                                        selectedOption={
                                            selection.value.selection
                                        }
                                        rgOptions={selection.value.actions.map(
                                            (a) => {
                                                return {
                                                    label: a.name,
                                                    data: a.id,
                                                };
                                            },
                                        )}
                                        onChange={(option) => {
                                            dispatchUpdate(
                                                profileBeingOverridden,
                                                {
                                                    externalProfile:
                                                        action.profile_override,
                                                    update: {
                                                        type: 'updateOneOf',
                                                        action_id: action.id,
                                                        toplevel_id:
                                                            action.toplevel_id,
                                                        target: target,
                                                        selection: option.data,
                                                    },
                                                },
                                            );
                                        }}
                                    />
                                </Focusable>
                            </Field>
                            {buildOneOf(selection.value)}
                        </>
                    ) : (
                        <div />
                    )}
                </>
            );
        case 'Action':
            const actionComponent = buildAction(
                action.id,
                action.toplevel_id,
                action.profile_override,
                selection.value,
            );

            if (actionComponent) {
                return (
                    <>
                        <Header {...props} />
                        {isEnabled ? actionComponent : <div />}
                    </>
                );
            } else {
                return <Fragment />;
            }

        default:
            const typecheck: never = type;
            throw `action type failed to typecheck: ${typecheck}`;
    }
}

interface HeaderProps {
    isEnabled: boolean;
    forcedEnabled: boolean;
    action: PipelineAction;
    configErrors?: DependencyError[] | undefined;
}

function FromProfileComponent({ action }: { action: PipelineAction }) {
    const profileBeingOverridden = useContext(ProfileContext);
    const target = useContext(PipelineTargetContext);

    const { dispatchUpdate } = useAppState();

    return (
        <Field focusable={false} label="Use per-game profile">
            <Focusable>
                <Toggle
                    value={!action.profile_override}
                    onChange={(value) => {
                        const newOverride = value
                            ? null
                            : profileBeingOverridden;
                        console.log(
                            'current profile override ',
                            action.profile_override,
                            'value:',
                            !action.profile_override,
                        );
                        console.log(
                            'changed to',
                            value,
                            ', setting profile override to ',
                            newOverride,
                        );

                        dispatchUpdate(profileBeingOverridden, {
                            externalProfile: null,
                            update: {
                                type: 'updateProfileOverride',
                                action_id: action.id,
                                toplevel_id: action.toplevel_id,
                                target: target,
                                profileOverride: newOverride,
                            },
                        });
                    }}
                />
            </Focusable>
        </Field>
    );
}

function EnabledComponent({
    isEnabled,
    forcedEnabled,
    action,
}: HeaderProps): ReactElement {
    const profileBeingOverridden = useContext(ProfileContext);
    const target = useContext(PipelineTargetContext);
    const { dispatchUpdate } = useAppState();

    return forcedEnabled ? (
        <div />
    ) : (
        <Field focusable={false} label="Enabled">
            <Focusable>
                <Toggle
                    value={isEnabled}
                    onChange={(value) =>
                        dispatchUpdate(profileBeingOverridden, {
                            externalProfile: action.profile_override,
                            update: {
                                target: target,
                                toplevel_id: action.toplevel_id,
                                type: 'updateEnabled',
                                action_id: action.id,
                                isEnabled: value,
                            },
                        })
                    }
                />
            </Focusable>
        </Field>
    );
}

function Header(props: HeaderProps): ReactElement {
    const action = props.action;
    const displayName = action.name.toLocaleUpperCase();
    const errors = props.configErrors ?? [];

    return (
        <>
            <Field
                focusable={false}
                label={displayName}
                icon={<ActionIcon action={action} />}
            >
                {errors.length === 0 ? (
                    <div />
                ) : (
                    <ConfigErrorWarning errors={errors} />
                )}
            </Field>
            <FromProfileComponent {...props} />
            <EnabledComponent {...props} />
        </>
    );
}
