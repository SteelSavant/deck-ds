use std::path::PathBuf;
use std::process::Command;

pub struct Context {
    /// path to directory containing contents of decky "defaults" folder
    defaults_dir: PathBuf,
    /// path to directory containing user configuration files
    config_dir: PathBuf,
}

pub enum Dependency {
    TrueVideoWall,
}

impl Dependency {
    fn install(&self, ctx: &Context) -> bool {
        match self {
            Dependency::TrueVideoWall => {
                Command::new("kpackagetool5")
                    .args(["-i", "170914-truevideowall-1.0.kwinscript"])
                    .current_dir(ctx.defaults_dir)
                    .output()
                    .map_err(|err| {
                        println!("{}, {}", err.kind(), err.to_string());
                        err.kind()
                    }) // TODO::remap "already installed" to Ok
                    .is_ok()
            }
        }
    }
}

pub trait PipelineActionExecutor {
    fn setup(args: PipelineArgs) -> bool {
        // default to no setup
        true
    }
    fn teardown(args: PipelineArgs) -> bool {
        // default to no teardown
        true
    }
    fn get_dependencies(args: PipelineArgs) -> Vec<Dependency> {
        // default to no dependencies
        vec![]
    }
}
