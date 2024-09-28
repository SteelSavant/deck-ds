import {
    DialogBody,
    DialogButton,
    DialogControlsSection,
    showModal,
} from 'decky-frontend-lib';
import { VFC } from 'react';
import { FaPlus } from 'react-icons/fa6';
import { getTemplates } from '../../../backend';
import { CreateProfileModal } from '../../../components/CreateProfileModal';
import HandleLoading from '../../../components/HandleLoading';
import useProfiles from '../../../hooks/useProfiles';
import { logger } from '../../../util/log';
import ProfileMenuItem from './ProfileMenuItem';

export const ProfilesPage: VFC = () => {
    const { profiles, deleteProfile } = useProfiles();

    const createNewProfile = async () => {
        const templates = await getTemplates();
        if (templates.isOk) {
            showModal(
                <CreateProfileModal
                    collection={null}
                    templates={templates.data.templates}
                />,
            );
        } else {
            logger.toastError(
                'Failed to load profile templates:',
                templates.err,
            );
        }
    };

    return (
        <HandleLoading
            value={profiles}
            onOk={(profiles) => (
                <DialogBody>
                    <DialogControlsSection>
                        {profiles.length > 0
                            ? profiles.map((p) => (
                                  <ProfileMenuItem
                                      profile={p}
                                      deleteProfile={deleteProfile}
                                  />
                              ))
                            : 'No profiles have been created.'}
                    </DialogControlsSection>
                    <div
                        style={{
                            padding: '15px',
                            display: 'flex',
                            minWidth: '100px',
                            justifyContent: 'space-between',
                            alignItems: 'center',
                        }}
                    >
                        <DialogButton
                            onClick={createNewProfile}
                            onOKButton={createNewProfile}
                        >
                            <FaPlus style={{ paddingRight: '1rem' }} />
                            New Profile
                        </DialogButton>
                    </div>
                </DialogBody>
            )}
        />
    );
};
