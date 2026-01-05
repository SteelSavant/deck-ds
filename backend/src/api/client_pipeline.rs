use std::sync::{Arc, Mutex};

use egui::ahash::HashSet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::client_pipeline::{ClientPipelineHandler, ClientTeardownAction};

use super::{
    request_handler::{exec_with_args, log_invoke, RequestHandler},
    ResponseErr, ResponseOk, StatusCode, ToResponse,
};

// Add Client Teardown Action

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AddClientTeardownActionRequest {
    action: ClientTeardownAction,
}

pub fn add_client_teardown_action(
    request_handler: Arc<Mutex<RequestHandler>>,
    client_pipeline: Arc<Mutex<ClientPipelineHandler>>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    exec_with_args(
        "add_client_teardown_action",
        request_handler,
        move |args: AddClientTeardownActionRequest| {
            let mut lock = client_pipeline
                .lock()
                .expect("client pipeline should not be poisoned");
            lock.add_client_teardown_action(args.action)
                .map(|_| ResponseOk)
                .map_err(|err| ResponseErr(StatusCode::ServerError, err))
        },
    )
}

// Remove Client Teardown Actions

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RemoveClientTeardownActionsRequest {
    ids: HashSet<String>,
}

pub fn remove_client_teardown_actions(
    request_handler: Arc<Mutex<RequestHandler>>,
    client_pipeline: Arc<Mutex<ClientPipelineHandler>>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    exec_with_args(
        "remove_client_teardown_actions",
        request_handler,
        move |args: RemoveClientTeardownActionsRequest| {
            let mut lock = client_pipeline
                .lock()
                .expect("client pipeline should not be poisoned");
            lock.remove_client_teardown_actions(args.ids)
                .map(|_| ResponseOk)
                .map_err(|err| ResponseErr(StatusCode::ServerError, err))
        },
    )
}

// Get Client Teardown Actions
crate::derive_api_marker!(GetClientTeardownActionsResponse);
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct GetClientTeardownActionsResponse {
    actions: Vec<ClientTeardownAction>,
}

pub fn get_client_teardown_actions(
    client_pipeline: Arc<Mutex<ClientPipelineHandler>>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args: super::ApiParameterType| {
        log_invoke("remove_client_teardown_actions", &args);

        let lock = client_pipeline
            .lock()
            .expect("client pipeline should not be poisoned");
        let actions = lock.get_client_teardown_actions();

        GetClientTeardownActionsResponse { actions }.to_response()
    }
}
