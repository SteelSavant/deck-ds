pub mod autostart;
pub mod general;
pub mod profile;

use anyhow::Result;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use usdpl_back::core::serdes::Primitive;

use self::{
    autostart::AutoStartRequest,
    profile::{
        CreateProfileRequest, CreateProfileResponse, GetPipelineActionsResponse, GetProfileRequest,
        GetProfileResponse, GetProfilesResponse, GetTemplatesResponse, SetProfileRequest,
    },
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

trait ParsePrimitiveAt {
    fn parse_at<T: DeserializeOwned>(&self, index: usize) -> Result<T>;
}

impl ParsePrimitiveAt for ApiParameterType {
    fn parse_at<T: DeserializeOwned>(&self, index: usize) -> Result<T> {
        let value = self.get(index);
        if let Some(&Primitive::Json(json)) = value.as_ref() {
            Ok(serde_json::from_str(json)?)
        } else {
            Err(anyhow::anyhow!(
                "Parameter {:?} could not be parsed into a value of type {}",
                value.map(primitive_to_string),
                std::any::type_name::<T>(),
            ))
        }
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
        let primitive = Primitive::Json(
            serde_json::to_string_pretty(self)
                .unwrap_or_else(|_| panic!("{:?} should be serializable as json", self)),
        );
        vec![StatusCode::Ok.into(), primitive]
    }
}

struct ResponseErr(StatusCode, anyhow::Error);

impl ToResponseType for ResponseErr {
    fn to_response(&self) -> ApiParameterType {
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
pub struct __Api {
    // profile
    pub create_profile_request: CreateProfileRequest,
    pub create_profile_response: CreateProfileResponse,
    pub get_profile_request: GetProfileRequest,
    pub get_profile_response: GetProfileResponse,
    pub set_profile_request: SetProfileRequest,
    pub get_profiles_response: GetProfilesResponse,
    pub get_templates_response: GetTemplatesResponse,
    pub get_pipeline_actions_response: GetPipelineActionsResponse,

    // autostart
    pub autostart_request: AutoStartRequest,
}

fn log_invoke(method: &str, args: &[Primitive]) {
    log::debug!(
        "API invoked {method}({:?})",
        args.iter().map(primitive_to_string).collect::<Vec<_>>()
    )
}

fn primitive_to_string(v: &Primitive) -> String {
    match v {
        Primitive::Empty => "Empty".to_string(),
        Primitive::String(s) => format!("String({s})"),
        Primitive::F32(v) => format!("F32({v})"),
        Primitive::F64(v) => format!("F64({v})"),
        Primitive::U32(v) => format!("U32({v})"),
        Primitive::U64(v) => format!("U64({v})"),
        Primitive::I32(v) => format!("I32({v})"),
        Primitive::I64(v) => format!("I64({v})"),
        Primitive::Bool(v) => format!("Bool({v})"),
        Primitive::Json(v) => format!("Json({v})"),
    }
}
