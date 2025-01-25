use std::sync::{Arc, Mutex};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use usdpl_back::core::serdes::Primitive;

use crate::{
    config::{ConfigLocator, GlobalConfig},
    decky_env::DeckyEnv,
    sys::{
        audio::{get_audio_sinks, get_audio_sources, AudioDeviceInfo},
        display_info::{self, DisplayInfo, DisplayMode},
    },
};

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
    settings: Arc<ConfigLocator>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args| {
        log_invoke("get_settings", &args);

        GetSettingsResponse {
            global_settings: settings.get_global_cfg(),
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
    config: Arc<ConfigLocator>,
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
                let res = config.set_global_cfg(&args.global_settings);
                match res {
                    Ok(_) => ResponseOk.to_response(),
                    Err(err) => ResponseErr(StatusCode::ServerError, err).to_response(),
                }
            }
            Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
        }
    }
}

/// Get Display Info

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GetDisplayInfoResponse {
    /// All available displays, sorted from most recently used to least recently used
    available_displays: Vec<DisplayInfo>,
}

pub fn get_display_info() -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args| {
        log_invoke("get_display_info", &args);
        GetDisplayInfoResponse {
            available_displays: display_info::get_display_info().unwrap_or_default(),
        }
        .to_response()
    }
}

/// Get Audio Device Info

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
