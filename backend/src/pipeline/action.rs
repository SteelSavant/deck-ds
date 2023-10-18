use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use self::display_teardown::DisplayTeardown;
use super::{dependency::DependencyId, executor::PipelineContext};
use anyhow::Result;

pub mod display_teardown;
pub mod virtual_screen;

#[enum_delegate::register]
#[allow(unused_variables)]
pub trait PipelineActionExecutor {
    /// Type of state for the executor
    type State: 'static;

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        // default to no setup
        Ok(())
    }

    fn tear_down(&self, ctx: &mut PipelineContext) -> Result<()> {
        // default to no Teardown
        Ok(())
    }

    fn get_dependencies(&self) -> Vec<DependencyId> {
        // default to no dependencies
        vec![]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[enum_delegate::implement(PipelineActionExecutor)]
pub enum PipelineAction {
    DisplayTeardown(DisplayTeardown),
}
