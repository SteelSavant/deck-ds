use anyhow::Result;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pipeline::executor::PipelineContext;

use super::DependencyExecutor;

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EmulatorWindowing;
impl EmulatorWindowing {
    pub fn id() -> super::DependencyId {
        super::DependencyId("EmulatorWindowing".to_string())
    }
}

impl DependencyExecutor for EmulatorWindowing {
    fn verify_or_install(&self, ctx: &mut PipelineContext) -> Result<()> {
        ctx.kwin.install_script("EmulatorWindowing")
    }
}
