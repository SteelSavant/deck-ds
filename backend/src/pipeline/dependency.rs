use schemars::JsonSchema;
use serde::Serialize;
use std::{collections::HashMap, path::PathBuf};
use thiserror::Error;

use anyhow::Result;
use which::which;

use super::{
    action::{ActionId, ErasedPipelineAction},
    data::{PipelineDefinition, Selection},
    executor::PipelineContext,
};

#[derive(Error, Debug, Clone, Serialize, JsonSchema)]
#[serde(tag = "type", content = "value")]
pub enum DependencyError {
    #[error("unable to find system command `{0}`")]
    System(String),
    #[error("required path `{0}` should be a file, not a directory")]
    PathIsNotFile(PathBuf),
    #[error("required path `{0}` should be a directory, not a file")]
    PathIsNotDir(PathBuf),
    #[error("required path `{0}` does not exist")]
    PathShouldExist(PathBuf),
    #[error("required kwin script `{0}` does not exist")]
    KwinScriptNotFound(String),
    #[error("required kwin script `{0}` failed to install")]
    KwinScriptFailedInstall(String),
    #[error("required field `{0}` must be set")]
    FieldNotSet(String),
}

pub enum Dependency {
    System(String),
    Path { path: PathBuf, is_file: bool },
    KwinScript(String),
    ConfigField(String),
}

impl Dependency {
    pub fn verify_config(&self, ctx: &PipelineContext) -> Result<(), DependencyError> {
        match self {
            Dependency::System(program) => which(program)
                .map(|_| ())
                .map_err(|_| DependencyError::System(program.clone())),
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
                    Err(DependencyError::PathShouldExist(path.clone()))
                }
            }
            Dependency::KwinScript(bundle_name) => ctx
                .kwin
                .get_bundle(bundle_name)
                .map(|_| ())
                .ok_or_else(|| DependencyError::KwinScriptNotFound(bundle_name.clone())),
            Dependency::ConfigField(field) => Err(DependencyError::FieldNotSet(field.clone())),
        }
    }

    pub fn verify_or_install(&self, ctx: &PipelineContext) -> Result<(), DependencyError> {
        let res = self.verify_config(ctx);

        res.and_then(|_| {
            if let Dependency::KwinScript(bundle_name) = self {
                ctx.kwin.install_script(bundle_name).map_err(|err| {
                    log::error!("{err}");
                    DependencyError::KwinScriptFailedInstall(bundle_name.clone())
                })
            } else {
                Ok(())
            }
        })
    }
}

impl PipelineDefinition {
    pub fn check_config(
        &self,
        ctx: &mut PipelineContext,
    ) -> HashMap<ActionId, Vec<DependencyError>> {
        self.actions
            .actions
            .iter()
            .filter_map(|(_, a)| {
                if let Selection::Action(a) = &a.selection {
                    let errors = a
                        .get_dependencies(ctx)
                        .into_iter()
                        .filter_map(|d| d.verify_config(ctx).err())
                        .collect::<Vec<_>>();
                    if errors.is_empty() {
                        None
                    } else {
                        Some((a.get_id(), errors))
                    }
                } else {
                    None
                }
            })
            .collect()
    }
}
