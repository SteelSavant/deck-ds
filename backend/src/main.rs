#![feature(exit_status_error)]

use anyhow::Result;
use client_pipeline::ClientPipelineHandler;

use std::{
    env,
    path::Path,
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use simplelog::{LevelFilter, WriteLogger};

use usdpl_back::Instance;

use crate::{
    api::{request_handler::RequestHandler, Api},
    autostart::AutoStart,
    consts::{PACKAGE_NAME, PACKAGE_VERSION, PORT},
    db::ProfileDb,
    decky_env::DeckyEnv,
    pipeline::{action_registar::PipelineActionRegistrar, executor::PipelineContext},
    secondary_app::SecondaryAppManager,
    settings::Settings,
    util::create_dir_all,
};
use clap::{Parser, Subcommand};
use derive_more::Display;

pub mod api;
pub mod asset;
pub mod consts;
pub mod db;
pub mod decky_env;
mod macros;
pub mod pipeline;
pub mod secondary_app;
pub mod sys;
pub mod util;

pub mod autostart;
pub mod client_pipeline;
pub mod settings;

mod ui_test;

#[derive(Clone, Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    mode: Option<AppModes>,
}

#[derive(Subcommand, Clone, Default, Debug, Display)]
pub enum AppModes {
    /// runs the autostart sequence
    Autostart {
        /// The file storing the environment variables of the decky instance from which
        /// the bootup process was started.
        env_source: String,
    },
    /// runs the plugin server backend
    #[default]
    Serve,
    /// generates the schema definitions to ts type generation.
    Schema {
        /// The folder in which to store the schema
        output: String,
    },
}

