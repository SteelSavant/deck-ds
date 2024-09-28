import { DialogButton, Field, showModal, useParams } from '@decky/ui';
import { ReactElement } from 'react';
import { FaEdit } from 'react-icons/fa';
import {
    PipelineContainer,
    isCategoryProfile,
    setProfile,
} from '../../../backend';
import HandleLoading from '../../../components/HandleLoading';
import {
    ModifiablePipelineContainerProvider,
    useModifiablePipelineContainer,
} from '../../../context/modifiablePipelineContext';
import useProfile from '../../../hooks/useProfile';
import PipelineDisplay from '../../PipelineDisplay';
import ProfileInfo from './ProfileInfo';
import EditProfileNameModal from './modals/EditPipelineNameModal';

export default function ProfilePreviewRoute(): ReactElement {
    const { profileid } = useParams<{ profileid: string }>();
    const profile = useProfile(profileid);

    return (
        <HandleLoading
            value={profile}
            onOk={(profile) => {
                if (!profile) {
                    return (
                        <p>
                            {' '}
                            Profile {profileid} does not exist! Something has
                            gone terribly wrong...
                        </p>
                    );
                } else {
                    return (
                        <ModifiablePipelineContainerProvider
                            initialContainer={profile}
                            onPipelineUpdate={async (profile) => {
                                if (!isCategoryProfile(profile)) {
                                    throw 'PipelineContainer should be CategoryProfile';
                                }

                                return await setProfile({
                                    profile: profile,
                                });
                            }}
                        >
                            <PipelineDisplay
                                header={PipelineHeader}
                                general={ProfileInfo}
                            />
                        </ModifiablePipelineContainerProvider>
                    );
                }
            }}
        />
    );
}

function PipelineHeader(container: PipelineContainer): ReactElement {
    const { state, dispatch } = useModifiablePipelineContainer();

    function onEditTitle() {
        showModal(
            <EditProfileNameModal
                pipeline={state.container.pipeline}
                onSave={async (name) => {
                    return (
                        await dispatch({
                            update: {
                                type: 'updatePipelineInfo',
                                info: {
                                    ...container.pipeline,
                                    name: name,
                                },
                            },
                        })
                    ).mapErr((e) => e.err);
                }}
            />,
        );
    }

    return (
        <Field
            focusable={false}
            label={<h3>{container.pipeline.name}</h3>}
            bottomSeparator="thick"
            inlineWrap="keep-inline"
        >
            <DialogButton onOKButton={onEditTitle} onClick={onEditTitle}>
                <FaEdit />
            </DialogButton>
        </Field>
    );
}
