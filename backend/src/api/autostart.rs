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
    settings::{self, AppId, GameId, ProfileId, Settings, SteamUserId64},
    sys::{
        steam,
        steamos_session_select::{steamos_session_select, Session},
    },
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
                    .ok()
                    .and_then(|app| app.overrides.get(&args.profile_id).cloned())
                    .or_else(|| {
                        profile_db
                            .get_profile(&args.profile_id)
                            .ok()
                            .flatten()
                            .map(|p| p.pipeline)
                    })
                    .unwrap(); // TODO::error handling

                let profiles = profile_db.get_profiles().unwrap();

                let pipeline = definition.reify(&profiles, &registrar).unwrap();

                let env = (*decky_env).clone();

                let id = args
                    .game_id
                    .map(Either::Right)
                    .unwrap_or(Either::Left(args.app_id.clone()));

                let lock = settings.lock().expect("settings mutex should be lockable");

                match args.target {
                    PipelineTarget::Desktop => {
                        let use_controller_hack = pipeline
                            .desktop_layout_config_hack_override
                            .unwrap_or_else(|| {
                                lock.get_global_cfg().use_desktop_controller_layout_hack
                            });

                        let res = lock.set_autostart_cfg(&settings::AutoStartConfig {
                            game_id: id,
                            pipeline,
                            env,
                        });

                        match res {
                            Ok(_) => {
                                if use_controller_hack {
                                    log::debug!("setting desktop controller config hack");
                                    let res = steam::set_desktop_controller_hack(
                                        &args.user_id_64,
                                        &args.app_id,
                                        &args.game_title,
                                        decky_env.steam_dir(),
                                    );

                                    if let Err(err) = res {
                                        log::warn!(
                                            "unable to set desktop controller hack: {err:#?}"
                                        )
                                    }
                                }

                                match steamos_session_select(Session::Plasma) {
                                    Ok(_) => ResponseOk.to_response(),

                                    Err(err) => {
                                        // remove autostart config if session select fails to avoid issues
                                        // switching to desktop later
                                        _ = lock.delete_autostart_cfg();
                                        ResponseErr(StatusCode::ServerError, err).to_response()
                                    }
                                }
                            }
                            Err(err) => ResponseErr(StatusCode::ServerError, err).to_response(),
                        }
                    }
                    PipelineTarget::Gamemode => {
                        let executor = LoadedAutoStart::new(
                            settings::AutoStartConfig {
                                game_id: id,
                                pipeline,
                                env,
                            },
                            PipelineTarget::Gamemode,
                        )
                        .build_executor(decky_env.clone());

                        let global_config = lock.get_global_cfg();

                        match executor {
                            Ok(executor) => match executor.exec(&global_config) {
                                Ok(_) => ResponseOk.to_response(),
                                Err(err) => ResponseErr(StatusCode::ServerError, err).to_response(),
                            },
                            Err(err) => ResponseErr(StatusCode::ServerError, err).to_response(),
                        }
                    }
                }
            }
            Err(err) => ResponseErr(StatusCode::BadRequest, err).to_response(),
        }
    }
}
