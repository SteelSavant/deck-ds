pub mod general;
pub mod profile;

use anyhow::Result;
use serde::de::DeserializeOwned;
use serde_json::json;
use usdpl_back::core::serdes::Primitive;

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
                value.map(|v| match v {
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
                }),
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

pub struct ResponseErr(StatusCode, anyhow::Error);

impl ToResponseType for ResponseErr {
    fn to_response(&self) -> ApiParameterType {
        let primitive = Primitive::Json(
            json!({
               "error": format!("{}", self.1),
            })
            .to_string(),
        );
        vec![self.0.into(), primitive]
    }
}

pub struct ResponseOk;

impl ToResponseType for ResponseOk {
    fn to_response(&self) -> ApiParameterType {
        vec![StatusCode::Ok.into()]
    }
}
