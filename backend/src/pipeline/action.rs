use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use self::display_teardown::DisplayTeardown;

use super::{common::Context, dependency::DependencyId};

pub mod display_teardown;
pub mod virtual_screen;

#[enum_delegate::register]
pub trait PipelineActionExecutor {
    fn setup(&self, ctx: &mut Context) -> Result<(), String> {
        // default to no setup
        Ok(())
    }

    fn tear_down(&self, ctx: &mut Context) -> Result<(), String> {
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
