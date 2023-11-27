use std::path::PathBuf;

use anyhow::{anyhow, Result};
use which::which;

use super::executor::PipelineContext;

pub enum Dependency {
    System(String),
    Path { path: PathBuf, is_file: bool },
    KwinScript(String),
    Installable(Box<dyn Installable>),
    FieldNotSet(String),
}

impl Dependency {
    pub fn verify_or_install(&self, ctx: &PipelineContext) -> Result<()> {
        match self {
            Dependency::System(program) => Ok(which(program).map(|_| ())?),
            Dependency::Path { path, is_file } => {
                if path.exists() {
                    if *is_file && path.is_file() {
                        Ok(())
                    } else if *is_file {
                        Err(anyhow!("Required path is not a file: {:?}", path))
                    } else {
                        Err(anyhow!("Required path is not a directory: {:?}", path))
                    }
                } else {
                    Err(anyhow!("Required path not found: {:?}", path))
                }
            }
            Dependency::KwinScript(bundle) => ctx.kwin.install_script(bundle),
            Dependency::Installable(installable) => match installable.is_installed(ctx) {
                Ok(true) => Ok(()),
                Ok(false) => installable.install(ctx),
                Err(err) => Err(err),
            },
            Dependency::FieldNotSet(field) => Err(anyhow!("field {field} must be set")),
        }
    }
}

pub trait Installable {
    fn is_installed(&self, ctx: &PipelineContext) -> Result<bool>;
    fn install(&self, ctx: &PipelineContext) -> Result<()>;
}
