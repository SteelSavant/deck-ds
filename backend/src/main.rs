use anyhow::Result;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use deck_ds::pipeline::{
    self,
    action::{
        display_teardown::{DisplayTeardown, RelativeLocation, TeardownExternalSettings},
        virtual_screen::VirtualScreen,
        PipelineAction,
    },
    config::{PipelineDefinition, Selection, SelectionType},
    executor::PipelineExecutor,
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
}

fn main() -> Result<()> {
    std::env::set_var("RUST_BACKTRACE", "1");

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
            executor.exec(botw.to_string(), &pipeline)?;
        }
        Modes::DisplayTest => todo!(),
        Modes::Serve => todo!(),
    }

    Ok(())
}
