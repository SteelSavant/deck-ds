use anyhow::Result;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use simplelog::{LevelFilter, WriteLogger};

use usdpl_back::Instance;

use clap::{Parser, Subcommand};
use deck_ds::{
    api,
    consts::{PACKAGE_NAME, PACKAGE_VERSION, PORT},
    pipeline::{
        action::{
            display_teardown::{DisplayConfig, RelativeLocation, TeardownExternalSettings},
            virtual_screen::VirtualScreen,
            PipelineAction,
        },
        config::{PipelineActionDefinition, PipelineDefinition, PipelineDefinitionId, Selection},
        executor::PipelineExecutor,
    },
    util,
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
    /// collects information about currently connected displays
    DisplayTest,
    /// runs the plugin server backend
    #[default]
    Serve,
    /// generates the schema definitions to ts type generation.
    Schema {
        /// The file in which to store the schema
        output: String,
    },
}

fn main() -> Result<()> {
    let pipeline = PipelineDefinition {
        name: "Single-Window Dual-Screen".to_string(),
        id: PipelineDefinitionId(Uuid::new_v4()),
        description: "Maps the internal and external monitor to a single virtual screen. Useful for emulators like melonDS which do not currently support multiple windows".to_string(),
        selection: Selection::AllOf(vec![
            PipelineActionDefinition {
                optional:None,
                id: todo!(),
                name: todo!(),
                selection:
                    DisplayConfig {
                        teardown_external_settings:TeardownExternalSettings::Previous,
                        teardown_deck_location:RelativeLocation::Below
                    }.into(),
            },
            PipelineActionDefinition {selection: VirtualScreen.into(),optional:None, id: todo!(), name: todo!(), }])
    };

    #[cfg(debug_assertions)]
    let log_filepath = usdpl_back::api::dirs::home()
        .unwrap_or_else(|| "/tmp/".into())
        .join(PACKAGE_NAME.to_owned() + ".log");
    #[cfg(not(debug_assertions))]
    let log_filepath = std::path::Path::new("/tmp").join(format!("{}.log", PACKAGE_NAME));
    #[cfg(debug_assertions)]
    let old_log_filepath = usdpl_back::api::dirs::home()
        .unwrap_or_else(|| "/tmp/".into())
        .join(PACKAGE_NAME.to_owned() + ".log.old");
    #[cfg(debug_assertions)]
    {
        if std::path::Path::new(&log_filepath).exists() {
            std::fs::copy(&log_filepath, &old_log_filepath)
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

    match mode {
        Modes::Autostart => {
            let mut executor =
                PipelineExecutor::new(PathBuf::from("./defaults"), PathBuf::from("todo"))?;

            let botw = "12146987087370911744";
            // let gungeon = "311690";
            // executor.exec(botw.to_string(), &pipeline)
            todo!()
        }
        Modes::DisplayTest => todo!(),
        Modes::Serve => {
            let instance = Instance::new(PORT)
                .register("LOG", api::general::log_it())
                .register("LOGPATH", move |_| {
                    vec![log_filepath.to_string_lossy().to_string().into()]
                });

            instance
                .run_blocking()
                .map_err(|_| anyhow::anyhow!("server stopped unexpectedly"))
        }
        Modes::Schema { output } => {
            let path = Path::new(&output);
            if path.is_dir() {
                Err(anyhow::anyhow!("output must be a file"))
            } else {
                let schema = schemars::schema_for!(PipelineDefinition);
                Ok(std::fs::write(
                    path,
                    serde_json::to_string_pretty(&schema)?,
                )?)
            }
        }
    }
}
