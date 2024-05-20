import { DialogButton, Dropdown, Field } from "decky-frontend-lib";
import { Fragment, ReactElement } from "react";
import { FaTrash } from "react-icons/fa6";
import { GamepadButtonSelection, gamepadButtonSelectionOptions } from "../backend";
import { ExitHooks } from "../types/backend_api";
import { labelForGamepadButton } from "../util/display";

interface EditExitHooksProps {
    exitHooks: ExitHooks,
    indentLevel?: number | undefined
    onChange: (hooks: ExitHooks) => void,
}

export function EditExitHooks({ exitHooks, indentLevel, onChange }: EditExitHooksProps): ReactElement {

    const flattenedHooks = [[exitHooks[0]], [exitHooks[1]], exitHooks[2]].flat();
    const availableHooks: GamepadButtonSelection[] = gamepadButtonSelectionOptions.filter((v) => !flattenedHooks.includes(v));

    function deleteExitHook(i: number) {
        flattenedHooks.splice(i, 1);
        onChange([flattenedHooks[0], flattenedHooks[1], flattenedHooks.slice(2)])
    }

    function onAddExitHook() {
        onChange([flattenedHooks[0], flattenedHooks[1], flattenedHooks.slice(2).concat(availableHooks[0])]);
    }

    return (
        <Fragment>
            {
                flattenedHooks.map((hook, i) => {
                    return (
                        <Field indentLevel={indentLevel} focusable={false}>
                            <div
                                style={{
                                    display: 'flex',
                                    flexDirection: 'row'
                                }}
                            >
                                <Dropdown
                                    selectedOption={hook}
                                    rgOptions={[hook].concat(availableHooks).map((v) => {
                                        return {
                                            label: labelForGamepadButton(v),
                                            data: v
                                        }
                                    })}
                                    onChange={(props) => {
                                        const data: GamepadButtonSelection = props.data;
                                        const index = flattenedHooks.indexOf(hook);
                                        flattenedHooks.splice(index, 1, data);
                                        onChange([flattenedHooks[0], flattenedHooks[1], flattenedHooks.slice(2)])

                                    }}
                                />
                                {
                                    // TODO::styling
                                    flattenedHooks.length > 2 ?
                                        <DialogButton style={{
                                            backgroundColor: 'red',
                                            height: '40px',
                                            width: '40px',
                                            padding: '10px 12px',
                                            minWidth: '40px',
                                            display: 'flex',
                                            flexDirection: 'column',
                                            justifyContent: 'center',
                                            marginRight: '10px'
                                        }}
                                            onOKButton={() => deleteExitHook(i)}
                                            onClick={() => deleteExitHook(i)}
                                        >
                                            <FaTrash />
                                        </DialogButton>
                                        : undefined
                                }
                            </div>
                        </Field>
                    )
                })

            }
            {
                availableHooks.length > 0
                    ? <Field indentLevel={1} focusable={false}>
                        <DialogButton
                            onClick={onAddExitHook}
                            onOKButton={onAddExitHook}
                        >
                            Add Chord Button
                        </DialogButton>
                    </Field>
                    : undefined
            }
        </Fragment>
    )
}

