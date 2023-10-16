use serde::{Deserialize, Serialize};

use self::display_teardown::DisplayTeardown;

use super::{common::Context, dependency::Dependency};

pub mod display_teardown;
pub mod virtual_screen;

#[enum_delegate::register]
pub trait PipelineActionExecutor {
    fn setup(&self, ctx: &mut Context) -> Result<(), String> {
        // default to no setup
        Ok(())
    }

    fn teardown(&self, ctx: &mut Context) -> Result<(), String> {
        // default to no teardown
        Ok(())
    }

    fn get_dependencies(&self) -> Vec<Dependency> {
        // default to no dependencies
        vec![]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[enum_delegate::implement(PipelineActionExecutor)]
pub enum PipelineAction {
    DisplayTeardown(DisplayTeardown),
}
