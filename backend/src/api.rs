pub mod autostart;
pub mod general;
pub mod profile;
pub mod request_handler;
pub mod secondary_app;

use schemars::{schema::RootSchema, JsonSchema};
use usdpl_back::core::serdes::Primitive;

use self::{
    autostart::AutoStartRequest,
    general::{GetSettingsResponse, SetSettingsRequest},
    profile::{
        CreateProfileRequest, CreateProfileResponse, DeleteProfileRequest, GetAppProfileRequest,
        GetAppProfileResponse, GetDefaultAppOverrideForProfileRequest,
        GetDefaultAppOverrideForProfileResponse, GetProfileRequest, GetProfileResponse,
        GetProfilesResponse, GetTemplatesResponse, PatchPipelineActionRequest,
        PatchPipelineActionResponse, ReifyPipelineRequest, ReifyPipelineResponse,
        SetAppProfileOverrideRequest, SetAppProfileSettingsRequest, SetProfileRequest,
    },
    secondary_app::GetSecondaryAppInfoResponse,
};

pub(super) type ApiParameterType = Vec<Primitive>;

#[derive(Debug, Copy, Clone)]
enum StatusCode {
    Ok,
    BadRequest,
    ServerError,
}

impl From<StatusCode> for Primitive {
    fn from(value: StatusCode) -> Self {
        Primitive::U32(match value {
            StatusCode::Ok => 200,
            StatusCode::BadRequest => 400,
            StatusCode::ServerError => 500,
        })
    }
}

trait ToResponseType {
    fn to_response(&self) -> ApiParameterType;
}

impl<T> ToResponseType for T
where
    T: serde::Serialize + std::fmt::Debug,
{
    fn to_response(&self) -> ApiParameterType {
        let json = serde_json::to_string_pretty(self)
            .unwrap_or_else(|_| panic!("{:?} should be serializable as json", self));

        log::debug!("response content: {json}");
        let primitive = Primitive::Json(json);
        vec![StatusCode::Ok.into(), primitive]
    }
}

struct ResponseErr(StatusCode, anyhow::Error);

impl ToResponseType for ResponseErr {
    fn to_response(&self) -> ApiParameterType {
        log::warn!("returning error response: {:?}", self.1);
        let primitive = Primitive::String(format!("Error: {}", self.1));
        vec![self.0.into(), primitive]
    }
}

struct ResponseOk;

impl ToResponseType for ResponseOk {
    fn to_response(&self) -> ApiParameterType {
        vec![StatusCode::Ok.into()]
    }
}

/// Marker type for generating API json schema types for ts
#[derive(JsonSchema)]
pub struct Api {
    // profile/pipeline
    pub create_profile_request: CreateProfileRequest,
    pub create_profile_response: CreateProfileResponse,
    pub get_profile_request: GetProfileRequest,
    pub get_profile_response: GetProfileResponse,
    pub set_profile_request: SetProfileRequest,
    pub delete_profile_request: DeleteProfileRequest,
    pub get_profiles_response: GetProfilesResponse,
    pub get_app_profile_request: GetAppProfileRequest,
    pub get_app_profile_response: GetAppProfileResponse,
    pub set_app_profile_settings_request: SetAppProfileSettingsRequest,
    pub set_app_profile_override_request: SetAppProfileOverrideRequest,
    pub get_default_app_override_for_profile_request: GetDefaultAppOverrideForProfileRequest,
    pub get_default_app_override_for_profile_response: GetDefaultAppOverrideForProfileResponse,
    pub patch_pipeline_action_request: PatchPipelineActionRequest,
    pub patch_pipeline_action_response: PatchPipelineActionResponse,
    pub reify_pipeline_request: ReifyPipelineRequest,
    pub reify_pipeline_response: ReifyPipelineResponse,
    pub get_templates_response: GetTemplatesResponse,

    // secondary app
    pub get_secondary_app_info: GetSecondaryAppInfoResponse,

    // settings
    pub get_settings_response: GetSettingsResponse,
    pub set_settings_request: SetSettingsRequest,

    // autostart
    pub autostart_request: AutoStartRequest,
}

impl Api {
    pub fn generate() -> RootSchema {
        schemars::schema_for!(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::Api;

    #[test]
    fn test_generate_schema() {
        Api::generate();
    }
}
