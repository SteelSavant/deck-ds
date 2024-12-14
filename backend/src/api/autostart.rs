use std::{
    ops::Deref,
    sync::{Arc, Mutex, MutexGuard},
};

use anyhow::Result;
use either::Either;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    autostart::LoadedAutoStart,
    db::ProfileDb,
    decky_env::DeckyEnv,
    pipeline::{
        action_registar::PipelineActionRegistrar,
        data::{Pipeline, PipelineTarget},
    },
    settings::{self, AppId, GameId, ProfileId, Settings, SteamLaunchInfo, SteamUserId64},
    sys::steamos_session_select::{check_session, steamos_session_select, Session},
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

        let session = match check_session() {
            Ok(session) => session,
            Err(err) => return ResponseErr(StatusCode::ServerError, err).to_response(),
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

                        let id = args
                            .game_id
                            .map(Either::Right)
                            .unwrap_or(Either::Left(args.app_id.clone()));

                        let launch_info = SteamLaunchInfo {
                            app_id: args.app_id,
                            user_id_64: args.user_id_64,
                            game_title: args.game_title,
                            is_steam_game: args.is_steam_game,
                        };
                        let autostart_info = AutostartInfo {
                            id,
                            pipeline,
                            env: decky_env.clone(),
                        };

                        match (args.target, session) {
                            (PipelineTarget::Desktop, Session::Gamescope) => {
                                autostart_desktop_gamescope(
                                    settings.clone(),
                                    autostart_info,
                                    launch_info,
                                )
                            }
                            (target, _) => autostart_in_place(
                                target,
                                settings.clone(),
                                autostart_info,
                                launch_info,
                            ),
                        }
                    }
                    Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
                }
            }
            Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
        }
    }
}

struct AutostartInfo {
    pipeline: Pipeline,
    id: Either<AppId, GameId>,
    env: Arc<DeckyEnv>,
}

fn autostart_in_place(
    target: PipelineTarget,
    settings: Arc<Mutex<Settings>>,
    autostart_info: AutostartInfo,
    launch_info: SteamLaunchInfo,
) -> super::ApiParameterType {
    let lock: MutexGuard<Settings> = settings.lock().expect("settings mutex should be lockable");

    let global_config = lock.get_global_cfg();

    let executor = LoadedAutoStart::new(
        settings::AutoStartConfig {
            game_id: autostart_info.id,
            pipeline: autostart_info.pipeline,
            env: autostart_info.env.deref().clone(),
            launch_info,
        },
        target,
    )
    .build_executor(global_config.clone(), autostart_info.env.clone());

    match executor {
        Ok(executor) => match executor.exec() {
            Ok(_) => ResponseOk.to_response(),
            Err(err) => ResponseErr(StatusCode::ServerError, err).to_response(),
        },
        Err(err) => ResponseErr(StatusCode::ServerError, err).to_response(),
    }
}

fn autostart_desktop_gamescope(
    settings: Arc<Mutex<Settings>>,
    autostart_info: AutostartInfo,
    launch_info: SteamLaunchInfo,
) -> super::ApiParameterType {
    let lock: MutexGuard<Settings> = settings.lock().expect("settings mutex should be lockable");

    let res = lock.set_autostart_cfg(&settings::AutoStartConfig {
        game_id: autostart_info.id,
        pipeline: autostart_info.pipeline,
        env: autostart_info.env.deref().clone(),
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
        Err(err) => ResponseErr(StatusCode::ServerError, err).to_response(),
    }
}
