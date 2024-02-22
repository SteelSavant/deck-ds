import { DialogButton, Dropdown, Field, Focusable, Toggle, showModal } from "decky-frontend-lib";
import { ReactElement } from "react";
import { FaPlus, FaX } from "react-icons/fa6";
import { CategoryProfile, PipelineContainer, isCategoryProfile } from "../../../backend";
import HandleLoading from "../../../components/HandleLoading";
import { useModifiablePipelineContainer } from "../../../context/modifiablePipelineContext";
import useGlobalSettings from "../../../hooks/useGlobalSettings";
import AddProfileTagModal from "./modals/AddProfileTagModal";

export default function ProfileInfo(container: PipelineContainer): ReactElement {
    if (!isCategoryProfile(container)) {
        throw 'PipelineContainer should be CategoryProfile';
    }

    const profile: CategoryProfile = container;

    const { dispatch } = useModifiablePipelineContainer();

    const { settings } = useGlobalSettings();

    function removeTag(tag: string) {
        dispatch({
            update: {
                type: 'updateTags',
                tags: profile.tags.filter((t) => t !== tag),
            }
        })
    }

    function addTag() {
        showModal(<AddProfileTagModal onSave={(tag) => {
            const unique = new Set(profile.tags);
            unique.delete(tag);
            dispatch({
                update: {
                    type: 'updateTags',
                    tags: [...unique, tag], // set unique tags; no duplicates. If tag exists in 
                }
            })
        }} />)
    }

    // TODO::make description editable
    return <HandleLoading
        value={settings}
        onOk={(globalSettings) => (
            <div>
                <Field focusable={false} description={profile.pipeline.description} />
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