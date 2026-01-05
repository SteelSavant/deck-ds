use std::sync::{Arc, Mutex};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use usdpl_back::core::serdes::Primitive;

use crate::{
    decky_env::DeckyEnv,
    settings::{GlobalConfig, Settings},
    sys::{
        audio::{get_audio_sinks, get_audio_sources, AudioDeviceInfo},
        display_info::{self, DisplayValues},
    },
};

use super::{
    request_handler::{exec_with_args, log_invoke, RequestHandler},
    ResponseErr, ResponseOk, StatusCode, ToResponse,
};

// Get settings
crate::derive_api_marker!(GetSettingsResponse);
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
    exec_with_args(
        "set_settings",
        request_handler,
        move |args: SetSettingsRequest| {
            let lock = settings
                .lock()
                .expect("settings mutex should not be poisoned");

            lock.set_global_cfg(&args.global_settings)
                .map(|_| ResponseOk)
                .map_err(|err| ResponseErr(StatusCode::ServerError, err))
        },
    )
}

// Get Display Info

crate::derive_api_marker!(GetDisplayInfoResponse);
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GetDisplayInfoResponse {
    available_values: Vec<DisplayValues>,
}

pub fn get_display_info() -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args| {
        log_invoke("get_display_info", &args);
        GetDisplayInfoResponse {
            available_values: display_info::get_display_info().unwrap_or_default(),
        }
        .to_response()
    }
}

// Get Audio Device Info

crate::derive_api_marker!(GetAudioDeviceInfoResponse);
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GetAudioDeviceInfoResponse {
    sources: Vec<AudioDeviceInfo>,
    sinks: Vec<AudioDeviceInfo>,
}

pub fn get_audio_device_info(
    decky_env: Arc<DeckyEnv>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args| {
        log_invoke("get_audio_device_info", &args);
        let sources = get_audio_sources(&decky_env);
        let sinks = get_audio_sinks(&decky_env);
        GetAudioDeviceInfoResponse { sources, sinks }.to_response()
    }
}

/// Error Msg Test

pub fn test_error() -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |_| {
        ResponseErr(
            StatusCode::ServerError,
            anyhow::anyhow!("An expected error occurred")
                .context("May you never see a real one.")
                .context("This is a test."),
        )
        .to_response()
    }
}

/// API web method to send log messages to the back-end log, callable from the front-end

pub fn log_it() -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args| {
        log_invoke("LOG", &args);

        if let Some(Primitive::F64(level)) = args.first() {
            if let Some(Primitive::String(msg)) = args.get(1) {
                log_msg_by_level(*level as u8, msg);
                vec![true.into()]
            } else if let Some(Primitive::Json(msg)) = args.get(1) {
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
