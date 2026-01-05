use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use anyhow::Result;

use crate::{
    db::ProfileDb,
    decky_env::DeckyEnv,
    pipeline::{
        action::{Action, ActionId, ErasedPipelineAction},
        action_registar::PipelineActionRegistrar,
        data::{
            ConfigSelection, Pipeline, PipelineActionId, PipelineDefinition, PipelineDefinitionId,
            PipelineTarget, RuntimeSelection, Template, TopLevelDefinition, TopLevelId,
        },
        dependency::DependencyError,
        executor::PipelineContext,
    },
    settings::{AppId, AppProfile, CategoryProfile, ProfileId},
};

use super::{
    request_handler::{exec_with_args, log_invoke, RequestHandler},
    ResponseErr, ResponseOk, StatusCode, ToResponse,
};

// Create Profile

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateProfileRequest {
    pipeline: PipelineDefinition,
}

crate::derive_api_marker!(CreateProfileResponse);
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct CreateProfileResponse {
    profile_id: ProfileId,
}

pub fn create_profile(
    request_handler: Arc<Mutex<RequestHandler>>,
    profiles: &'static ProfileDb,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    exec_with_args(
        "create_profile",
        request_handler,
        |args: CreateProfileRequest| {
            profiles
                .create_profile(args.pipeline)
                .map(|res| CreateProfileResponse { profile_id: res.id })
                .map_err(|err| ResponseErr(StatusCode::ServerError, err))
        },
    )
}

// Get Profiles

crate::derive_api_marker!(GetProfilesResponse);
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GetProfilesResponse {
    profiles: Vec<CategoryProfile>,
}

pub fn get_profiles(
    profiles: &'static ProfileDb,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args: super::ApiParameterType| {
        log_invoke("get_profiles", &args);

        let res = profiles.get_profiles();

        match res {
            Ok(profiles) => GetProfilesResponse { profiles }.to_response(),
            Err(err) => ResponseErr(StatusCode::ServerError, err).to_response(),
        }
    }
}

// Get Profile
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct GetProfileRequest {
    profile_id: ProfileId,
}

crate::derive_api_marker!(GetProfileResponse);
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GetProfileResponse {
    profile: Option<CategoryProfile>,
}

pub fn get_profile(
    request_handler: Arc<Mutex<RequestHandler>>,
    profiles: &'static ProfileDb,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    exec_with_args("get_profile", request_handler, |args: GetProfileRequest| {
        profiles
            .get_profile(&args.profile_id)
            .map(|profile| GetProfileResponse { profile })
            .map_err(|err| ResponseErr(StatusCode::ServerError, err))
    })
}

// Set Profile

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SetProfileRequest {
    profile: CategoryProfile,
}

pub fn set_profile(
    request_handler: Arc<Mutex<RequestHandler>>,
    profiles: &'static ProfileDb,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    exec_with_args("set_profile", request_handler, |args: SetProfileRequest| {
        profiles
            .set_profile(args.profile)
            .map(|_| ResponseOk)
            .map_err(|err| ResponseErr(StatusCode::ServerError, err))
    })
}

// Delete Profile

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DeleteProfileRequest {
    profile: ProfileId,
}

pub fn delete_profile(
    request_handler: Arc<Mutex<RequestHandler>>,
    profiles: &'static ProfileDb,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    exec_with_args(
        "delete_profile",
        request_handler,
        |args: DeleteProfileRequest| {
            profiles
                .delete_profile(&args.profile)
                .map(|_| ResponseOk)
                .map_err(|err| ResponseErr(StatusCode::ServerError, err))
        },
    )
}

// Get App Profile

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct GetAppProfileRequest {
    app_id: AppId,
}

crate::derive_api_marker!(GetAppProfileResponse);
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GetAppProfileResponse {
    app: AppProfile,
}

