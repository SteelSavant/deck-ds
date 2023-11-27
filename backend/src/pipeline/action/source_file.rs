use std::path::PathBuf;

use anyhow::Context;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pipeline::dependency::Dependency;

use super::ActionImpl;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum SourceFile {
    Known(PathBuf),
    Custom(Option<PathBuf>),
}

impl ActionImpl for SourceFile {
    type State = PathBuf;

    fn setup(&self, ctx: &mut crate::pipeline::executor::PipelineContext) -> anyhow::Result<()> {
        match &self {
            SourceFile::Known(file) | SourceFile::Custom(Some(file)) => {
                ctx.set_state::<Self>(file.clone());
                Ok(())
            }
            SourceFile::Custom(None) => {
                None.with_context(|| "could not set source file; field not set")
            }
        }
    }

    fn get_dependencies(&self) -> Vec<crate::pipeline::dependency::Dependency> {
        let dep = match &self {
            SourceFile::Known(file) | SourceFile::Custom(Some(file)) => Dependency::Path {
                path: file.clone(),
                is_file: true,
            },
            SourceFile::Custom(None) => Dependency::FieldNotSet("Custom File".to_string()),
        };
        vec![dep]
    }
}
