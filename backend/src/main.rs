use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

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
        },
        config::{
            PipelineActionDefinition, PipelineActionDefinitionId, PipelineDefinition,
            PipelineDefinitionId, Selection,
        },
        executor::PipelineExecutor,
    },
    settings::autostart::AutoStartSettings,
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

    #[cfg(not(debug_assertions))]
    let config_dir = usdpl_back::api::dirs::home()
        .unwrap_or_else(|| "/tmp/".into())
        .join(".config/deck-ds");

    #[cfg(debug_assertions)]
    let config_dir = PathBuf::from(shellexpand::tilde("~/.config/deck-ds").to_string());

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

    match mode {
        Modes::Autostart => {
            let definition = PipelineDefinition {
                name: "Single-Window Dual-Screen".to_string(),
                tags: vec!["NDS".to_string()],
                id: PipelineDefinitionId::parse("d92ca87d-282f-4897-86f0-86a3af16bf3e"),
                description: "Maps the internal and external monitor to a single virtual screen. Useful for emulators like melonDS which do not currently support multiple windows".to_string(),
                selection: Selection::AllOf(vec![
                    PipelineActionDefinition {
                        optional: None,
                        id: PipelineActionDefinitionId::parse("4ff26ece-dcab-4dd3-b941-96bd96a2c045"),
                        name: "Display Configuration".to_string(),
                        description: None,
                        selection: DisplayConfig {
                                teardown_external_settings:TeardownExternalSettings::Previous,
                                teardown_deck_location:RelativeLocation::Below
                            }.into(),
                    },
                    PipelineActionDefinition {selection: VirtualScreen.into(),optional:None, id: PipelineActionDefinitionId::parse("2c843c15-fafa-4ee1-b960-e0e0aaa60882"), name: "Virtual Screen".to_string(), description: None,}])
            };

            let autostart_settings = std::fs::read_to_string(config_dir.join("autostart.json"))
                .with_context(|| "Could not find autostart configuration file")?;
            let autostart_settings: AutoStartSettings = serde_json::from_str(&autostart_settings)?;

            let mut executor = PipelineExecutor::new(PathBuf::from("/home/deck/homebrew/plugins/deck-ds/defaults"), config_dir)?;

            executor.exec(&autostart_settings, &definition)
        }
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
