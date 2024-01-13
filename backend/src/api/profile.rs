use std::sync::{Arc, Mutex};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use anyhow::Result;

use crate::{
    db::ProfileDb,
    pipeline::{
        action_registar::PipelineActionRegistrar,
        data::{Pipeline, PipelineDefinition, Template},
    },
    settings::{CategoryProfile, ProfileId},
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
    profiles: Arc<Mutex<ProfileDb>>,
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
                let lock = profiles.lock().expect("profiles mutex should be lockable");
                let res = lock.create_profile(args.pipeline);
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
    profiles: Arc<Mutex<ProfileDb>>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args: super::ApiParameterType| {
        log_invoke("get_profiles", &args);

        let lock = profiles.lock().expect("profiles mutex should be lockable");
        let res = lock.get_profiles();
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
    profiles: Arc<Mutex<ProfileDb>>,
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
            Ok(args) => {
                let lock = profiles.lock().expect("profiles mutex should be lockable");
                match lock.get_profile(&args.profile_id) {
                    Ok(profile) => GetProfileResponse { profile }.to_response(),
                    Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
                }
            }
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
    profiles: Arc<Mutex<ProfileDb>>,
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
                let lock = profiles.lock().expect("profiles mutex should be lockable");
                let res = lock.set_profile(args.profile);

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
    profiles: Arc<Mutex<ProfileDb>>,
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
                let lock = profiles.lock().expect("profiles mutex should be lockable");
                let res = lock.delete_profile(&args.profile);

                match res {
                    Ok(()) => ResponseOk.to_response(),
                    Err(err) => ResponseErr(StatusCode::ServerError, err).to_response(),
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
}

pub fn reify_pipeline(
    request_handler: Arc<Mutex<RequestHandler>>,
    profiles: Arc<Mutex<ProfileDb>>,
    registrar: PipelineActionRegistrar,
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
            Ok(args) => {
                let lock = profiles.lock().expect("profiles mutex should be lockable");
                match lock.get_profiles() {
                    Ok(profiles) => {
                        let res = args.pipeline.reify(&profiles, &registrar);
                        match res {
                            Ok(pipeline) => ReifyPipelineResponse { pipeline }.to_response(),
                            Err(err) => ResponseErr(StatusCode::ServerError, err).to_response(),
                        }
                    }
                    Err(err) => ResponseErr(StatusCode::ServerError, err).to_response(),
                }
            }
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
    profiles: Arc<Mutex<ProfileDb>>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args: super::ApiParameterType| {
        log_invoke("get_templates", &args);

        let lock = profiles.lock().expect("profiles mutex should be lockable");
        let templates = lock.get_templates().to_vec();

        GetTemplatesResponse { templates }.to_response()
    }
}

// // Pipeline Actions

// #[derive(Debug, Clone, Serialize, JsonSchema)]
// pub struct GetPipelineActionsResponse {
//     pipeline_actions: HashMap<PipelineActionId, PipelineActionDefinition>,
// }

// pub fn get_pipeline_actions(
//     action_registrar: PipelineActionRegistrar,
// ) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
//     move |_: super::ApiParameterType| {
//         GetPipelineActionsResponse {
//             pipeline_actions: action_registrar.all().as_ref().clone(),
//         }
//         .to_response()
//     }
// }