pub fn get_app_profile(
    request_handler: Arc<Mutex<RequestHandler>>,
    profiles: &'static ProfileDb,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    exec_with_args(
        "get_app_profile",
        request_handler,
        |args: GetAppProfileRequest| {
            profiles
                .get_app_profile(&args.app_id)
                .map(|app| GetAppProfileResponse { app })
                .map_err(|err| ResponseErr(StatusCode::ServerError, err))
        },
    )
}

// Set App Settings

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SetAppProfileSettingsRequest {
    app_id: AppId,
    default_profile: Option<ProfileId>,
}

pub fn set_app_profile_settings(
    request_handler: Arc<Mutex<RequestHandler>>,
    profiles: &'static ProfileDb,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    exec_with_args(
        "set_app_profile_settings",
        request_handler,
        |args: SetAppProfileSettingsRequest| {
            profiles
                .set_app_profile_settings(args.app_id, args.default_profile)
                .map(|_| ResponseOk)
                .map_err(|err| ResponseErr(StatusCode::ServerError, err))
        },
    )
}

// Set App Profile Override

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SetAppProfileOverrideRequest {
    app_id: AppId,
    profile_id: ProfileId,
    pipeline: PipelineDefinition,
}

pub fn set_app_profile_override(
    request_handler: Arc<Mutex<RequestHandler>>,
    profiles: &'static ProfileDb,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    exec_with_args(
        "set_app_profile_override",
        request_handler,
        |args: SetAppProfileOverrideRequest| {
            profiles
                .set_app_profile_override(args.app_id, args.profile_id, args.pipeline)
                .map(|_| ResponseOk)
                .map_err(|err| ResponseErr(StatusCode::ServerError, err))
        },
    )
}

// Get Default App Override Pipline for Profile

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct GetDefaultAppOverrideForProfileRequest {
    profile_id: ProfileId,
}

crate::derive_api_marker!(GetDefaultAppOverrideForProfileResponse);
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GetDefaultAppOverrideForProfileResponse {
    pipeline: Option<PipelineDefinition>,
}

pub fn get_default_app_override_pipeline_for_profile(
    request_handler: Arc<Mutex<RequestHandler>>,
    profiles: &'static ProfileDb,
    registrar: PipelineActionRegistrar,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    exec_with_args(
        "get_default_app_override_pipeline_for_profile",
        request_handler,
        move |args: GetDefaultAppOverrideForProfileRequest| {
            let profile = profiles.get_profile(&args.profile_id);
            match profile {
                Ok(profile) => Ok(GetDefaultAppOverrideForProfileResponse {
                    pipeline: profile.map(|profile| {
                        let pipeline = profile.pipeline;

                        let mut platform_lookup = registrar.make_lookup(&pipeline.platform.root);

                        for (id, action) in platform_lookup.actions.iter_mut() {
                            action.profile_override = Some(args.profile_id);
                            // override the visibility with the profile visibility, since the QAM can't actually set it
                            if let Some(profile_action) = pipeline.platform.actions.actions.get(id)
                            {
                                action.copy_qam_values(profile_action);
                            }
                        }

                        let toplevel = pipeline
                            .toplevel
                            .iter()
                            .map(|v| {
                                let mut lookup = registrar.make_lookup(&v.root);

                                for (id, action) in lookup.actions.iter_mut() {
                                    action.profile_override = Some(args.profile_id);
                                    // override the visibility with the profile visibility, since the QAM can't actually set it
                                    if let Some(profile_action) = v.actions.actions.get(id) {
                                        action.copy_qam_values(profile_action);
                                    }
                                }

                                TopLevelDefinition {
                                    actions: lookup,
                                    ..v.clone()
                                }
                            })
                            .collect();

                        PipelineDefinition {
                            id: PipelineDefinitionId::nil(),
                            platform: TopLevelDefinition {
                                actions: platform_lookup,
                                ..pipeline.platform
                            },
                            toplevel,
                            ..pipeline
                        }
                    }),
                }),
                Err(err) => Err(ResponseErr(StatusCode::ServerError, err)),
            }
        },
    )
}

