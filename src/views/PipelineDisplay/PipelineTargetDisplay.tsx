import {
    DialogBody,
    DialogButton,
    DialogControlsSection,
    Dropdown,
    Field,
    Focusable,
    Toggle,
} from '@decky/ui';
import { Fragment, ReactElement, createContext, useContext } from 'react';
import { FaEye, FaEyeSlash } from 'react-icons/fa';
import {
    Action,
    ActionOneOf,
    PipelineAction,
    PipelineTarget,
    RuntimeSelection,
} from '../../backend';
import ActionIcon from '../../components/ActionIcon';
import ConfigErrorWarning from '../../components/ConfigErrorWarning';
import { EditAction } from '../../components/EditAction';
import { ConfigErrorContext } from '../../context/configErrorContext';
import { useModifiablePipelineContainer } from '../../context/modifiablePipelineContext';
import { MaybeString } from '../../types/short';

const PipelineTargetContext = createContext<PipelineTarget>('Desktop');

export default function PipelineTargetDisplay({
    root,
}: {
    root: RuntimeSelection;
}): ReactElement {
    console.log('building PipelineTargetDisplay from', root);
    return (
        <DialogBody>
            <DialogControlsSection>
                {buildSelection(
                    'root',
                    null,
                    root,
                    root.type === 'AllOf' ? -1 : 0,
                    false,
                )}
            </DialogControlsSection>
        </DialogBody>
    );
}

function buildSelection(
    action_id: string,
    toplevel_id: MaybeString,
    selection: RuntimeSelection,
    indentLevel: number,
    qamHiddenByParent: boolean,
): ReactElement | null {
    const type = selection.type;
    console.log('Building selection: ', selection);
    switch (type) {
        case 'Action':
            return buildAction(
                action_id,
                toplevel_id!,
                selection.value,
                indentLevel,
            );
        case 'OneOf':
            return buildOneOf(selection.value, indentLevel, qamHiddenByParent);
        case 'AllOf': // fallthrough
        case 'AllOfErased':
            return buildAllOf(selection.value, indentLevel, qamHiddenByParent);
        default:
            const typecheck: never = type;
            throw `buildSelection switch failed to typecheck: ${typecheck}`;
    }
}

function buildAction(
    action_id: string,
    toplevel_id: string,
    action: Action,
    indentLevel: number,
): ReactElement | null {
    const { dispatch } = useModifiablePipelineContainer();
    const target = useContext(PipelineTargetContext);

    return EditAction({
        action: action,
        indentLevel: indentLevel + 1,
        onChange: (updatedAction) => {
            dispatch({
                update: {
                    type: 'updateAction',
                    action_id: action_id,
                    toplevel_id: toplevel_id,
                    action: updatedAction,
                    target,
                },
            });
        },
    });
}

function buildOneOf(
    oneOf: ActionOneOf,
    indentLevel: number,
    qamHiddenByParent: boolean,
): ReactElement {
    const action = oneOf.actions.find((a) => a.id === oneOf.selection)!;
    return buildPipelineAction(action, indentLevel + 1, qamHiddenByParent);
}

function buildAllOf(
    allOf: PipelineAction[],
    indentLevel: number,
    qamHiddenByParent: boolean,
): ReactElement {
    return (
        <>
            {allOf.map((action) =>
                buildPipelineAction(action, indentLevel + 1, qamHiddenByParent),
            )}
        </>
    );
}

