import { DialogButton, Field, Focusable, showModal } from "decky-frontend-lib";
import { ReactElement } from "react";
import { FaPlus, FaX } from "react-icons/fa6";
import { CategoryProfile, PipelineContainer, isCategoryProfile } from "../../../backend";
import { useModifiablePipelineContainer } from "../../../context/modifiablePipelineContext";
import AddProfileTagModal from "./modals/AddProfileTagModal";

export default function ProfileInfo(container: PipelineContainer): ReactElement {
    if (!isCategoryProfile(container)) {
        throw 'PipelineContainer should be CategoryProfile';
    }

    const profile: CategoryProfile = container;

    const { dispatch } = useModifiablePipelineContainer();

    function removeTag(tag: string) {
        dispatch({
            externalProfile: null,
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
                externalProfile: null,
                update: {
                    type: 'updateTags',
                    tags: [...unique, tag], // set unique tags; no duplicates. If tag exists in 
                }
            })
        }} />)
    }

    // TODO::make description editable
    // TODO::dependencies section
    return (
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
            <div style={{ display: 'flex', flexDirection: 'row', flexWrap: 'wrap' }}>
                {
                    profile.tags.map((t) =>
                        <Focusable>
                            <ProfileTag tag={t} removeTag={removeTag} />
                        </Focusable>
                    )
                }
            </div>
            <Field focusable={false} />
        </div>
    );
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