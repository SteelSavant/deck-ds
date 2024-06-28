use std::sync::{Arc, Mutex};

use anyhow::Result;
use either::Either;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    autostart::LoadedAutoStart,
    db::ProfileDb,
    decky_env::DeckyEnv,
    pipeline::{action_registar::PipelineActionRegistrar, data::PipelineTarget},
    settings::{self, AppId, GameId, ProfileId, Settings, SteamLaunchInfo, SteamUserId64},
    sys::steamos_session_select::{steamos_session_select, Session},
};

use super::{
    request_handler::{log_invoke, RequestHandler},
    ResponseErr, ResponseOk, StatusCode, ToResponseType,
};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AutoStartRequest {
    game_id: Option<GameId>,
    app_id: AppId,
    profile_id: ProfileId,
    user_id_64: SteamUserId64,
    game_title: String,
    target: PipelineTarget,
    is_steam_game: bool,
}

pub fn autostart(
    request_handler: Arc<Mutex<RequestHandler>>,
    profile_db: &'static ProfileDb,
    registrar: PipelineActionRegistrar,
    settings: Arc<Mutex<Settings>>,
    decky_env: Arc<DeckyEnv>,
) -> impl Fn(super::ApiParameterType) -> super::ApiParameterType {
    move |args: super::ApiParameterType| {
        log_invoke("autostart", &args);

        let args: Result<AutoStartRequest, _> = {
            let mut lock = request_handler
                .lock()
                .expect("request handler should not be poisoned");

            lock.resolve(args)
        };

        match args {
            Ok(args) => {
                let definition = profile_db
                    .get_app_profile(&args.app_id)
                    .and_then(|app| {
                        app.overrides.get(&args.profile_id).cloned().ok_or_else(|| {
                            anyhow::anyhow!("Failed to find app override for profile: {:?}", app.id)
                        })
                    })
                    .or_else(|_| {
                        profile_db
                            .get_profile(&args.profile_id)
                            .and_then(|v| {
                                v.ok_or_else(|| {
                                    anyhow::anyhow!(
                                        "Failed to find profile for {:?}",
                                        &args.profile_id
                                    )
                                })
                            })
                            .map(|p| p.pipeline)
                    });

                match definition {
                    Ok(definition) => {
                        let profiles = profile_db.get_profiles().unwrap();

                        let pipeline = definition.reify(&profiles, &registrar).unwrap();

                        let env = (*decky_env).clone();

                        let id = args
                            .game_id
                            .map(Either::Right)
                            .unwrap_or(Either::Left(args.app_id.clone()));

                        let lock = settings.lock().expect("settings mutex should be lockable");
                        let global_config = lock.get_global_cfg();

                        let launch_info = SteamLaunchInfo {
                            app_id: args.app_id,
                            user_id_64: args.user_id_64,
                            game_title: args.game_title,
                            is_steam_game: args.is_steam_game,
                        };

                        match args.target {
                            PipelineTarget::Desktop => {
                                let res = lock.set_autostart_cfg(&settings::AutoStartConfig {
                                    game_id: id,
                                    pipeline,
                                    env,
                                    launch_info: launch_info.clone(),
                                });

                                match res {
                                    Ok(_) => match steamos_session_select(Session::Plasma) {
                                        Ok(_) => ResponseOk.to_response(),

                                        Err(err) => {
                                            // remove autostart config if session select fails to avoid issues
                                            // switching to desktop later
                                            _ = lock.delete_autostart_cfg();
                                            ResponseErr(StatusCode::ServerError, err).to_response()
                                        }
                                    },
                                    Err(err) => {
                                        ResponseErr(StatusCode::ServerError, err).to_response()
                                    }
                                }
                            }
                            PipelineTarget::Gamemode => {
                                let executor = LoadedAutoStart::new(
                                    settings::AutoStartConfig {
                                        game_id: id,
                                        pipeline,
                                        env,
                                        launch_info,
                                    },
                                    PipelineTarget::Gamemode,
                                )
                                .build_executor(global_config, decky_env.clone());

                                match executor {
                                    Ok(executor) => match executor.exec() {
                                        Ok(_) => ResponseOk.to_response(),
                                        Err(err) => {
                                            ResponseErr(StatusCode::ServerError, err).to_response()
                                        }
                                    },
                                    Err(err) => {
                                        ResponseErr(StatusCode::ServerError, err).to_response()
                                    }
                                }
                            }
                        }
                    }
                    Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
                }
            }
            Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
        }
    }
}