// Patch Pipeline
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "value")]
pub enum PipelineActionUpdate {
    UpdateEnabled { is_enabled: bool },
    UpdateProfileOverride { profile_override: Option<ProfileId> },
    UpdateOneOf { selection: PipelineActionId },
    UpdateAction { action: Action },
    UpdateVisibleOnQAM { is_visible: bool },
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct PatchPipelineActionRequest {
    pipeline: PipelineDefinition,
    toplevel_id: TopLevelId,
    action_id: PipelineActionId,
    target: PipelineTarget,
    update: PipelineActionUpdate,
}

crate::derive_api_marker!(PatchPipelineActionResponse);
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct PatchPipelineActionResponse {
    pipeline: PipelineDefinition,
}

pub fn patch_pipeline_action(
    request_handler: Arc<Mutex<RequestHandler>>,
    registrar: PipelineActionRegistrar,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    exec_with_args(
        "patch_pipeline_action",
        request_handler,
        move |args: PatchPipelineActionRequest| {
            let registered = registrar.get(&args.action_id, args.target);
            if let Some(registered) = registered {
                let mut pipeline = args.pipeline;
                let tl = if pipeline.platform.id == args.toplevel_id {
                    Some(&mut pipeline.platform)
                } else {
                    pipeline
                        .toplevel
                        .iter_mut()
                        .find(|v| v.id == args.toplevel_id)
                };

                match tl {
                    Some(tl) => {
                        let current_id = tl
                            .actions
                            .get(&args.action_id, args.target)
                            .and_then(|v| match &v.selection {
                                ConfigSelection::Action(a) => Some(a.get_id()),
                                _ => None,
                            })
                            .unwrap_or(ActionId::nil());
                        let pipeline_action = tl
                            .actions
                            .actions
                            .entry(args.action_id.clone())
                            .or_insert(registered.clone().settings.into());

                        match args.update {
                            PipelineActionUpdate::UpdateEnabled { is_enabled } => {
                                pipeline_action.enabled = Some(is_enabled);
                            }
                            PipelineActionUpdate::UpdateProfileOverride { profile_override } => {
                                log::info!(
                                    "profile override for {:?} set to {:?}",
                                    args.action_id,
                                    profile_override
                                );

                                pipeline_action.profile_override = profile_override
                            }
                            PipelineActionUpdate::UpdateOneOf { selection } => {
                                pipeline_action.selection = ConfigSelection::OneOf { selection }
                            }
                            PipelineActionUpdate::UpdateAction { action } => {
                                pipeline_action.selection =
                                    ConfigSelection::Action(action.cloned_with_id(current_id))
                            }
                            PipelineActionUpdate::UpdateVisibleOnQAM { is_visible } => {
                                pipeline_action.is_visible_on_qam = is_visible
                            }
                        }

                        if let ConfigSelection::OneOf { selection } = &mut pipeline_action.selection
                        {
                            let reified = registrar.get(selection, args.target).unwrap().id.clone();
                            *selection = reified
                        }

                        Ok(PatchPipelineActionResponse { pipeline })
                    }
                    // TODO::Notfound
                    None => Err(ResponseErr(
                        StatusCode::BadRequest,
                        anyhow::anyhow!("toplevel {:?} to registered", args.toplevel_id),
                    )),
                }
            } else {
                Err(ResponseErr(
                    StatusCode::BadRequest,
                    anyhow::anyhow!("action {:?} not registered", args.action_id),
                ))
            }
        },
    )
}

// Reify Pipeline

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ReifyPipelineRequest {
    pipeline: PipelineDefinition,
}

crate::derive_api_marker!(ReifyPipelineResponse);
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ReifyPipelineResponse {
    pipeline: Pipeline,
    config_errors: HashMap<PipelineActionId, Vec<DependencyError>>,
}

pub fn reify_pipeline(
    request_handler: Arc<Mutex<RequestHandler>>,
    profiles: &'static ProfileDb,
    registrar: PipelineActionRegistrar,
    decky_env: Arc<DeckyEnv>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args: super::ApiParameterType| {
        log_invoke("reify_pipeline", &args);

        let args: Result<ReifyPipelineRequest, _> = {
            let mut lock = request_handler
                .lock()
                .expect("request handler should not be poisoned");

            lock.resolve(args)
        };
        match args {
            Ok(args) => match profiles.get_profiles() {
                Ok(profiles) => {
                    let ctx =
                        &mut PipelineContext::new(None, Default::default(), decky_env.clone());
                    let res = args.pipeline.reify(&profiles, ctx, &registrar);

                    match res {
                        Ok(pipeline) => ReifyPipelineResponse {
                            config_errors: check_config_errors(&pipeline, ctx),
                            pipeline,
                        }
                        .to_response(),
                        Err(err) => ResponseErr(StatusCode::ServerError, err).to_response(),
                    }
                }
                Err(err) => ResponseErr(StatusCode::ServerError, err).to_response(),
            },
            Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
        }
    }
}

fn check_config_errors(
    pipeline: &Pipeline,
    ctx: &mut PipelineContext,
) -> HashMap<PipelineActionId, Vec<DependencyError>> {
    fn collect_actions<'a>(
        selection: &'a RuntimeSelection,
        parent_id: &PipelineActionId,
    ) -> Vec<(PipelineActionId, &'a Action)> {
        match selection {
            RuntimeSelection::Action(action) => {
                vec![(parent_id.clone(), action)]
            }
            RuntimeSelection::OneOf { selection, actions } => {
                let action = actions
                    .iter()
                    .find(|a| a.id.no_variant() == selection.no_variant()) // TODO:: These shouldn't be required to filter by no_variant
                    .unwrap_or_else(|| {
                        panic!("selected action {selection:?} should exist in {actions:?}")
                    });

                collect_actions(&action.selection, &action.id)
            }
            RuntimeSelection::AllOf(actions) | RuntimeSelection::AllOfErased(actions) => actions
                .iter()
                .flat_map(|a| collect_actions(&a.selection, &a.id))
                .collect(),
        }
    }

