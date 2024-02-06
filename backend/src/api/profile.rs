use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use anyhow::Result;

use crate::{
    asset::AssetManager,
    db::ProfileDb,
    pipeline::{
        action::ActionId,
        action_registar::PipelineActionRegistrar,
        data::{Pipeline, PipelineDefinition, Template},
        dependency::DependencyError,
        executor::PipelineContext,
    },
    settings::{AppId, AppProfile, CategoryProfile, ProfileId},
};

use super::{
    request_handler::{log_invoke, RequestHandler},
    ResponseErr, ResponseOk, StatusCode, ToResponseType,
};

// Create Profile

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateProfileRequest {
    pipeline: PipelineDefinition,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct CreateProfileResponse {
    profile_id: ProfileId,
}

pub fn create_profile(
    request_handler: Arc<Mutex<RequestHandler>>,
    profiles: &'static ProfileDb,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args: super::ApiParameterType| {
        log_invoke("create_profile", &args);

        let args: Result<CreateProfileRequest, _> = {
            let mut lock = request_handler
                .lock()
                .expect("request handler should not be poisoned");

            lock.resolve(args)
        };
        match args {
            Ok(args) => {
                let res = profiles.create_profile(args.pipeline);
                match res {
                    Ok(res) => CreateProfileResponse { profile_id: res.id }.to_response(),
                    Err(err) => ResponseErr(StatusCode::ServerError, err).to_response(),
                }
            }
            Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
        }
    }
}

// Get Profiles

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

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GetProfileResponse {
    profile: Option<CategoryProfile>,
}

pub fn get_profile(
    request_handler: Arc<Mutex<RequestHandler>>,
    profiles: &'static ProfileDb,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args: super::ApiParameterType| {
        log_invoke("get_profile", &args);

        let args: Result<GetProfileRequest, _> = {
            let mut lock = request_handler
                .lock()
                .expect("request handler should not be poisoned");

            lock.resolve(args)
        };
        match args {
            Ok(args) => match profiles.get_profile(&args.profile_id) {
                Ok(profile) => GetProfileResponse { profile }.to_response(),
                Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
            },
            Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
        }
    }
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
    move |args: super::ApiParameterType| {
        log_invoke("set_profile", &args);

        let args: Result<SetProfileRequest, _> = {
            let mut lock = request_handler
                .lock()
                .expect("request handler should not be poisoned");

            lock.resolve(args)
        };
        match args {
            Ok(args) => {
                let res = profiles.set_profile(args.profile);

                match res {
                    Ok(()) => ResponseOk.to_response(),
                    Err(err) => ResponseErr(StatusCode::ServerError, err).to_response(),
                }
            }
            Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
        }
    }
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
    move |args: super::ApiParameterType| {
        log_invoke("delete_profile", &args);

        let args: Result<DeleteProfileRequest, _> = {
            let mut lock = request_handler
                .lock()
                .expect("request handler should not be poisoned");

            lock.resolve(args)
        };
        match args {
            Ok(args) => {
                let res = profiles.delete_profile(&args.profile);

                match res {
                    Ok(()) => ResponseOk.to_response(),
                    Err(err) => ResponseErr(StatusCode::ServerError, err).to_response(),
                }
            }
            Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
        }
    }
}

// Get App Profile

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct GetAppProfileRequest {
    app_id: AppId,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GetAppProfileResponse {
    app: AppProfile,
}

pub fn get_app_profile(
    request_handler: Arc<Mutex<RequestHandler>>,
    profiles: &'static ProfileDb,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args: super::ApiParameterType| {
        log_invoke("get_app_profile", &args);

        let args: Result<GetAppProfileRequest, _> = {
            let mut lock = request_handler
                .lock()
                .expect("request handler should not be poisoned");

            lock.resolve(args)
        };
        match args {
            Ok(args) => match profiles.get_app_profile(&args.app_id) {
                Ok(app) => GetAppProfileResponse { app }.to_response(),
                Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
            },
            Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
        }
    }
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
    move |args: super::ApiParameterType| {
        log_invoke("set_app_profile_settings", &args);

        let args: Result<SetAppProfileSettingsRequest, _> = {
            let mut lock = request_handler
                .lock()
                .expect("request handler should not be poisoned");

            lock.resolve(args)
        };

        match args {
            Ok(args) => {
                match profiles.set_app_profile_settings(args.app_id, args.default_profile) {
                    Ok(_) => ResponseOk.to_response(),
                    Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
                }
            }
            Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
        }
    }
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
    move |args: super::ApiParameterType| {
        log_invoke("get_app_profile", &args);

        let args: Result<SetAppProfileOverrideRequest, _> = {
            let mut lock = request_handler
                .lock()
                .expect("request handler should not be poisoned");

            lock.resolve(args)
        };

        match args {
            Ok(args) => {
                match profiles.set_app_profile_override(args.app_id, args.profile_id, args.pipeline)
                {
                    Ok(_) => ResponseOk.to_response(),
                    Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
                }
            }
            Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
        }
    }
}

// Get Default App Override Pipline for Profile

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct GetDefaultAppOverrideForProfileRequest {
    profile_id: ProfileId,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GetDefaultAppOverrideForProfileResponse {
    pipeline: Option<PipelineDefinition>,
}

pub fn get_default_app_override_pipeline_for_profile(
    request_handler: Arc<Mutex<RequestHandler>>,
    profiles: &'static ProfileDb,
    registrar: PipelineActionRegistrar,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args: super::ApiParameterType| {
        log_invoke("get_default_app_override_pipeline_for_profile", &args);

        let args: Result<GetDefaultAppOverrideForProfileRequest, _> = {
            let mut lock = request_handler
                .lock()
                .expect("request handler should not be poisoned");

            lock.resolve(args)
        };

        match args {
            Ok(args) => {
                let profile = profiles.get_profile(&args.profile_id);
                match profile {
                    Ok(profile) => GetDefaultAppOverrideForProfileResponse {
                        pipeline: profile.map(|profile| {
                            let pipeline = profile.pipeline;

                            let mut lookup = registrar.make_lookup(&pipeline.targets);

                            for action in lookup.actions.values_mut() {
                                action.profile_override = Some(args.profile_id)
                            }

                            return PipelineDefinition {
                                actions: lookup,
                                ..pipeline
                            };
                        }),
                    }
                    .to_response(),
                    Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
                }
            }
            Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
        }
    }
}

// Reify Pipeline

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ReifyPipelineRequest {
    pipeline: PipelineDefinition,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ReifyPipelineResponse {
    pipeline: Pipeline,
    config_errors: HashMap<ActionId, Vec<DependencyError>>,
}

pub fn reify_pipeline(
    request_handler: Arc<Mutex<RequestHandler>>,
    profiles: &'static ProfileDb,
    registrar: PipelineActionRegistrar,
    assets_manager: AssetManager<'static>,
    home_dir: PathBuf,
    config_dir: PathBuf,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let assets_manager = Arc::new(assets_manager);
    let home_dir = Arc::new(home_dir);
    let config_dir = Arc::new(config_dir);

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
                    let ctx = &mut PipelineContext::new(
                        (*assets_manager).clone(),
                        (*home_dir).clone(),
                        (*config_dir).clone(),
                    );
                    let config_errors = args.pipeline.check_config(ctx);
                    let res = args.pipeline.reify(&profiles, &registrar);

                    match res {
                        Ok(pipeline) => ReifyPipelineResponse {
                            pipeline,
                            config_errors,
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

// Templates

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
