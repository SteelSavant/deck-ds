use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, Mutex},
};

use serde::de::DeserializeOwned;
use strum::EnumString;
use usdpl_back::core::serdes::Primitive;

use anyhow::{anyhow, Context, Result};

use super::{ApiParameterType, ResponseErr, ResponseOk, ToResponseType};

#[derive(Debug, Default)]
pub struct RequestHandler {
    chunks: HashMap<u64, Vec<String>>,
}

impl RequestHandler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_chunk(&mut self, index: u64, chunk: String) {
        self.chunks.entry(index).or_default().push(chunk);
    }

    pub fn resolve<T>(&mut self, args: super::ApiParameterType) -> Result<T>
    where
        T: DeserializeOwned,
    {
        args.mode_at(0).and_then(|mode| match mode {
            RequestMode::Full => args.parse_at(1),
            RequestMode::Chunked => args.u64_at(1).and_then(|index| self.resolve_chunks(index)),
        })
    }

    fn resolve_chunks<T>(&mut self, index: u64) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let chunks = self
            .chunks
            .remove(&index)
            .with_context(|| format!("Chunks for index {index} not found"))?;
        let json = chunks.join("");
        log::trace!("reconstructed json from chunks: {json}");
        Ok(serde_json::from_str(&json)?)
    }
}

pub fn chunked_request(
    request_handler: Arc<Mutex<RequestHandler>>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args: super::ApiParameterType| {
        log_invoke("chunked_request", &args);

        let mut lock = request_handler
            .lock()
            .expect("chunks should not be poisoned");
        if let Ok(index) = args.u64_at(0) {
            if let Some(Primitive::String(chunk)) = args.get(1) {
                lock.add_chunk(index, chunk.clone());

                ResponseOk.to_response()
            } else {
                ResponseErr(
                    super::StatusCode::BadRequest,
                    anyhow!("index 1 of chunk request must be a string chunk"),
                )
                .to_response()
            }
        } else {
            ResponseErr(
                super::StatusCode::BadRequest,
                anyhow!("index 0 of chunk request must be a u64 id"),
            )
            .to_response()
        }
    }
}

trait ParsePrimitiveAt {
    fn parse_at<T: DeserializeOwned>(&self, index: usize) -> Result<T>;
    fn mode_at(&self, index: usize) -> Result<RequestMode>;
    fn u64_at(&self, index: usize) -> Result<u64>;
}

#[derive(Debug, Copy, Clone, EnumString)]
pub enum RequestMode {
    Full,
    Chunked,
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

    fn mode_at(&self, index: usize) -> Result<RequestMode> {
        let value = self.get(index);
        if let Some(Primitive::String(mode)) = value {
            RequestMode::from_str(mode)
                .with_context(|| format!("could not parse {mode} to RequestMode"))
        } else {
            Err(anyhow::anyhow!(
                "Parameter {:?} could not be parsed into a RequestMode",
                value.map(primitive_to_string),
            ))
        }
    }

    fn u64_at(&self, index: usize) -> Result<u64> {
        let value = self.get(index);
        if let Some(&Primitive::U64(mode)) = value {
            Ok(mode)
        } else if let Some(&Primitive::F64(mode)) = self.get(index) {
            Ok(mode as _)
        } else if let Some(&Primitive::U32(mode)) = self.get(index) {
            Ok(mode as _)
        } else if let Some(&Primitive::F32(mode)) = self.get(index) {
            Ok(mode as _)
        } else {
            Err(anyhow::anyhow!(
                "Parameter {:?} could not be parsed into a u64",
                value.map(primitive_to_string),
            ))
        }
    }
}

pub fn log_invoke(method: &str, args: &[Primitive]) {
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
