import {
    DialogButton,
    Dropdown,
    Field,
    Focusable,
    showModal,
} from 'decky-frontend-lib';
import { Fragment, ReactElement } from 'react';
import { FaPlus, FaTrash, FaX } from 'react-icons/fa6';
import {
    CategoryProfile,
    PipelineContainer,
    isCategoryProfile,
} from '../../../backend';
import HandleLoading from '../../../components/HandleLoading';
import { useModifiablePipelineContainer } from '../../../context/modifiablePipelineContext';
import useGlobalSettings from '../../../hooks/useGlobalSettings';
import useTemplates from '../../../hooks/useTemplates';
import useToplevel from '../../../hooks/useToplevel';
import { logger } from '../../../util/log';
import AddProfileTagModal from './modals/AddProfileTagModal';
import AddToplevelActionModal from './modals/AddToplevelActionModal';

export default function ProfileInfo(
    container: PipelineContainer,
): ReactElement {
    if (!isCategoryProfile(container)) {
        throw 'PipelineContainer should be CategoryProfile';
    }

    const profile: CategoryProfile = container;

    const { dispatch } = useModifiablePipelineContainer();

    const { settings } = useGlobalSettings();

    const templates = useTemplates();
    const toplevel = useToplevel();

    async function removeTag(tag: string) {
        const res = await dispatch({
            update: {
                type: 'updateTags',
                tags: profile.tags.filter((t) => t !== tag),
            },
        });

        if (!res.isOk) {
            logger.toastWarn('Failed to remove tag:', res.err.err);
        }
    }

    function addTag() {
        showModal(
            <AddProfileTagModal
                onSave={async (tag) => {
                    const unique = new Set(profile.tags);
                    unique.delete(tag);
                    return (
                        await dispatch({
                            update: {
                                type: 'updateTags',
                                tags: [...unique, tag], // set unique tags; no duplicates. If tag exists in
                            },
                        })
                    ).mapErr((v) => v.err);
                }}
            />,
        );
    }

    function addTopLevelAction() {
        showModal(
            <AddToplevelActionModal
                onSave={async (info) => {
                    return (
                        await dispatch({
                            update: {
                                type: 'addTopLevel',
                                action_id: info.id,
                            },
                        })
                    ).mapErr((v) => v.err);
                }}
            />,
        );
    }

    async function deleteToplevelAction(id: string): Promise<void> {
        // TODO::make this a confirm modal
        const res = await dispatch({
            update: {
                type: 'removeTopLevel',
                id: id,
            },
        });

        if (!res.isOk) {
            logger.toastWarn('Failed to remove toplevel action:', res.err.err);
        }
    }

    const loading =
        templates && settings && toplevel
            ? templates
                  .andThen((t) =>
                      settings.map((s) => {
                          return { templates: t, globalSettings: s };
                      }),
                  )
                  .andThen((ts) =>
                      toplevel.map((tl) => {
                          return {
                              ...ts,
                              toplevel: tl,
                          };
                      }),
                  )
            : undefined;
    // TODO::make description editable
    return (
        <HandleLoading
            value={loading}
            onOk={({ templates, globalSettings, toplevel }) => (
                <>
                    <Field
                        focusable={false}
                        label="Platform"
                        description="Platform on which the application runs. Native apps and a selection of emulators are supported."
                    >
                        <Dropdown
                            selectedOption={container.pipeline.platform.root}
                            rgOptions={templates.map((t) => {
                                return {
                                    label: t.pipeline.name,
                                    data: t.pipeline.platform.root,
                                };
                            })}
                            onChange={async (v) => {
                                const res = await dispatch({
                                    update: {
                                        type: 'updatePlatform',
                                        platform: v.data,
                                    },
                                });

                                if (!res.isOk) {
                                    if (!res.isOk) {
                                        logger.toastWarn(
                                            'Failed to update platform:',
                                            res.err.err,
                                        );
                                    }
                                }
                            }}
                        />
                    </Field>
                    <Field
                        focusable={false}
                        label={'Additional Actions'}
                        description={
                            'Additional top-level actions to run, such as launching a secondary app.'
                        }
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

                        return (
                            <Field
                                focusable={false}
                                indentLevel={1}
                                label={match.name}
                                description={match.description}
                            >
                                <DialogButton
                                    style={{
                                        backgroundColor: 'red',
                                        height: '40px',
                                        width: '40px',
                                        padding: '10px 12px',
                                        minWidth: '40px',
                                        display: 'flex',
                                        flexDirection: 'column',
                                        justifyContent: 'center',
                                        marginRight: '10px',
                                    }}
                                    onOKButton={() =>
                                        deleteToplevelAction(v.id)
                                    }
                                    onClick={() => deleteToplevelAction(v.id)}
                                >
                                    <FaTrash />
                                </DialogButton>
                            </Field>
                        );
                    })}
                    <Field
                        focusable={false}
                        label="Collections"
                        description="Steam collections for which this profile should be available."
                        bottomSeparator="none"
                    >
                        <DialogButton
                            onOKButton={addTag}
                            onClick={addTag}
                            onOKActionDescription="Add Collection"
                        >
                            <FaPlus />
                            Add Collection
                        </DialogButton>
                    </Field>
                    <Focusable
                        style={{
                            display: 'flex',
                            flexDirection: 'row',
                            flexWrap: 'wrap',
                        }}
                    >
                        {profile.tags.map((t) => (
                            <Focusable>
                                <ProfileTag tag={t} removeTag={removeTag} />
                            </Focusable>
                        ))}
                    </Focusable>
                    <Field focusable={false} />
                    <Field
                        focusable={false}
                        label="Primary Target"
                        description="Determines which target is used by the primary 'Play' button when patching the UI."
                    >
                        <Dropdown
                            selectedOption={
                                profile.pipeline.primary_target_override
                            }
                            rgOptions={[
                                {
                                    label: `Global Setting (${globalSettings.primary_ui_target})`,
                                    data: null,
                                },
                                ...['Gamemode', 'Desktop'].map((t) => {
                                    return {
                                        label: t,
                                        data: t,
                                    };
                                }),
                            ]}
                            onChange={async (option) => {
                                const res = await dispatch({
                                    update: {
                                        type: 'updatePipelineInfo',
                                        info: {
                                            primary_target_override:
                                                option.data,
                                        },
                                    },
                                });

                                if (!res.isOk) {
                                    logger.toastWarn(
                                        'Failed to update primary target:',
                                        res.err.err,
                                    );
                                }
                            }}
                        />
                    </Field>
                    {/* <Field
                        focusable={false}
                        label="Register Exit Hooks"
                        description="In desktop mode, register a button chord that exits the app when held. Disable if your controller config in Steam Input already has an exit mapping."
                    >
                        <Toggle
                            value={profile.pipeline.should_register_exit_hooks}
                            onChange={async (value) => {
                                const res = await dispatch({
                                    update: {
                                        type: 'updatePipelineInfo',
                                        info: {
                                            register_exit_hooks: value,
                                        },
                                    },
                                });

                                if (!res.isOk) {
                                    logger.toastWarn(
                                        'Failed to update exit hooks:',
                                        res.err.err,
                                    );
                                }
                            }}
                        />
                    </Field> */}
                    {/* {profile.pipeline.should_register_exit_hooks ? (
                        <>
                            <Field
                                focusable={false}
                                indentLevel={1}
                                label="Exit Hooks"
                                description="The button chord to hold to exit the app in desktop mode. These are the buttons mapped in Steam Input, not guaranteed to match the physical buttons on the controller."
                            >
                                <Dropdown
                                    selectedOption={
                                        profile.pipeline.exit_hooks_override
                                    }
                                    rgOptions={[
                                        {
                                            label: `Global Setting`,
                                            data: null,
                                        },
                                        {
                                            label: 'Custom',
                                            data:
                                                profile.pipeline
                                                    .exit_hooks_override ??
                                                globalSettings.exit_hooks,
                                        },
                                    ]}
                                    onChange={async (option) => {
                                        const res = await dispatch({
                                            update: {
                                                type: 'updatePipelineInfo',
                                                info: {
                                                    exit_hooks_override:
                                                        option.data,
                                                },
                                            },
                                        });

                                        if (!res.isOk) {
                                            logger.toastWarn(
                                                'Failed to update exit hooks:',
                                                res.err.err,
                                            );
                                        }
                                    }}
                                />
                            </Field>
                            {profile.pipeline.exit_hooks_override ? (
                                <EditHooks
                                    exitHooks={
                                        profile.pipeline.exit_hooks_override
                                    }
                                    indentLevel={1}
                                    onChange={async (hooks) => {
                                        return (
                                            await dispatch({
                                                update: {
                                                    type: 'updatePipelineInfo',
                                                    info: {
                                                        exit_hooks_override:
                                                            hooks,
                                                    },
                                                },
                                            })
                                        ).mapErr((e) => e.err);
                                    }}
                                />
                            ) : null}
                        </>
                    ) : null} */}
                    <Field
                        label="Force App Controller Layout"
                        description="Forces Steam to use the controller layout for the given app, if defined. 
                        Overrides the desktop configuration completely, and prevents controller layouts from context-switching. 
                        Useful if/when Steam fails to apply controller layouts in Desktop mode."
                    />
                    <Field label="Override for Steam Games" indentLevel={1}>
                        <Dropdown
                            selectedOption={
                                profile.pipeline.desktop_controller_layout_hack
                                    .steam_override ?? null
                            }
                            rgOptions={[null, true, false].map((v) => {
                                return {
                                    label: mapControllerHackValueToSelection(
                                        v,
                                        globalSettings.use_steam_desktop_controller_layout_hack,
                                    ),
                                    data: v,
                                };
                            })}
                            onChange={async (props) => {
                                const data: boolean | null | undefined =
                                    props.data;

                                const res = await dispatch({
                                    update: {
                                        type: 'updatePipelineInfo',
                                        info: {
                                            steam_desktop_layout_config_hack_override:
                                                data,
                                        },
                                    },
                                });

                                if (!res.isOk) {
                                    logger.toastWarn(
                                        'Failed to update update controller override:',
                                        res.err.err,
                                    );
                                }
                            }}
                        />
                    </Field>
                    <Field label="Override for Non-Steam Games" indentLevel={1}>
                        <Dropdown
                            selectedOption={
                                profile.pipeline.desktop_controller_layout_hack
                                    .nonsteam_override ?? null
                            }
                            rgOptions={[null, true, false].map((v) => {
                                return {
                                    label: mapControllerHackValueToSelection(
                                        v,
                                        globalSettings.use_nonsteam_desktop_controller_layout_hack,
                                    ),
                                    data: v,
                                };
                            })}
                            onChange={async (props) => {
                                const data: boolean | null | undefined =
                                    props.data;

                                const res = await dispatch({
                                    update: {
                                        type: 'updatePipelineInfo',
                                        info: {
                                            nonsteam_desktop_layout_config_hack_override:
                                                data,
                                        },
                                    },
                                });

                                if (!res.isOk) {
                                    logger.toastWarn(
                                        'Failed to update update controller override:',
                                        res.err.err,
                                    );
                                }
                            }}
                        />
                    </Field>
                </>
            )}
        />
    );
}

function mapControllerHackValueToSelection(
    value: boolean | null,
    global: boolean,
): string {
    if (value === false) {
        return 'Disabled';
    } else if (value === true) {
        return 'Enabled';
    } else {
        return `Global (${mapControllerHackValueToSelection(global, global)})`;
    }
}

function ProfileTag({
    tag,
    removeTag,
}: {
    tag: string;
    removeTag: (tag: string) => void;
}): ReactElement | null {
    const display = collectionStore.userCollections.find(
        (uc) => uc.id === tag,
    )?.displayName;
    return display ? (
        <div style={{ marginRight: '10px' }}>
            <DialogButton
                style={{
                    margin: '5px',
                    display: 'flex',
                    flexDirection: 'row',
                    justifyContent: 'space-between',
                }}
                onClick={() => removeTag(tag)}
                onOKButton={() => removeTag(tag)}
                onOKActionDescription="Remove Collection"
            >
                {display}
                <FaX />
            </DialogButton>
        </div>
    ) : null;
}
