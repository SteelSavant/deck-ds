use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    pipeline::{
        data::{ActionPipeline, PipelineActionDefinition, PipelineActionId, Template},
        registar::PipelineActionRegistrar,
    },
    settings::{Profile, ProfileId, Settings},
};

use super::{log_invoke, ParsePrimitiveAt, ResponseErr, ResponseOk, StatusCode, ToResponseType};

// Create Profile

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateProfileRequest {
    pipeline: ActionPipeline,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct CreateProfileResponse {
    profile_id: ProfileId,
}

pub fn create_profile(
    settings: Arc<Mutex<Settings>>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args: super::ApiParameterType| {
        let args: Result<CreateProfileRequest, _> = args.parse_at(0);
        match args {
            Ok(args) => {
                let lock = settings.lock().expect("settings mutex should be lockable");
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
    profiles: Vec<Profile>,
}

pub fn get_profiles(
    settings: Arc<Mutex<Settings>>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |_: super::ApiParameterType| {
        let lock = settings.lock().expect("settings mutex should be lockable");
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
    profile: Profile,
}

pub fn get_profile(
    settings: Arc<Mutex<Settings>>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args: super::ApiParameterType| {
        let args: Result<GetProfileRequest, _> = args.parse_at(0);
        match args {
            Ok(args) => {
                let lock = settings.lock().expect("settings mutex should be lockable");
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
    profile: Profile,
}

pub fn set_profile(
    settings: Arc<Mutex<Settings>>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args: super::ApiParameterType| {
        let args: Result<SetProfileRequest, _> = args.parse_at(0);
        match args {
            Ok(args) => {
                let lock = settings.lock().expect("settings mutex should be lockable");
                let res = lock.set_profile(&args.profile);

                match res {
                    Ok(()) => ResponseOk.to_response(),
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
    settings: Arc<Mutex<Settings>>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args: super::ApiParameterType| {
        log_invoke("get_templates", &args);
        let lock = settings.lock().expect("settings mutex should be lockable");
        let res = lock.get_templates().to_vec();

        GetTemplatesResponse { templates: res }.to_response()
    }
}

// Pipeline Actions

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GetPipelineActionsResponse {
    pipeline_actions: HashMap<PipelineActionId, PipelineActionDefinition>,
}

pub fn get_pipeline_actions(
    action_registrar: PipelineActionRegistrar,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |_: super::ApiParameterType| {
        GetPipelineActionsResponse {
            pipeline_actions: action_registrar.all().as_ref().clone(),
        }
        .to_response()
    }
}