fn main() -> Result<()> {
    // simple fn to sanity check ui
    // return ui_test::ui_test();

    set_env_vars();

    let args = Cli::parse();
    let mode = args.mode.unwrap_or_default();

    let decky_env = Arc::new(DeckyEnv::from_mode(&mode));

    let log_file_name = format!(
        "{}.{}.log",
        PACKAGE_NAME,
        match mode {
            AppModes::Autostart { .. } => "autostart",
            AppModes::Serve => "server",
            AppModes::Schema { .. } => "schema",
        }
    );

    let log_filepath = decky_env.decky_plugin_log_dir.join(log_file_name);
    let settings = Settings::new(env::current_exe()?, &decky_env);

    WriteLogger::init(
        #[cfg(debug_assertions)]
        {
            LevelFilter::Debug
        },
        #[cfg(not(debug_assertions))]
        {
            match settings.get_global_cfg().log_level {
                1 => LevelFilter::Trace,
                2 => LevelFilter::Debug,
                3 => LevelFilter::Info,
                4 => LevelFilter::Warn,
                5 => LevelFilter::Error,
                _ => LevelFilter::max(),
            }
        },
        Default::default(),
        std::fs::File::create(&log_filepath).unwrap(),
    )
    .unwrap();
    log_panics::init();

    log::info!("Starting back-end ({} v{})", PACKAGE_NAME, PACKAGE_VERSION);
    println!("Starting back-end ({} v{})", PACKAGE_NAME, PACKAGE_VERSION);

    log::debug!("Logging to: {:?}.", log_filepath);
    log::info!("Log level set to {:?}", log::max_level());
    println!("Logging to: {:?} @ {:?}", log_filepath, log::max_level());

    // usdpl_back::api::home_dir not available outside of decky, so we use the home_dir from the system and assume the user hasn't messed with things;
    // the alternative is to pass the dir as an argument when running in autostart mode.
    let home_dir = &decky_env.deck_user_home;

    let config_dir = &decky_env.decky_plugin_settings_dir;

    log::info!("home dir: {:?}", home_dir);
    println!("home dir `{}`", config_dir.display());

    log::info!("Config dir `{}`", config_dir.display());
    println!("Config dir `{}`", config_dir.display());

    log::info!(
        "Last version file: {}",
        crate::util::read_version_file(&decky_env)
    );
    if let Err(e) = crate::util::save_version_file(&decky_env) {
        log::error!("Error storing version: {}", e);
    } else {
        log::info!("Updated version file succesfully");
    }

    if let Err(err) = create_dir_all(config_dir) {
        log::error!("Error setting up config dir: {err}");
    }

    let registrar = PipelineActionRegistrar::builder().with_core().build();

    let global_config = settings.get_global_cfg();

    let settings = Arc::new(Mutex::new(settings));

    let request_handler = Arc::new(Mutex::new(RequestHandler::new()));
    let secondary_app_manager = SecondaryAppManager::new(decky_env.asset_manager());
    let client_pipeline_handler =
        Arc::new(Mutex::new(ClientPipelineHandler::new(decky_env.clone())));

    // teardown persisted state
    match PipelineContext::load(global_config, decky_env.clone()) {
        Ok(Some(mut loaded)) => {
            log::info!("Tearing down last executed pipeline");
            // TODO::this will cause display-dependent actions to automatically fail, but
            // this (hopefully) isn't a major problem because xrandr isn't persistent across reboots
            loaded.teardown(&mut vec![]);
        }
        Ok(None) => (),
        Err(err) => log::warn!("failed to load persisted context state: {err}"),
    }

    match mode {
        AppModes::Autostart { .. } => {
            let global_config = settings.lock().unwrap().get_global_cfg();

            // build the executor
            let executor = AutoStart::new(settings.clone())
                .load()
                .map(|l| l.build_executor(global_config.clone(), decky_env.clone()));

            let thread_settings = settings.clone();

            std::thread::spawn(move || loop {
                // Ensure the autostart config gets removed, to avoid launching old configs
                {
                    let lock = thread_settings
                        .lock()
                        .expect("settings mutex should be able to lock");

                    match lock.delete_autostart_cfg() {
                        Ok(_) => return,
                        Err(err) => {
                            log::error!("Failed to delete autostart config; retrying: {err}")
                        }
                    }
                }

                sleep(Duration::from_secs(1));
            });

            match executor {
                Some(executor) => {
                    log::info!("Found autostart pipeline.");

                    let exec_result = executor.and_then(|e| {
                        log::debug!("Pipeline executor initialized; executing");
                        e.exec()
                    });

                    // ensure any system effects from teardown have time to take effect
                    sleep(Duration::from_millis(200));

                    // return to gamemode

                    use crate::sys::steamos_session_select::{steamos_session_select, Session};
                    steamos_session_select(Session::Gamescope).and(exec_result)
                }
                None => {
                    log::info!("No autostart pipeline found. Staying on desktop.");

                    let lock = settings
                        .lock()
                        .expect("settings mutex should not be poisoned");

                    let config = lock.get_global_cfg();
                    if config.restore_displays_if_not_executing_pipeline {
                        let ctx =
                            &mut PipelineContext::new(None, config.clone(), decky_env.clone());

                        let res = config.display_restoration.desktop_only(ctx);
                        if let Err(err) = res {
                            log::error!("{err}");
                        }
                    }

                    Ok(())
                }
            }
        }
        AppModes::Serve => {
            decky_env.write()?;

            let db_path = decky_env.decky_plugin_runtime_dir.join("profiles.db");
            let profiles_db: &'static ProfileDb =
                Box::leak(Box::new(ProfileDb::new(db_path, registrar.clone())));

            let instance = Instance::new(PORT)
                // logging
                .register("LOG", crate::api::general::log_it())
                .register("LOGPATH", move |_| {
                    vec![log_filepath.to_string_lossy().to_string().into()]
                })
                // requests
                .register(
                    "chunked_request",
                    api::request_handler::chunked_request(request_handler.clone()),
                )
                // profile
                .register(
                    "create_profile",
                    api::profile::create_profile(request_handler.clone(), profiles_db),
                )
                .register(
                    "get_profile",
                    crate::api::profile::get_profile(request_handler.clone(), profiles_db),
                )
                .register(
                    "set_profile",
                    crate::api::profile::set_profile(request_handler.clone(), profiles_db),
                )
                .register(
                    "delete_profile",
                    crate::api::profile::delete_profile(request_handler.clone(), profiles_db),
                )
                .register(
                    "get_profiles",
                    crate::api::profile::get_profiles(profiles_db),
                )
                .register(
                    "get_app_profile",
                    crate::api::profile::get_app_profile(request_handler.clone(), profiles_db),
                )
                .register(
                    "set_app_profile_settings",
                    crate::api::profile::set_app_profile_settings(
                        request_handler.clone(),
                        profiles_db,
                    ),
                )
                .register(
                    "set_app_profile_override",
                    crate::api::profile::set_app_profile_override(
                        request_handler.clone(),
                        profiles_db,
                    ),
                )
                .register(
                    "get_default_app_override_for_profile_request",
                    crate::api::profile::get_default_app_override_pipeline_for_profile(
                        request_handler.clone(),
                        profiles_db,
                        registrar.clone(),
                    ),
                )
                .register(
                    "patch_pipeline_action",
                    crate::api::profile::patch_pipeline_action(
                        request_handler.clone(),
                        registrar.clone(),
                    ),
                )
                .register(
                    "reify_pipeline",
                    crate::api::profile::reify_pipeline(
                        request_handler.clone(),
                        profiles_db,
                        registrar.clone(),
                        decky_env.clone(),
                    ),
                )
                .register(
                    "get_toplevel",
                    crate::api::profile::get_toplevel(registrar.clone()),
                )
                .register(
                    "get_templates",
                    crate::api::profile::get_templates(profiles_db),
                )
                // client pipeline
                .register(
                    "add_client_teardown_action",
                    crate::api::client_pipeline::add_client_teardown_action(
                        request_handler.clone(),
                        client_pipeline_handler.clone(),
                    ),
                )
                .register(
                    "remove_client_teardown_actions",
                    crate::api::client_pipeline::remove_client_teardown_actions(
                        request_handler.clone(),
                        client_pipeline_handler.clone(),
                    ),
                )
                .register(
                    "get_client_teardown_actions",
                    crate::api::client_pipeline::get_client_teardown_actions(
                        client_pipeline_handler.clone(),
                    ),
                )
                // secondary app
                .register(
                    "get_secondary_app_info",
                    crate::api::secondary_app::get_secondary_app_info(secondary_app_manager),
                )
                // settings
                .register(
                    "get_settings",
                    crate::api::general::get_settings(settings.clone()),
                )
                .register(
                    "set_settings",
                    crate::api::general::set_settings(request_handler.clone(), settings.clone()),
                )
                // system info
                .register("get_display_info", api::general::get_display_info())
                .register(
                    "get_audio_device_info",
                    api::general::get_audio_device_info(decky_env.clone()),
                )
                // autostart
                .register(
                    "autostart",
                    crate::api::autostart::autostart(
                        request_handler.clone(),
                        profiles_db,
                        registrar.clone(),
                        settings.clone(),
                        decky_env.clone(),
                    ),
                )
                // test
                .register("test_error", crate::api::general::test_error());

            instance
                .run_blocking()
                .map_err(|_| anyhow::anyhow!("server stopped unexpectedly"))
        }
        AppModes::Schema { output } => {
            let path = Path::new(&output);
            if path.is_file() {
                Err(anyhow::anyhow!("output must be a directory"))
            } else {
                create_dir_all(path)?;

                let pipeline_schema = Api::generate();
                let schema_path = path.join("schema.json");

                println!("writing schema to {:?}", schema_path.canonicalize());

                Ok(std::fs::write(
                    schema_path,
                    serde_json::to_string_pretty(&pipeline_schema)?,
                )?)
            }
        }
    }
}

fn set_env_vars() {
    // TODO::consider XDG_RUNDIME_DIR and XDG_DATA_DIRS

    // flatpak links the wrong openssl lib since the default
    // LD_LIBRARY_PATH links to a generated temporary directory,
    // so we ensure "good" paths show up first.
    std::env::set_var(
        "LD_LIBRARY_PATH",
        "/usr/lib:/usr/local/lib:$LD_LIBRARY_PATH",
    );
}