function buildPipelineAction(
    action: PipelineAction,
    indentLevel: number,
    qamHiddenByParent: boolean,
): ReactElement {
    const { dispatch } = useModifiablePipelineContainer();
    const configErrors = useContext(ConfigErrorContext);
    const target = useContext(PipelineTargetContext);

    const selection = action.selection;
    const isEnabled = action.enabled;

    if (selection.type === 'AllOfErased') {
        return (
            <>
                {' '}
                {...selection.value.map((v) =>
                    buildSelection(
                        v.id,
                        v.toplevel_id,
                        v.selection,
                        indentLevel,
                        qamHiddenByParent,
                    ),
                )}{' '}
            </>
        );
    }

    const forcedEnabled = isEnabled === null || isEnabled === undefined;

    const toggleQAMVisible = (action: PipelineAction) => {
        dispatch({
            update: {
                type: 'updateVisibleOnQAM',
                action_id: action.id,
                toplevel_id: action.toplevel_id,
                visible: !action.is_visible_on_qam,
                target,
            },
        });
    };

    const hideQamForChildren = !action.is_visible_on_qam || qamHiddenByParent;
    const newIndentLevel =
        selection.type === 'OneOf' ? (indentLevel = +1) : indentLevel;

    const childAction =
        isEnabled || forcedEnabled
            ? buildSelection(
                  action.id,
                  action.toplevel_id,
                  action.selection,
                  newIndentLevel,
                  hideQamForChildren,
              )
            : null;
    const childActionIsConfigurable = childAction !== null;
    const hasError = configErrors[action.id]?.length ?? 0 > 0;

    return (
        <>
            <Field
                indentLevel={indentLevel}
                focusable={
                    !hasError &&
                    ((!childAction && forcedEnabled) ||
                        (selection.type !== 'AllOf' &&
                            forcedEnabled &&
                            qamHiddenByParent &&
                            !configErrors[action.id]))
                }
                label={action.name}
                description={action.description}
                icon={<ActionIcon action={action} />}
            >
                <Focusable
                    style={{
                        display: 'flex',
                        flexDirection: 'row',
                        justifyContent: 'space-between',
                        alignItems: 'center',
                    }}
                >
                    {[
                        <ConfigErrorWarning errors={configErrors[action.id]} />,
                        forcedEnabled ? null : (
                            <Focusable>
                                <Toggle
                                    value={isEnabled}
                                    onChange={(value) =>
                                        dispatch({
                                            update: {
                                                type: 'updateEnabled',
                                                action_id: action.id,
                                                toplevel_id: action.toplevel_id,
                                                isEnabled: value,
                                                target,
                                            },
                                        })
                                    }
                                />
                            </Focusable>
                        ),
                        selection.type === 'OneOf' ? (
                            <Focusable>
                                <Dropdown
                                    selectedOption={selection.value.selection}
                                    rgOptions={selection.value.actions.map(
                                        (a) => {
                                            return {
                                                label: a.name,
                                                data: a.id,
                                            };
                                        },
                                    )}
                                    onChange={(option) => {
                                        dispatch({
                                            update: {
                                                type: 'updateOneOf',
                                                toplevel_id: action.toplevel_id,
                                                action_id: action.id,
                                                selection: option.data,
                                                target,
                                            },
                                        });
                                    }}
                                />
                            </Focusable>
                        ) : null,
                        (selection.type !== 'AllOf' &&
                            (childActionIsConfigurable || !forcedEnabled)) ||
                        selection.type === 'OneOf' ? (
                            <DialogButton
                                focusable={!qamHiddenByParent}
                                style={{
                                    width: 'fit-content',
                                    minWidth: 'fit-content',
                                    height: 'fit-content',
                                    padding: '10px 12px',
                                    opacity: qamHiddenByParent ? '60%' : '100%',
                                }}
                                onClick={
                                    qamHiddenByParent
                                        ? undefined
                                        : () => toggleQAMVisible(action)
                                }
                                onOKButton={
                                    qamHiddenByParent
                                        ? undefined
                                        : () => toggleQAMVisible(action)
                                }
                                onOKActionDescription={
                                    qamHiddenByParent
                                        ? undefined
                                        : action.is_visible_on_qam
                                        ? 'hide on QAM'
                                        : 'show on QAM'
                                }
                            >
                                {action.is_visible_on_qam &&
                                !qamHiddenByParent ? (
                                    <FaEye />
                                ) : (
                                    <FaEyeSlash />
                                )}
                            </DialogButton>
                        ) : null,
                    ]
                        .filter((x) => x)
                        .map((x) => (
                            <div style={{ marginRight: '10px' }}>{x}</div>
                        ))}
                </Focusable>
            </Field>
            {childAction}
        </>
    );
}
