import { DialogButton, Dropdown, Field, Focusable, Toggle, showModal } from "decky-frontend-lib";
import { Fragment, ReactElement } from "react";
import { FaPlus, FaTrash, FaX } from "react-icons/fa6";
import { CategoryProfile, GamepadButtonSelection, PipelineContainer, gamepadButtonSelectionOptions, isCategoryProfile } from "../../../backend";
import HandleLoading from "../../../components/HandleLoading";
import { useModifiablePipelineContainer } from "../../../context/modifiablePipelineContext";
import useGlobalSettings from "../../../hooks/useGlobalSettings";
import useTemplates from "../../../hooks/useTemplates";
import useToplevel from "../../../hooks/useToplevel";
import { labelForGamepadButton } from "../../../util/display";
import AddProfileTagModal from "./modals/AddProfileTagModal";
import AddToplevelActionModal from "./modals/AddToplevelActionModal";


export default function ProfileInfo(container: PipelineContainer): ReactElement {
    if (!isCategoryProfile(container)) {
        throw 'PipelineContainer should be CategoryProfile';
    }

    const profile: CategoryProfile = container;

    const { dispatch } = useModifiablePipelineContainer();

    const { settings } = useGlobalSettings();

    const templates = useTemplates();
    const toplevel = useToplevel();

    function removeTag(tag: string) {
        dispatch({
            update: {
                type: 'updateTags',
                tags: profile.tags.filter((t) => t !== tag),
            }
        })
    }

    function addTag() {
        showModal(<AddProfileTagModal
            onSave={(tag) => {
                const unique = new Set(profile.tags);
                unique.delete(tag);
                dispatch({
                    update: {
                        type: 'updateTags',
                        tags: [...unique, tag], // set unique tags; no duplicates. If tag exists in 
                    }
                })
            }}
        />)
    }

    function addTopLevelAction() {
        showModal(<AddToplevelActionModal
            onSave={(info) => {
                dispatch({
                    update: {
                        type: 'addTopLevel',
                        action_id: info.id,
                    }
                })
            }}
        />)
    }

    function deleteToplevelAction(id: string): void {
        // TODO::make this a confirm modal
        dispatch({
            update: {
                type: 'removeTopLevel',
                id: id,
            },
        })
    }

    const loading = templates && settings && toplevel
        ? templates.andThen((t) => settings.map((s) => { return { templates: t, globalSettings: s } })).andThen((ts) => toplevel.map((tl) => {
            return {
                ...ts,
                toplevel: tl
            }
        }))
        : undefined
        ;

    const exitHooks = profile.pipeline.exit_hooks;
    const flattenedHooks = [[exitHooks[0]], [exitHooks[1]], exitHooks[2]].flat();
    const availableHooks: GamepadButtonSelection[] = gamepadButtonSelectionOptions.filter((v) => !flattenedHooks.includes(v));

    console.log('exit hooks:', exitHooks);
    console.log('flattened hooks:', flattenedHooks)

    function deleteExitHook(i: number) {
        flattenedHooks.splice(i, 1);
        dispatch({
            update: {
                type: 'updatePipelineInfo',
                info: {
                    exit_hooks: [flattenedHooks[0], flattenedHooks[1], flattenedHooks.slice(2)]
                }
            }
        })
    }

    function onAddExitHook() {
        dispatch({
            update: {
                type: 'updatePipelineInfo',
                info: {
                    exit_hooks: [flattenedHooks[0], flattenedHooks[1], flattenedHooks.slice(2).concat(availableHooks[0])]
                }
            }
        })
    }

    // TODO::make description editable
    return <HandleLoading
        value={loading}
        onOk={({ templates, globalSettings, toplevel }) => (
            <div>
                <Field
                    focusable={false}
                // description={profile.pipeline.description}
                />
                <Field
                    focusable={false}
                    label='Platform'
                    description='Platform on which the application runs. Native apps and a selection of emulators are supported.'
                >
                    <Dropdown
                        selectedOption={container.pipeline.platform.root}
                        rgOptions={templates.map((t) => {
                            return {
                                label: t.pipeline.name,
                                data: t.pipeline.platform.root
                            }
                        })}
                        onChange={(v) => {
                            dispatch({
                                update: {
                                    type: 'updatePlatform',
                                    platform: v.data
                                }
                            })
                        }}
                    />
                </Field>
                <Field
                    focusable={false}
                    label={"Additional Actions"}
                    description={"Additional top-level actions to run, such as launching a secondary app."}
                >
                    <DialogButton
                        onOKButton={addTopLevelAction}
                        onClick={addTopLevelAction}
                    >
                        Add Action
                    </DialogButton>

                </Field>
                {profile.pipeline.toplevel.map((v) => {
                    const match = toplevel.find((tl) => tl.id === v.root);

                    if (!match) {
                        return null;
                    }

                    return <Field focusable={false} indentLevel={1} label={match.name} description={match.description}>
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
                            onOKButton={() => deleteToplevelAction(v.id)}
                            onClick={() => deleteToplevelAction(v.id)}
                        >
                            <FaTrash />
                        </DialogButton>
                    </Field>
                })}
                <Field
                    focusable={false}
                    label='Collections'
                    description='Steam collections for which this profile should be available.'
                    bottomSeparator="none"
                >
                    <DialogButton onOKButton={addTag} onClick={addTag} onOKActionDescription='Add Collection'>
                        <FaPlus />
                        Add Collection
                    </DialogButton>
                </Field>
                <Focusable style={{ display: 'flex', flexDirection: 'row', flexWrap: 'wrap' }}>
                    {
                        profile.tags.map((t) =>
                            <Focusable>
                                <ProfileTag tag={t} removeTag={removeTag} />
                            </Focusable>
                        )
                    }
                </Focusable>
                <Field focusable={false} />
                <Field
                    focusable={false}
                    label='Register Exit Hooks'
                    description='In desktop mode, register a button chord that exits the app when held. Disable if your controller config in Steam Input already has an exit mapping.'
                >
                    <Toggle
                        value={profile.pipeline.should_register_exit_hooks}
                        onChange={(value) => {
                            dispatch({
                                update: {
                                    type: 'updatePipelineInfo',
                                    info: {
                                        register_exit_hooks: value
                                    }
                                }
                            });
                        }}
                    />
                </Field>
                {
                    profile.pipeline.should_register_exit_hooks ?
                        <Fragment>
                            {
                                flattenedHooks.map((hook, i) => {
                                    return (
                                        <Field indentLevel={1} focusable={false}>
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
                                                        dispatch({
                                                            update: {
                                                                type: 'updatePipelineInfo',
                                                                info: {
                                                                    exit_hooks: [flattenedHooks[0], flattenedHooks[1], flattenedHooks.slice(2)]
                                                                }
                                                            }
                                                        })
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
                        : undefined
                }
                <Field
                    focusable={false}
                    label="Primary Target"
                    description="Determines which target is used by the primary 'Play' button when patching the UI."
                >
                    <Dropdown
                        selectedOption={profile.pipeline.primary_target_override}
                        rgOptions={
                            [
                                {
                                    label: `Global Setting (${globalSettings.primary_ui_target})`,
                                    data: null
                                },
                                ...['Gamemode', 'Desktop'].map((t) => {
                                    return {
                                        label: t,
                                        data: t
                                    }
                                })]}
                        onChange={(option) => {
                            dispatch({
                                update: {
                                    type: 'updatePipelineInfo',
                                    info: {
                                        primary_target_override: option.data,
                                    }
                                }
                            });
                        }
                        }
                    />
                </Field>
            </div>
        )}
    />;
}


function ProfileTag({ tag, removeTag }: { tag: string, removeTag: (tag: string) => void }): ReactElement {
    const display = collectionStore.userCollections.find((uc) => uc.id === tag)?.displayName;
    return display ? (
        <div style={{ marginRight: '10px' }}>
            <DialogButton style={{ margin: '5px', display: 'flex', flexDirection: 'row', justifyContent: 'space-between' }} onClick={() => removeTag(tag)} onOKButton={() => removeTag(tag)} onOKActionDescription='Remove Collection'>
                {display}
                <FaX />
            </DialogButton>
        </div>
    ) : <div />
}