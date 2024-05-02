#![feature(exit_status_error)]

use anyhow::Result;
use include_dir::{include_dir, Dir};
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
    asset::AssetManager,
    autostart::AutoStart,
    consts::{PACKAGE_NAME, PACKAGE_VERSION, PORT},
    db::ProfileDb,
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
mod macros;
mod native_model_serde_json;
pub mod pipeline;
pub mod secondary_app;
pub mod sys;
pub mod util;

pub mod autostart;
pub mod settings;

mod ui_test;

#[derive(Clone, Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    mode: Option<Modes>,
}

#[derive(Subcommand, Clone, Default, Debug, Display)]
enum Modes {
    /// runs the autostart sequence
    Autostart,
    /// runs the plugin server backend
    #[default]
    Serve,
    /// generates the schema definitions to ts type generation.
    Schema {
        /// The folder in which to store the schema
        output: String,
    },
}

static ASSETS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets");

fn main() -> Result<()> {
    // simple fn to sanity check ui
    // return ui_test::ui_test();

    let args = Cli::parse();
    let mode = args.mode.unwrap_or_default();

    let log_file_name = format!(
        "{}.{}.log",
        PACKAGE_NAME,
        match mode {
            Modes::Autostart => "autostart",
            Modes::Serve => "server",
            Modes::Schema { .. } => "schema",
        }
    );

    #[cfg(debug_assertions)]
    let log_filepath = dirs::home_dir()
        .unwrap_or_else(|| "/tmp/".into())
        .join(log_file_name);
    #[cfg(not(debug_assertions))]
    let log_filepath = std::path::Path::new("/tmp").join(log_file_name);
    WriteLogger::init(
        #[cfg(debug_assertions)]
        {
            LevelFilter::Debug
        },
        #[cfg(not(debug_assertions))]
        {
            LevelFilter::Debug
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
    let home_dir = dirs::home_dir().expect("home dir must exist");

    let config_dir = home_dir.join(".config").join(PACKAGE_NAME);
    let autostart_dir = home_dir.join(".config/autostart");

    log::info!("home dir: {:?}", home_dir);
    println!("home dir `{}`", config_dir.display());

    log::info!("Config dir `{}`", config_dir.display());
    println!("Config dir `{}`", config_dir.display());

    log::info!("Last version file: {}", crate::util::read_version_file());
    if let Err(e) = crate::util::save_version_file() {
        log::error!("Error storing version: {}", e);
    } else {
        log::info!("Updated version file succesfully");
    }

    if let Err(err) = create_dir_all(&config_dir) {
        log::error!("Error setting up config dir: {err}");
    }

    let registrar = PipelineActionRegistrar::builder().with_core().build();

    let settings = Settings::new(&env::current_exe()?, &config_dir, &autostart_dir);

    let settings = Arc::new(Mutex::new(settings));

    let assets_dir = config_dir.join("assets"); // TODO::keep assets with decky plugin, not config
    let assets_manager = AssetManager::new(&ASSETS_DIR, assets_dir.clone());
    let request_handler = Arc::new(Mutex::new(RequestHandler::new()));
    let secondary_app_manager = SecondaryAppManager::new(assets_manager.clone());

    // teardown persisted state
    match PipelineContext::load(assets_manager.clone(), home_dir.clone(), config_dir.clone()) {
        Ok(Some(loaded)) => {
            log::info!("Tearing down last executed pipeline");
            // TODO::this will cause display-dependent actions to automatically fail, but
            // this (hopefully) isn't a major problem because xrandr isn't persistent across reboots
            loaded.teardown(&mut vec![]);
        }
        Ok(None) => (),
        Err(err) => log::warn!("failed to load persisted context state: {err}"),
    }

    match mode {
        Modes::Autostart => {
            sleep(Duration::from_millis(500));

            // build the executor
            let executor = AutoStart::new(settings.clone())
                .load()
                .map(|l| l.build_executor(assets_manager, home_dir.clone(), config_dir.clone()));

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

                    // return to gamemode

                    use crate::sys::steamos_session_select::{steamos_session_select, Session};
                    let res = steamos_session_select(Session::Gamescope).and(exec_result);

                    res
                }
                None => {
                    log::info!("No autostart pipeline found. Staying on desktop.");
                    let lock = settings
                        .lock()
                        .expect("settings mutex should not be poisoned");

                    let config = lock.get_global_cfg();
                    if config.restore_displays_if_not_executing_pipeline {
                        let asset_manager = AssetManager::new(&ASSETS_DIR, assets_dir);
                        let ctx = &mut PipelineContext::new(asset_manager, home_dir, config_dir);

                        let res = config.display_restoration.desktop_only(ctx);
                        if let Err(err) = res {
                            log::error!("{err}");
                        }
                    }

                    Ok(())
                }
            }
        }
        Modes::Serve => {
            let decky_data_dir = std::env::var("DECKY_PLUGIN_RUNTIME_DIR")
                .expect("unable to find decky plugin runtime dir");
            let db_path = Path::new(&decky_data_dir).join("profiles.db");
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
                        assets_manager.clone(),
                        home_dir.clone(),
                        config_dir.clone(),
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
                // autostart
                .register(
                    "autostart",
                    crate::api::autostart::autostart(
                        request_handler.clone(),
                        profiles_db,
                        registrar.clone(),
                        settings.clone(),
                        assets_manager,
                        home_dir,
                        config_dir,
                    ),
                );

            instance
                .run_blocking()
                .map_err(|_| anyhow::anyhow!("server stopped unexpectedly"))
        }
        Modes::Schema { output } => {
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
