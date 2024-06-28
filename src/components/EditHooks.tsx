import { ReactElement } from 'react';
import { BtnChord } from '../types/backend_api';
import { Result } from '../util/result';

interface EditHooksProps {
    exitHooks: BtnChord;
    indentLevel?: number;
    onChange: (hooks: BtnChord) => Promise<Result<void, string>>;
}

export function EditHooks({}: EditHooksProps): ReactElement {
    return <p>TODO</p>;
    // const flattenedHooks = [
    //     [exitHooks[0]],
    //     [exitHooks[1]],
    //     exitHooks[2],
    // ].flat();
    // const availableHooks: GamepadButtonSelection[] =
    //     gamepadButtonSelectionOptions.filter(
    //         (v) => !flattenedHooks.includes(v),
    //     );

    // async function deleteExitHook(i: number) {
    //     flattenedHooks.splice(i, 1);
    //     const res = await onChange([
    //         flattenedHooks[0],
    //         flattenedHooks[1],
    //         flattenedHooks.slice(2),
    //     ]);

    //     if (!res.isOk) {
    //         logger.toastWarn('Error deleting hook button:', res.err);
    //     }
    // }

    // function onAddExitHook() {
    //     showModal(
    //         <AddGamepadButtonModal
    //             buttons={availableHooks}
    //             onSave={(button) =>
    //                 onChange([
    //                     flattenedHooks[0],
    //                     flattenedHooks[1],
    //                     flattenedHooks.slice(2).concat(button),
    //                 ])
    //             }
    //         />,
    //     );
    // }

    // return (
    //     <>
    //         {flattenedHooks.map((hook, i) => {
    //             return (
    //                 <Field indentLevel={indentLevel} focusable={false}>
    //                     <FocusableRow>
    //                         <Dropdown
    //                             selectedOption={hook}
    //                             rgOptions={[hook]
    //                                 .concat(availableHooks)
    //                                 .map((v) => {
    //                                     return {
    //                                         label: labelForGamepadButton(v),
    //                                         data: v,
    //                                     };
    //                                 })}
    //                             onChange={async (props) => {
    //                                 const data: GamepadButtonSelection =
    //                                     props.data;
    //                                 const index = flattenedHooks.indexOf(hook);
    //                                 flattenedHooks.splice(index, 1, data);
    //                                 const res = await onChange([
    //                                     flattenedHooks[0],
    //                                     flattenedHooks[1],
    //                                     flattenedHooks.slice(2),
    //                                 ]);
    //                                 if (!res.isOk) {
    //                                     logger.toastWarn(
    //                                         'Error updating hook button:',
    //                                         res.err,
    //                                     );
    //                                 }
    //                             }}
    //                         />
    //                         {flattenedHooks.length > 2 ? (
    //                             <DialogButton
    //                                 style={{
    //                                     backgroundColor: 'red',
    //                                     height: '40px',
    //                                     width: '40px',
    //                                     padding: '10px 12px',
    //                                     minWidth: '40px',
    //                                     display: 'flex',
    //                                     flexDirection: 'column',
    //                                     justifyContent: 'center',
    //                                     marginRight: '10px',
    //                                     marginLeft: '10px',
    //                                 }}
    //                                 onOKButton={() => deleteExitHook(i)}
    //                                 onClick={() => deleteExitHook(i)}
    //                             >
    //                                 <FaTrash />
    //                             </DialogButton>
    //                         ) : null}
    //                     </FocusableRow>
    //                 </Field>
    //             );
    //         })}
    //         {availableHooks.length > 0 ? (
    //             <Field indentLevel={1} focusable={false}>
    //                 <DialogButton
    //                     onClick={onAddExitHook}
    //                     onOKButton={onAddExitHook}
    //                 >
    //                     Add Chord Button
    //                 </DialogButton>
    //             </Field>
    //         ) : null}
    //     </>
    // );
}
