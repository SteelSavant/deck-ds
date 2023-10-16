use std::marker::PhantomData;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use typemap::Key;

use self::display_config::DisplayConfig;
use super::{dependency::DependencyId, executor::PipelineContext};
use anyhow::Result;

pub mod display_config;
pub mod virtual_screen;

#[enum_delegate::register]
pub trait PipelineActionExecutor {
    /// Type of `Self`; necessary because of limitations in [enum_delegate]
    type S: Sized + 'static;
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

    fn get_state<'a>(
        &self,
        ctx: &'a PipelineContext,
    ) -> Option<&'a <Self as PipelineActionExecutor>::State> {
        ctx.state.get::<StateKey::<<Self as PipelineActionExecutor>::S, <Self as PipelineActionExecutor>::State>>()
    }

    fn get_state_mut<'a>(
        &self,
        ctx: &'a mut PipelineContext,
    ) -> Option<&'a mut <Self as PipelineActionExecutor>::State> {
        ctx.state.get_mut::<StateKey::<<Self as PipelineActionExecutor>::S, <Self as PipelineActionExecutor>::State>>()
    }

    fn set_state(
        &self,
        ctx: &mut PipelineContext,
        state: <Self as PipelineActionExecutor>::State,
    ) -> Option<<Self as PipelineActionExecutor>::State> {
        ctx.state.insert::<StateKey::<<Self as PipelineActionExecutor>::S, <Self as PipelineActionExecutor>::State>>(state)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[enum_delegate::implement(PipelineActionExecutor)]
pub enum PipelineAction {
    DisplayTeardown(DisplayConfig),
}

// state impl

struct StateKey<S: Sized, T>(PhantomData<S>, PhantomData<T>);

impl<S, T> Key for StateKey<S, T>
where
    S: 'static,
    T: 'static,
{
    type Value = T;
}


