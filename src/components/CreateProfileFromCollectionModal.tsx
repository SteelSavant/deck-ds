import { ConfirmModal, Dropdown, Navigation } from 'decky-frontend-lib';
import { ReactElement, useState } from 'react';
import { Template, createProfile, getProfile, setProfile } from '../backend';
import { logger } from '../util/log';

export function CreateProfileFromCollectionModal({
    templates,
    collection,
    closeModal,
}: {
    collection: SteamCollection;
    templates: Template[];
    closeModal?: () => void;
}): ReactElement {
    const normalized = normalize(collection.displayName);

    // TODO::better comparison
    const matchingTemplate = templates.find((t) =>
        t.tags.find((tag) => normalized === normalize(tag)),
    );
    const closeTemplate = templates.find((t) =>
        t.tags.find((tag) => normalized.includes(normalize(tag))),
    );
    const defaultTemplate = templates.find(
        (v) => v.id === '84f870e9-9491-41a9-8837-d5a6f591f687',
    )!; // hardcoded app template id

    const initialTemplate =
        matchingTemplate ?? closeTemplate ?? defaultTemplate;

    const [template, setTemplate] = useState<Template>(initialTemplate);
    const [done, setDone] = useState(false);

    return (
        <ConfirmModal
            strTitle="Platform Select"
            strDescription={
                <div style={{ paddingBottom: '10px' }}>
                    Choose the platform to use for the new profile. <br />
                    <br />
                    Select "App" for native PC games, the desired emulator
                    otherwise. <br />A default has been selected based on the
                    selected collection. <br />
                </div>
            }
            onOK={async () => {
                if (!done) {
                    setDone(true);
                    const profile = await createProfile({
                        pipeline: {
                            ...template.pipeline,
                            name: collection.displayName,
                        },
                    });

                    if (profile.isOk) {
                        let id = profile.data.profile_id;

                        const savedProfile = await getProfile({
                            profile_id: id,
                        });

                        if (savedProfile.isOk) {
                            await setProfile({
                                profile: {
                                    ...savedProfile.data.profile!,
                                    tags: [collection.id],
                                },
                            });
                            Navigation.CloseSideMenus();
                            Navigation.Navigate(
                                `/deck-ds/settings/profiles/${id}`,
                            );
                            closeModal!();
                        } else {
                            setDone(false);
                            logger.toastWarn(
                                'Failed to create set:',
                                savedProfile.err.err,
                            );
                        }
                    } else {
                        setDone(true);
                        logger.toastWarn(
                            'Failed to create set:',
                            profile.err.err,
                        );
                    }
                }
            }}
            onCancel={closeModal}
            onEscKeypress={closeModal}
        >
            <Dropdown
                selectedOption={template}
                rgOptions={templates.map((d) => {
                    return {
                        label: d.pipeline.name,
                        data: d,
                    };
                })}
                onChange={(props) => {
                    setTemplate(props.data);
                }}
            />
        </ConfirmModal>
    );
}

function normalize(s: string) {
    return s.toLowerCase().replace(/\\s+/g, '');
}
