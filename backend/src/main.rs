use std::path::PathBuf;

use clap::{Parser, Subcommand};
use deck_ds::pipeline::executor::PipelineExecutor;
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

fn main() {
    let args = Cli::parse();
    println!("got arg {:?}", args.mode);
    let mode = args.mode.unwrap_or_default();

    match mode {
        Modes::Autostart => {
            let executor =
                PipelineExecutor::new(PathBuf::from("./defaults"), PathBuf::from("todo"));

            todo!();
        }
        Modes::DisplayTest => todo!(),
        Modes::Serve => todo!(),
    }
}