    let deps: HashMap<_, _> = pipeline
        .targets
        .iter()
        .flat_map(|(_target, selection)| {
            let actions = collect_actions(selection, &PipelineActionId::new("root"));

            let mut kv = vec![];
            for a in actions.into_iter() {
                kv.push((a.0, a.1.get_dependencies(ctx)));
            }

            kv
        })
        .filter(|v| !v.1.is_empty())
        .collect();

    deps.into_iter()
        .map(|(id, dep)| {
            (
                id,
                dep.into_iter()
                    .filter_map(|d| d.verify_config(ctx).err())
                    .collect(),
            )
        })
        .collect()
}

// Toplevel

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ToplevelInfo {
    pub id: PipelineActionId,
    pub name: String,
    pub description: Option<String>,
}

crate::derive_api_marker!(GetTopLevelResponse);
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GetTopLevelResponse {
    toplevel: Vec<ToplevelInfo>,
}

pub fn get_toplevel(
    registrar: PipelineActionRegistrar,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args: super::ApiParameterType| {
        log_invoke("get_toplevel", &args);

        let mut toplevel = registrar
            .toplevel()
            .into_values()
            .map(|v| ToplevelInfo {
                id: v.id.clone(),
                name: v.name.clone(),
                description: v.description.clone(),
            })
            .collect::<Vec<_>>();

        toplevel.sort_by_key(|k| k.name.clone());

        GetTopLevelResponse { toplevel }.to_response()
    }
}

// Templates

crate::derive_api_marker!(GetTemplatesResponse);
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GetTemplatesResponse {
    templates: Vec<Template>,
}

pub fn get_templates(
    profiles: &'static ProfileDb,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args: super::ApiParameterType| {
        log_invoke("get_templates", &args);

        let lock = profiles;
        let templates = lock.get_templates().to_vec();

        GetTemplatesResponse { templates }.to_response()
    }
}
