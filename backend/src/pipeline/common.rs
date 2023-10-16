use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use super::config::{PipelineArgs, PipelineAction};


pub(super) struct Context {
    /// path to directory containing contents of decky "defaults" folder
    pub defaults_dir: PathBuf,
    /// path to directory containing user configuration files
    pub config_dir: PathBuf,
    /// configured executors
    pub executors: HashMap<String, Box<dyn PipelineActionExecutor>>

}

pub enum Dependency {
    TrueVideoWall,
}

impl Dependency {
    pub fn install(&self, ctx: &Context) -> Result<(), String> {
        match self {
            Dependency::TrueVideoWall => {
                let res = Command::new("kpackagetool5")
                    .args([
                        "-i",
                        "./true_video_wall/170914-truevideowall-1.0.kwinscript",
                    ])
                    .current_dir(&ctx.defaults_dir)
                    .output()
                    .map(|v| {
                        let err = String::from_utf8_lossy(&v.stderr);
                        if err.is_empty() | err.contains("already exists") {
                            Ok(())
                        } else {
                            Err(err.to_string())
                        }
                    })
                    .map_err(|err| err.to_string());
                match res {
                    Ok(Ok(_)) => Ok(()),
                    Ok(Err(err)) | Err(err) => Err(err),
                }
            }
        }
    }
}

pub struct PipelineActionExecutorId(pub String);

pub trait PipelineActionExecutor {
    fn id() -> PipelineActionExecutorId;

    fn definition(&self) -> PipelineAction;

    fn setup(&mut self, ctx: &Context, args: PipelineArgs) -> Result<(), String>{
        // default to no setup
        Ok(())
    }
    fn teardown(&mut self, ctx: &Context, args: PipelineArgs) -> Result<(), String> {
        // default to no teardown
        Ok(())
    }

    fn get_dependencies(&self) -> Vec<Dependency> {
        // default to no dependencies
        vec![]
    }
}
