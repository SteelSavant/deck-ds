use std::collections::HashMap;

use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    secondary_app::{SecondaryAppManager, SecondaryAppPreset, SecondaryAppPresetId},
    sys::flatpak::{list_installed_flatpaks, FlatpakInfo},
};

use super::{request_handler::log_invoke, ToResponse};

crate::derive_api_marker!(GetSecondaryAppInfoResponse);
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GetSecondaryAppInfoResponse {
    presets: HashMap<SecondaryAppPresetId, SecondaryAppPreset>,
    installed_flatpaks: Vec<FlatpakInfo>,
}

pub fn get_secondary_app_info(
    secondary_app_manager: SecondaryAppManager,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args: super::ApiParameterType| {
        log_invoke("get_secondary_app_info", &args);

        let presets = secondary_app_manager.get_presets();
        let installed_flatpaks = list_installed_flatpaks()
            .inspect_err(|err| log::warn!("Unable to fetch installed flatpaks: {err}"))
            .unwrap_or_default();

        GetSecondaryAppInfoResponse {
            presets,
            installed_flatpaks,
        }
        .to_response()
    }
}
