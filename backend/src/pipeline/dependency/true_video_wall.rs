use anyhow::Result;
use std::process::Command;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pipeline::executor::PipelineContext;

use super::DependencyExecutor;

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TrueVideoWall;
impl TrueVideoWall {
    pub fn id() -> super::DependencyId {
        super::DependencyId("TrueVideoWall".to_string())
    }
}

impl DependencyExecutor for TrueVideoWall {
    fn install(&self, ctx: &mut PipelineContext) -> Result<()> {
        ctx.kwin.install_script("TrueVideoWall")
    }
}
