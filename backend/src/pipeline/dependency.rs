use schemars::JsonSchema;
use serde::Serialize;
use std::path::PathBuf;
use thiserror::Error;

use anyhow::Result;
use which::which;

use crate::{secondary_app::SecondaryAppPresetId, sys::flatpak::list_installed_flatpaks};

use super::executor::PipelineContext;

#[derive(Error, Debug, Clone, Serialize, JsonSchema)]
#[serde(tag = "type", content = "value")]
pub enum DependencyError {
    #[error("unable to find system command `{0}`")]
    SystemCmdNotFound(String),
    #[error("required path `{0}` should be a file, not a directory")]
    PathIsNotFile(PathBuf),
    #[error("required path `{0}` should be a directory, not a file")]
    PathIsNotDir(PathBuf),
    #[error("required path `{0}` does not exist")]
    PathNotFound(PathBuf),
    #[error("required kwin script `{0}` does not exist")]
    KwinScriptNotFound(String),
    #[error("required kwin script `{0}` failed to install")]
    KwinScriptFailedInstall(String),
    #[error("required field `{0}` must be set")]
    FieldNotSet(String),
    #[error("required flatpak `{0}` must be installed")]
    FlatpakNotFound(String),
    #[error("required secondary app preset `{0:?}` must exist")]
    SecondaryAppPresetNotFound(SecondaryAppPresetId),
}

pub enum Dependency {
    System(String),
    Path { path: PathBuf, is_file: bool },
    KwinScript(String),
    ConfigField(String),
    Flatpak(String),
    SecondaryAppPreset(SecondaryAppPresetId),
    Display,
}

impl Dependency {
    pub fn verify_config(&self, ctx: &PipelineContext) -> Result<(), DependencyError> {
        match self {
            Dependency::System(program) => which(program)
                .map(|_| ())
                .map_err(|_| DependencyError::SystemCmdNotFound(program.clone())),
            Dependency::Path { path, is_file } => {
                if path.exists() {
                    if *is_file && path.is_file() {
                        Ok(())
                    } else if *is_file {
                        Err(DependencyError::PathIsNotFile(path.clone()))
                    } else {
                        Err(DependencyError::PathIsNotDir(path.clone()))
                    }
                } else {
                    Err(DependencyError::PathNotFound(path.clone()))
                }
            }
            Dependency::KwinScript(script_name) => {
                verify_system_deps(
                    &["kpackagetool5", "kreadconfig5", "kwriteconfig5", "qdbus"],
                    ctx,
                )?;

                ctx.kwin
                    .get_bundle(script_name)
                    .map(|_| ())
                    .ok_or_else(|| DependencyError::KwinScriptNotFound(script_name.clone()))
            }
            Dependency::ConfigField(field) => Err(DependencyError::FieldNotSet(field.clone())),
            Dependency::Display => verify_system_deps(&["xrandr", "cvt", "kscreen-doctor"], ctx),
            Dependency::Flatpak(app_id) => {
                Dependency::System("flatpak".into()).verify_config(ctx)?;
                let apps = list_installed_flatpaks().expect("list flatpaks should work");
                if apps.into_iter().any(|v| v.app_id == *app_id) {
                    Ok(())
                } else {
                    Err(DependencyError::FlatpakNotFound(app_id.clone()))
                }
            }
            Dependency::SecondaryAppPreset(id) => {
                let presets = ctx.secondary_app.get_presets();

                if presets.contains_key(id) {
                    Ok(())
                } else {
                    Err(DependencyError::SecondaryAppPresetNotFound(*id))
                }
            }
        }
    }

    pub fn verify_or_install(&self, ctx: &PipelineContext) -> Result<(), DependencyError> {
        let res = self.verify_config(ctx);

        res.and_then(|_| {
            if let Dependency::KwinScript(script_name) = self {
                ctx.kwin.install_script(script_name).map_err(|err| {
                    log::error!("{err}");
                    DependencyError::KwinScriptFailedInstall(script_name.clone())
                })
            } else {
                Ok(())
            }
        })
    }
}

fn verify_system_deps(deps: &[&str], ctx: &PipelineContext) -> Result<(), DependencyError> {
    for dep in deps.iter() {
        Dependency::System(dep.to_string()).verify_config(ctx)?
    }

    Ok(())
}
