use std::fmt::Debug;

use schemars::{schema::RootSchema, schema_for, JsonSchema};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use self::{
    cemu_config::CemuConfig, citra_config::CitraConfig, display_config::DisplayConfig,
    melonds_config::MelonDSConfig, multi_window::MultiWindow, virtual_screen::VirtualScreen,
};

use super::{data::Selection, dependency::Dependency, executor::PipelineContext};
use anyhow::Result;

pub mod cemu_config;
pub mod citra_config;
pub mod display_config;
pub mod melonds_config;
pub mod multi_window;
pub mod virtual_screen;

pub trait ActionImpl: DeserializeOwned + Serialize {
    /// Type of runtime state of the action
    type State: 'static;

    fn setup(&self, _ctx: &mut PipelineContext) -> Result<()> {
        // default to no setup
        Ok(())
    }

    fn teardown(&self, _ctx: &mut PipelineContext) -> Result<()> {
        // default to no Teardown
        Ok(())
    }

    fn get_dependencies(&self) -> Vec<Dependency> {
        // default to no dependencies
        vec![]
    }

    fn update_from(&mut self, value: &str) -> Result<()> {
        let de: Self = serde_json::from_str(value)?;
        *self = de;

        Ok(())
    }
}

#[enum_delegate::register]
pub trait ErasedPipelineAction {
    fn setup(&self, ctx: &mut PipelineContext) -> Result<()>;
    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()>;
    fn get_dependencies(&self) -> Vec<Dependency>;
    fn update_from(&mut self, value: &str) -> Result<()>;
    fn get_schema(&self) -> RootSchema;
}

impl<T> ErasedPipelineAction for T
where
    T: ActionImpl + JsonSchema + Serialize + DeserializeOwned + Debug + Clone,
{
    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        self.setup(ctx)
    }

    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()> {
        self.teardown(ctx)
    }

    fn get_dependencies(&self) -> Vec<Dependency> {
        self.get_dependencies()
    }

    fn update_from(&mut self, value: &str) -> Result<()> {
        self.update_from(value)
    }

    fn get_schema(&self) -> RootSchema {
        schema_for!(Self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[enum_delegate::implement(ErasedPipelineAction)]
#[serde(tag = "type", content = "value")]
pub enum Action {
    DisplayConfig(DisplayConfig),
    VirtualScreen(VirtualScreen),
    MultiWindow(MultiWindow),
    CitraConfig(CitraConfig),
    CemuConfig(CemuConfig),
    MelonDSConfig(MelonDSConfig),
}

impl<T: Into<Action>, R> From<T> for Selection<R> {
    fn from(value: T) -> Self {
        Selection::Action(value.into())
    }
}
