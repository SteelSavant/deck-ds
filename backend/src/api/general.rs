use std::sync::{Arc, Mutex};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use usdpl_back::core::serdes::Primitive;

use crate::settings::{GlobalConfig, Settings};

use super::{
    request_handler::{log_invoke, RequestHandler},
    ResponseErr, ResponseOk, StatusCode, ToResponseType,
};

// Get settings

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GetSettingsResponse {
    global_settings: GlobalConfig,
}

pub fn get_settings(
    settings: Arc<Mutex<Settings>>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args| {
        log_invoke("get_settings", &args);

        let lock = settings
            .lock()
            .expect("request handler should not be poisoned");

        GetSettingsResponse {
            global_settings: lock.get_global_cfg(),
        }
        .to_response()
    }
}

// Set settings

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SetSettingsRequest {
    global_settings: GlobalConfig,
}

pub fn set_settings(
    request_handler: Arc<Mutex<RequestHandler>>,
    settings: Arc<Mutex<Settings>>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args| {
        log_invoke("set_settings", &args);

        let args: Result<SetSettingsRequest, _> = {
            let mut lock = request_handler
                .lock()
                .expect("request handler should not be poisoned");

            lock.resolve(args)
        };
        match args {
            Ok(args) => {
                let lock = settings
                    .lock()
                    .expect("settings mutex should not be poisoned");
                let res = lock.set_global_cfg(&args.global_settings);
                match res {
                    Ok(_) => ResponseOk.to_response(),
                    Err(err) => ResponseErr(StatusCode::ServerError, err).to_response(),
                }
            }
            Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
        }
    }
}

/// API web method to send log messages to the back-end log, callable from the front-end
pub fn log_it() -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |params| {
        if let Some(Primitive::F64(level)) = params.first() {
            if let Some(Primitive::String(msg)) = params.get(1) {
                log_msg_by_level(*level as u8, msg);
                vec![true.into()]
            } else if let Some(Primitive::Json(msg)) = params.get(1) {
                log_msg_by_level(*level as u8, msg);
                vec![true.into()]
            } else {
                log::warn!("Got log_it call with wrong/missing 2nd parameter");
                vec![false.into()]
            }
        } else {
            log::warn!("Got log_it call with wrong/missing 1st parameter");
            vec![false.into()]
        }
    }
}

fn log_msg_by_level(level: u8, msg: &str) {
    match level {
        1 => log::trace!("FRONT-END: {}", msg),
        2 => log::debug!("FRONT-END: {}", msg),
        3 => log::info!("FRONT-END: {}", msg),
        4 => log::warn!("FRONT-END: {}", msg),
        5 => log::error!("FRONT-END: {}", msg),
        _ => log::trace!("FRONT-END: {}", msg),
    }
}
