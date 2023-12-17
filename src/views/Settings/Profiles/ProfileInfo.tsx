import { DialogButton, Field, Focusable, showModal } from "decky-frontend-lib";
import { ReactElement } from "react";
import { FaPlus, FaX } from "react-icons/fa6";
import { useModifiablePipelineDefinition } from "../../../context/modifiablePipelineContext";
import { Pipeline } from "../../../types/backend_api";
import AddProfileTagModal from "./modals/AddProfileTagModal";

export default function ProfileInfo(pipeline: Pipeline): ReactElement {
    const { dispatch } = useModifiablePipelineDefinition();

    function removeTag(tag: string) {
        dispatch({
            type: 'updatePipelineInfo',
            info: {
                tags: pipeline.tags.filter((t) => t !== tag),
                description: undefined,
                name: undefined
            }
        })
    }


    function addTag() {
        showModal(<AddProfileTagModal onSave={(tag) => {
            const unique = new Set(pipeline.tags);
            unique.delete(tag);
            dispatch({
                type: 'updatePipelineInfo',
                info: {
                    tags: [...unique, tag], // set unique tags; no duplicates. If tag exists in 
                    description: undefined,
                    name: undefined
                }
            })
        }} />)
    }

    // TODO::make description editable
    // TODO::dependencies section
    return (
        <div>
            <Field focusable={false} description={pipeline.description} />
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
                    pipeline.tags.map((t) =>
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