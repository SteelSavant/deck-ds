use std::sync::{Arc, Mutex};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    pipeline::config::{PipelineDefinition, PipelineDefinitionId},
    settings::{Profile, ProfileId, Settings},
};

use super::{ParsePrimitiveAt, ResponseErr, ResponseOk, StatusCode, ToResponseType};

// Create Profile

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct CreateProfileRequest {
    profile_name: String,
    template_id: PipelineDefinitionId,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
struct CreateProfileResponse {
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
                let res = lock.create_profile(args.profile_name, &args.template_id);
                match res {
                    Ok(res) => CreateProfileResponse { profile_id: res.id }.to_response(),
                    Err(err) => ResponseErr(StatusCode::ServerError, err).to_response(),
                }
            }
            Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
        }
    }
}

// Get Profile

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct GetProfileRequest {
    profile_id: ProfileId,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
struct GetProfileResponse {
    profile: Profile,
    template: PipelineDefinition,
}

pub fn get_profile(
    settings: Arc<Mutex<Settings>>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args: super::ApiParameterType| {
        let args: Result<GetProfileRequest, _> = args.parse_at(0);
        match args {
            Ok(args) => {
                let lock = settings.lock().expect("settings mutex should be lockable");
                let profile = lock.get_profile(&args.profile_id);

                match profile {
                    Ok(profile) => {
                        let template = lock.get_template(&profile.template);
                        match template {
                            Some(template) => GetProfileResponse {
                                profile,
                                template: template.clone(),
                            }
                            .to_response(),
                            None => todo!(),
                        }
                    }
                    Err(err) => ResponseErr(StatusCode::ServerError, err).to_response(),
                }
            }
            Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
        }
    }
}

// Set Profile

#[derive(Debug, Clone, Deserialize, JsonSchema)]
struct SetProfileRequest {
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
