use anyhow::Result;
use include_dir::{include_dir, Dir};
use schemars::JsonSchema;
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use simplelog::{LevelFilter, WriteLogger};

use usdpl_back::Instance;

use clap::{Parser, Subcommand};
use deck_ds::{
    api::{self, __Api},
    asset::AssetManager,
    autostart::AutoStart,
    consts::{PACKAGE_NAME, PACKAGE_VERSION, PORT},
    pipeline::config::{PipelineDefinition, PipelineTarget},
    settings::{AppId, Overrides, Profile, ProfileId, Settings},
    util::{self, create_dir_all},
};
use derive_more::Display;

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
        /// The file in which to store the schema
        output: String,
    },
}

static ASSETS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets");

fn main() -> Result<()> {
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
            LevelFilter::Info
        },
        Default::default(),
        std::fs::File::create(&log_filepath).unwrap(),
    )
    .unwrap();
    log::debug!("Logging to: {:?}.", log_filepath);
    println!("Logging to: {:?}", log_filepath);

    let home_dir = usdpl_back::api::dirs::home()
        .or_else(dirs::home_dir)
        .expect("home dir must exist");

    let config_dir = home_dir.join(".config/deck-ds");

    log::info!("Starting back-end ({} v{})", PACKAGE_NAME, PACKAGE_VERSION);
    println!("Starting back-end ({} v{})", PACKAGE_NAME, PACKAGE_VERSION);
    log::info!(
        "Current dir `{}`",
        std::env::current_dir().unwrap().display()
    );
    println!(
        "Current dir `{}`",
        std::env::current_dir().unwrap().display()
    );

    println!("Config dir `{}`", config_dir.display());

    log::info!("home dir: {:?}", usdpl_back::api::dirs::home());

    log::info!("Last version file: {}", util::read_version_file());
    if let Err(e) = util::save_version_file() {
        log::error!("Error storing version: {}", e);
    } else {
        log::info!("Updated version file succesfully");
    }

    let args = Cli::parse();
    println!("got arg {:?}", args.mode);
    let mode = args.mode.unwrap_or_default();

    let settings = Settings::new(&config_dir);
    {
        // temp test code
        let template = &settings.get_templates()[2]; // Cemu Template

        let test_profile = ProfileId::from_uuid(uuid::Uuid::nil());

        settings.set_profile(&Profile {
            name: "Temp".to_string(),
            id: test_profile,
            template: template.id,
            tags: vec![],
            overrides: Overrides::default(),
        })?;

        settings.set_autostart_cfg(&Some(deck_ds::settings::AutoStart {
            app_id: AppId::new("12146987087370911744"), //botw
            profile_id: test_profile,
        }))?;
    }

    let settings = Arc::new(Mutex::new(settings));

    let assets_dir = config_dir.join("assets"); // TODO::keep assets with decky plugin, not config
    let asset_manager = AssetManager::new(&ASSETS_DIR, assets_dir);

    match mode {
        Modes::Autostart => {
            // build the executor
            let executor = AutoStart::new(settings.clone())
                .load()?
                .map(|l| l.build_executor(asset_manager, home_dir, config_dir))
                .transpose();

            // remove autostart config, so we don't end up in a loop
            let lock = settings
                .lock()
                .expect("settings mutex should be able to lock");
            lock.set_autostart_cfg(&None)?;

            // run the executor
            let exec_result = executor.and_then(|executor| match executor {
                Some(mut executor) => executor.exec(PipelineTarget::Desktop),
                None => Ok(()),
            });

            // return to gamemode
            #[cfg(not(debug_assertions))]
            {
                use deck_ds::sys::steamos_session_select::{steamos_session_select, Session};
                steamos_session_select(Session::Gamescope).and(exec_result)
            }
            #[cfg(debug_assertions)]
            {
                exec_result // avoid gamemode switch during dev
            }
        }
        Modes::Serve => {
            let instance = Instance::new(PORT)
                .register("LOG", api::general::log_it())
                .register("LOGPATH", move |_| {
                    vec![log_filepath.to_string_lossy().to_string().into()]
                })
                .register(
                    "create_profile",
                    api::profile::create_profile(settings.clone()),
                )
                .register("get_profile", api::profile::get_profile(settings.clone()))
                .register("set_profile", api::profile::set_profile(settings.clone()))
                .register(
                    "autostart",
                    api::autostart::autostart(settings, asset_manager, home_dir, config_dir),
                )
                .register(
                    "get_template_infos",
                    api::profile::get_template_infos(settings),
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
                create_dir_all(&path)?;

                /// Marker type for generating json schema types for ts
                #[derive(JsonSchema)]
                struct __Backend {
                    pub _api: __Api,
                    pub _pipeline_definition: PipelineDefinition,
                }

                let pipeline_schema = schemars::schema_for!(__Backend);
                Ok(std::fs::write(
                    path.join("schema.json"),
                    serde_json::to_string_pretty(&pipeline_schema)?,
                )?)
            }
        }
    }
}
