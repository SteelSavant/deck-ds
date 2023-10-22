use anyhow::Result;
use std::path::PathBuf;


use simplelog::{LevelFilter, WriteLogger};

use usdpl_back::Instance;



use clap::{Parser, Subcommand};
use deck_ds::{pipeline::{
    action::{
        display_teardown::{DisplayTeardown, RelativeLocation, TeardownExternalSettings},
        virtual_screen::VirtualScreen,
        PipelineAction,
    },
    config::{PipelineDefinition, Selection, SelectionType},
    executor::PipelineExecutor,
}, consts::{PACKAGE_NAME, PACKAGE_VERSION, PORT}, api, util};
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
}



fn main() -> Result<()> {
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
        //std::fs::File::create("/home/deck/powertools-rs.log").unwrap(),
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
            let pipeline = PipelineDefinition {
                name: "Single-Window Dual-Screen".to_string(),
                description: "Maps the internal and external monitor to a single virtual screen. Useful for emulators like melonDS which do not currently support multiple windows".to_string(),
                actions: vec![
                    Selection {
                        value: SelectionType::Single(PipelineAction::DisplayTeardown(DisplayTeardown {
                             teardown_external_settings: TeardownExternalSettings::Previous,
                             teardown_deck_location: RelativeLocation::Below
                        })),
                        optional: None,
                        hidden_in_ui: false,
                    },
                    Selection {
                    value: SelectionType::Single(PipelineAction::VirtualScreen(
                        VirtualScreen,
                    )),
                    optional: None,
                    hidden_in_ui: false,
                }],
            };
            let botw = "12146987087370911744";
            // let gungeon = "311690";
            executor.exec(botw.to_string(), &pipeline)
        }
        Modes::DisplayTest => todo!(),
        Modes::Serve => {
            let instance = Instance::new(PORT)

            .register("LOG", api::general::log_it());

            instance.run_blocking().map_err(|_| anyhow::anyhow!("server stopped unexpectedly"))
        },
    }
}
