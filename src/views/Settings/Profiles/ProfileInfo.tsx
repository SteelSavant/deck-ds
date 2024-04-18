import { DialogButton, Dropdown, Field, Focusable, Toggle, showModal } from "decky-frontend-lib";
import { ReactElement } from "react";
import { FaPlus, FaTrash, FaX } from "react-icons/fa6";
import { CategoryProfile, PipelineContainer, isCategoryProfile } from "../../../backend";
import HandleLoading from "../../../components/HandleLoading";
import { useModifiablePipelineContainer } from "../../../context/modifiablePipelineContext";
import useGlobalSettings from "../../../hooks/useGlobalSettings";
import useTemplates from "../../../hooks/useTemplates";
import useToplevel from "../../../hooks/useToplevel";
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
                    bottomSeparator="none"
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
                    description='Register holding (select + start) as hooks to exit app when launched in desktop mode. Disable if your controller config in Steam Input already has an exit mapping.'
                >
                    <Toggle
                        value={profile.pipeline.register_exit_hooks}
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