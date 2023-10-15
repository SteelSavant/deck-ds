use clap::{Parser, Subcommand};
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
    let mode = args.mode.unwrap_or_default();

    match mode {
        Modes::Autostart => {
            let executor = PipelineExecutor::new();

            todo!();
        }
        Modes::DisplayTest => todo!(),
        Modes::Serve => todo!(),
    }
}
