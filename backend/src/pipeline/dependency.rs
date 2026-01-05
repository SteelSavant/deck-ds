use schemars::JsonSchema;
use serde::Serialize;
use std::path::PathBuf;
use thiserror::Error;

use anyhow::Result;
use which::which;

use crate::{secondary_app::SecondaryAppPresetId, sys::flatpak::list_installed_flatpaks};

use super::{
    action::{multi_window::secondary_app::LaunchSecondaryFlatpakApp, ActionId, ActionImpl},
    executor::PipelineContext,
};

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
    KWinScriptNotFound(String),
    #[error("required kwin script `{0}` failed to install")]
    KWinScriptFailedInstall(String),
    #[error("required field `{0}` must be set")]
    FieldNotSet(String),
    #[error("required flatpak `{0}` must be installed")]
    FlatpakNotFound(String),
    #[error("required secondary app preset `{0:?}` must exist")]
    SecondaryAppPresetNotFound(SecondaryAppPresetId),
}

pub enum Dependency {
    System(String),
    Path {
        path: PathBuf,
        is_file: bool,
    },
    KWinScript(String),
    ConfigField(String),
    /// Requires specified flatpak to exist
    Flatpak(String),
    /// Requires preset to exist
    SecondaryAppPreset(SecondaryAppPresetId),
    /// Requires active display (KDE/Kwin/X11)
    Display,
    /// EmuDeck; `PathBuf` points to required settings file
    EmuDeckSettings(PathBuf),
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
            Dependency::KWinScript(script_name) => {
                verify_system_deps(
                    &["kpackagetool5", "kreadconfig5", "kwriteconfig5", "qdbus"],
                    ctx,
                )?;

                ctx.kwin
                    .get_bundle(script_name)
                    .map(|_| ())
                    .ok_or_else(|| DependencyError::KWinScriptNotFound(script_name.clone()))
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

                if let Some(preset) = presets.get(id) {
                    match &preset.app {
                        crate::secondary_app::SecondaryApp::Flatpak(app) => {
                            LaunchSecondaryFlatpakApp {
                                id: ActionId::nil(),
                                app: app.clone(),
                                windowing_behavior: Default::default(),
                                screen_preference: Default::default(),
                            }
                            .get_dependencies(ctx)
                            .into_iter()
                            .try_for_each(|v| v.verify_config(ctx))
                        }
                    }
                } else {
                    Err(DependencyError::SecondaryAppPresetNotFound(*id))
                }
            }
            Dependency::EmuDeckSettings(path) => {
                if path.exists() {
                    Ok(())
                } else {
                    Err(DependencyError::PathNotFound(path.clone()))
                }
            }
        }
    }

    pub fn verify_or_install(&self, ctx: &PipelineContext) -> Result<(), DependencyError> {
        let res = self.verify_config(ctx);

        res.and_then(|_| {
            if let Dependency::KWinScript(script_name) = self {
                ctx.kwin.install_script(script_name).map_err(|err| {
                    log::error!("{err}");
                    DependencyError::KWinScriptFailedInstall(script_name.clone())
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
