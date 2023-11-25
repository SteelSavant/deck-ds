use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    asset::AssetManager,
    autostart::LoadedAutoStart,
    pipeline::{
        data::{
            ActionOrProfilePipeline, PipelineTarget, Selection, WrappedPipelineActionOrProfile,
        },
        registar::PipelineActionRegistrar,
    },
    settings::{self, AppId, Settings},
    sys::steamos_session_select::{steamos_session_select, Session},
};

use super::{ParsePrimitiveAt, ResponseErr, ResponseOk, StatusCode, ToResponseType};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AutoStartRequest {
    app: AppId,
    pipeline: ActionOrProfilePipeline,
    target: PipelineTarget,
}

pub fn autostart(
    settings: Arc<Mutex<Settings>>,
    assets_manager: AssetManager<'static>,
    home_dir: PathBuf,
    config_dir: PathBuf,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    let assets_manager = Arc::new(assets_manager);
    let home_dir = Arc::new(home_dir);
    let config_dir = Arc::new(config_dir);

    move |args: super::ApiParameterType| {
        let args: Result<AutoStartRequest, _> = args.parse_at(0);
        match args {
            Ok(args) => match args.target {
                PipelineTarget::Desktop => {
                    let lock = settings.lock().expect("settings mutex should be lockable");
                    let res = lock.set_autostart_cfg(&Some(settings::AutoStart {
                        app_id: args.app,
                        pipeline: args.pipeline,
                    }));
                    match res {
                        Ok(_) => match steamos_session_select(Session::Plasma) {
                            Ok(_) => ResponseOk.to_response(),
                            Err(err) => {
                                // remove autostart config if session select fails to avoid issues
                                // switching to desktop later
                                _ = lock.set_autostart_cfg(&None);
                                ResponseErr(StatusCode::ServerError, err).to_response()
                            }
                        },
                        Err(err) => ResponseErr(StatusCode::ServerError, err).to_response(),
                    }
                }
                PipelineTarget::Gamemode => {
                    let executor = LoadedAutoStart::new(
                        settings::AutoStart {
                            app_id: args.app,
                            pipeline: args.pipeline,
                        },
                        settings.clone(),
                        PipelineTarget::Gamemode,
                    )
                    .build_executor(
                        (*assets_manager).clone(),
                        (*home_dir).clone(),
                        (*config_dir).clone(),
                    );

                    match executor {
                        Ok(mut executor) => match executor.exec() {
                            Ok(_) => ResponseOk.to_response(),
                            Err(err) => ResponseErr(StatusCode::ServerError, err).to_response(),
                        },
                        Err(err) => ResponseErr(StatusCode::ServerError, err).to_response(),
                    }
                }
            },
            Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
        }
    }
}
