use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use self::{display_teardown::DisplayTeardown, virtual_screen::VirtualScreen};
use super::{dependency::DependencyId, executor::PipelineContext};
use anyhow::Result;

pub mod display_teardown;
pub mod virtual_screen;

pub trait PipelineActionExecutor {
    /// Type of state for the executor
    type State: 'static;

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        // default to no setup
        Ok(())
    }

    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()> {
        // default to no Teardown
        Ok(())
    }

    fn get_dependencies(&self) -> Vec<DependencyId> {
        // default to no dependencies
        vec![]
    }
}

#[enum_delegate::register]
pub trait ErasedPipelineActionExecutor {
    fn setup(&self, ctx: &mut PipelineContext) -> Result<()>;
    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()>;
    fn get_dependencies(&self) -> Vec<DependencyId>;
}

impl<T> ErasedPipelineActionExecutor for T
where
    T: PipelineActionExecutor,
{
    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        self.setup(ctx)
    }

    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()> {
        self.teardown(ctx)
    }

    fn get_dependencies(&self) -> Vec<DependencyId> {
        self.get_dependencies()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[enum_delegate::implement(ErasedPipelineActionExecutor)]
pub enum PipelineAction {
    DisplayTeardown(DisplayTeardown),
    VirtualScreen(VirtualScreen),
}
