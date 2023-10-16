use std::process::Command;

use crate::pipeline::common::Context;

use super::DependencyExecutor;

pub struct TrueVideoWall;

impl DependencyExecutor for TrueVideoWall {
    fn install(&self, ctx: &mut Context) -> Result<(), String> {
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
