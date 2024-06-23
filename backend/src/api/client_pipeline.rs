use std::sync::{Arc, Mutex};

use egui::ahash::HashSet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::client_pipeline::{ClientPipelineHandler, ClientTeardownAction};

use super::{
    request_handler::{log_invoke, RequestHandler},
    ResponseErr, ResponseOk, StatusCode, ToResponseType,
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
    move |args: super::ApiParameterType| {
        log_invoke("add_client_teardown_action", &args);

        let args: Result<AddClientTeardownActionRequest, _> = {
            let mut lock = request_handler
                .lock()
                .expect("request handler should not be poisoned");

            lock.resolve(args)
        };

        match args {
            Ok(args) => {
                let mut lock = client_pipeline
                    .lock()
                    .expect("client pipeline should not be poisoned");
                let res = lock.add_client_teardown_action(args.action);

                match res {
                    Ok(_) => ResponseOk.to_response(),
                    Err(err) => ResponseErr(StatusCode::ServerError, err).to_response(),
                }
            }
            Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
        }
    }
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
    move |args: super::ApiParameterType| {
        log_invoke("remove_client_teardown_actions", &args);

        let args: Result<RemoveClientTeardownActionsRequest, _> = {
            let mut lock = request_handler
                .lock()
                .expect("request handler should not be poisoned");

            lock.resolve(args)
        };

        match args {
            Ok(args) => {
                let mut lock = client_pipeline
                    .lock()
                    .expect("client pipeline should not be poisoned");
                let res = lock.remove_client_teardown_actions(args.ids);

                match res {
                    Ok(_) => ResponseOk.to_response(),
                    Err(err) => ResponseErr(StatusCode::ServerError, err).to_response(),
                }
            }
            Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
        }
    }
}

// Get Client Teardown Actions
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
