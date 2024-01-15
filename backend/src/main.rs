use anyhow::Result;
use egui::Pos2;
use include_dir::{include_dir, Dir};
use std::{
    env,
    path::Path,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
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
    pipeline::{
        action::{
            multi_window::MultiWindow,
            ui_management::{DisplayRestoration, TeardownExternalSettings},
            virtual_screen::VirtualScreen,
            ActionId, ActionImpl,
        },
        action_registar::PipelineActionRegistrar,
        executor::PipelineContext,
    },
    settings::Settings,
    sys::x_display::{ModePreference, Resolution},
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
pub mod sys;
pub mod util;

pub mod autostart;
pub mod settings;

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

// fn ui_test() -> Result<()> {
//     let home_dir = usdpl_back::api::dirs::home()
//         .or_else(dirs::home_dir)
//         .expect("home dir must exist");

//     let config_dir = home_dir.join(".config").join(PACKAGE_NAME);
//     let assets_dir = config_dir.join("assets"); // TODO::keep assets with decky plugin, not config

//     let asset_manager = AssetManager::new(&ASSETS_DIR, assets_dir);
//     let mut ctx = &mut PipelineContext::new(asset_manager, home_dir, config_dir);

//     let ui = DisplayRestoration {
//         id: ActionId::nil(),
//         teardown_external_settings: TeardownExternalSettings::Preference(ModePreference {
//             resolution: sys::x_display::ModeOption::Exact(Resolution { w: 1920, h: 1080 }),
//             aspect_ratio: sys::x_display::AspectRatioOption::Any,
//             refresh: sys::x_display::ModeOption::AtLeast(60.),
//         }),
//         teardown_deck_location: pipeline::action::ui_management::RelativeLocation::Below,
//     };

//     let vscreen = MultiWindow {
//         id: ActionId::nil(),
//     };

//     let duration = Duration::from_secs(10);
//     ui.setup(&mut ctx);
//     std::thread::sleep(duration);
//     vscreen.setup(ctx);
//     std::thread::sleep(duration);
//     vscreen.teardown(ctx);
//     std::thread::sleep(duration);
//     ui.teardown(&mut ctx);
//     std::thread::sleep(duration);
//     println!("done");
//     return Ok(());
// }

fn main() -> Result<()> {
    // return ui_test();
    let args: Vec<String> = std::env::args().collect();
    log::info!("Running DeckDS from {}", args[0]);

    #[cfg(debug_assertions)]
    let log_filepath = usdpl_back::api::dirs::home()
        .unwrap_or_else(|| "/tmp/".into())
        .join(PACKAGE_NAME.to_owned() + ".log");
    #[cfg(not(debug_assertions))]
    let log_filepath = std::path::Path::new("/tmp").join(format!("{}.log", PACKAGE_NAME));
    #[cfg(debug_assertions)]
    {
        let old_log_filepath = usdpl_back::api::dirs::home()
            .unwrap_or_else(|| "/tmp/".into())
            .join(PACKAGE_NAME.to_owned() + ".log.old");

        if std::path::Path::new(&log_filepath).exists() {
            std::fs::copy(&log_filepath, old_log_filepath)
                .expect("Unable to increment logs. Do you have write permissions?");
        }
    }
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

    let home_dir = usdpl_back::api::dirs::home()
        .or_else(dirs::home_dir)
        .expect("home dir must exist");

    let config_dir = home_dir.join(".config").join(PACKAGE_NAME);
    let autostart_dir = home_dir.join(".config/autostart");

    log::info!("Starting back-end ({} v{})", PACKAGE_NAME, PACKAGE_VERSION);
    println!("Starting back-end ({} v{})", PACKAGE_NAME, PACKAGE_VERSION);

    log::debug!("Logging to: {:?}.", log_filepath);
    log::info!("Log level set to {:?}", log::max_level());
    println!("Logging to: {:?} @ {:?}", log_filepath, log::max_level());

    log::info!(
        "Current dir `{}`",
        std::env::current_dir().unwrap().display()
    );
    println!(
        "Current dir `{}`",
        std::env::current_dir().unwrap().display()
    );

    log::info!("Config dir `{}`", config_dir.display());
    println!("Config dir `{}`", config_dir.display());

    log::info!("home dir: {:?}", home_dir);
    println!("home dir `{}`", config_dir.display());

    log::info!("Last version file: {}", crate::util::read_version_file());
    if let Err(e) = crate::util::save_version_file() {
        log::error!("Error storing version: {}", e);
    } else {
        log::info!("Updated version file succesfully");
    }

    let args = Cli::parse();
    let mode = args.mode.unwrap_or_default();

    let registrar = PipelineActionRegistrar::builder().with_core().build();

    let settings = Settings::new(
        &env::current_exe()?,
        &config_dir,
        &autostart_dir,
        registrar.clone(),
    );

    let settings = Arc::new(Mutex::new(settings));

    let assets_dir = config_dir.join("assets"); // TODO::keep assets with decky plugin, not config
    let asset_manager = AssetManager::new(&ASSETS_DIR, assets_dir.clone());
    let request_handler = Arc::new(Mutex::new(RequestHandler::new()));

    match mode {
        Modes::Autostart => {
            // build the executor
            let executor = AutoStart::new(settings.clone())
                .load()
                .map(|l| l.build_executor(asset_manager, home_dir.clone(), config_dir.clone()));

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

                    let exec_result = executor.and_then(|mut e| {
                        log::debug!("Pipeline executor initialized; executing");
                        e.exec()
                    });

                    // return to gamemode
                    #[cfg(not(debug_assertions))]
                    {
                        use crate::sys::steamos_session_select::{steamos_session_select, Session};
                        steamos_session_select(Session::Gamescope).and(exec_result)
                    }
                    #[cfg(debug_assertions)]
                    {
                        exec_result // avoid gamemode switch during dev
                    }
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
            let db_path = config_dir.join("profiles.db");
            let profiles_db = Arc::new(Mutex::new(ProfileDb::new(db_path, registrar.clone())));

            let instance = Instance::new(PORT)
                .register("LOG", crate::api::general::log_it())
                .register("LOGPATH", move |_| {
                    vec![log_filepath.to_string_lossy().to_string().into()]
                })
                .register(
                    "chunked_request",
                    api::request_handler::chunked_request(request_handler.clone()),
                )
                .register(
                    "create_profile",
                    api::profile::create_profile(request_handler.clone(), profiles_db.clone()),
                )
                .register(
                    "get_profile",
                    crate::api::profile::get_profile(request_handler.clone(), profiles_db.clone()),
                )
                .register(
                    "set_profile",
                    crate::api::profile::set_profile(request_handler.clone(), profiles_db.clone()),
                )
                .register(
                    "delete_profile",
                    crate::api::profile::delete_profile(
                        request_handler.clone(),
                        profiles_db.clone(),
                    ),
                )
                .register(
                    "get_profiles",
                    crate::api::profile::get_profiles(profiles_db.clone()),
                )
                .register(
                    "reify_pipeline",
                    crate::api::profile::reify_pipeline(
                        request_handler.clone(),
                        profiles_db.clone(),
                        registrar.clone(),
                    ),
                )
                .register(
                    "get_templates",
                    crate::api::profile::get_templates(profiles_db.clone()),
                )
                // .register(
                //     "get_pipeline_actions",
                //     crate::api::profile::get_pipeline_actions(action_registrar.clone()),
                // )
                .register(
                    "autostart",
                    crate::api::autostart::autostart(
                        request_handler.clone(),
                        settings.clone(),
                        asset_manager,
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
