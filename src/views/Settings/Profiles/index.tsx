import {
    DialogBody,
    DialogButton,
    DialogControlsSection,
    Navigation,
} from 'decky-frontend-lib';
import { VFC } from 'react';
import { FaPlus } from 'react-icons/fa6';
import { createProfile, getTemplates } from '../../../backend';
import HandleLoading from '../../../components/HandleLoading';
import { useServerApi } from '../../../context/serverApiContext';
import useProfiles from '../../../hooks/useProfiles';
import { logger } from '../../../util/log';
import ProfileMenuItem from './ProfileMenuItem';

export const ProfilesPage: VFC = () => {
    const serverApi = useServerApi();
    const { profiles, deleteProfile } = useProfiles();

    const createNewProfile = async () => {
        const templates = await getTemplates();
        if (templates.isOk) {
            // TODO::this should probably use the platform modal...

            // hardcoded app template id
            const appTemplate = templates.data.templates.find(
                (v) => v.id === '84f870e9-9491-41a9-8837-d5a6f591f687',
            )!;
            const profile = await createProfile({
                pipeline: appTemplate.pipeline,
            });

            if (profile.isOk) {
                let id = profile.data.profile_id;
                Navigation.Navigate(`/deck-ds/settings/profiles/${id}`);
            } else {
                logger.toastWarn(
                    serverApi.toaster,
                    'Failed to create profile:',
                    profile.err.err,
                );
            }
        } else {
            logger.toastError(
                serverApi.toaster,
                'Failed to load templates:',
                templates.err.err,
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
