use std::fmt::Debug;

use schemars::{schema::RootSchema, schema_for, JsonSchema};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use self::{
    cemu_config::CemuConfig, citra_config::CitraConfig, display_config::DisplayConfig,
    melonds_config::MelonDSConfig, multi_window::MultiWindow, virtual_screen::VirtualScreen,
};

use super::{config::Selection, dependency::DependencyId, executor::PipelineContext};
use anyhow::Result;

pub mod cemu_config;
pub mod citra_config;
pub mod display_config;
pub mod melonds_config;
pub mod multi_window;
pub mod virtual_screen;

pub trait PipelineActionImpl: DeserializeOwned + Serialize {
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

    fn get_dependencies(&self) -> Vec<DependencyId> {
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
    fn get_dependencies(&self) -> Vec<DependencyId>;
    fn update_from(&mut self, value: &str) -> Result<()>;
    fn get_schema(&self) -> RootSchema;
}

impl<T> ErasedPipelineAction for T
where
    T: PipelineActionImpl + JsonSchema + Serialize + DeserializeOwned + Debug + Clone,
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

    fn update_from(&mut self, value: &str) -> Result<()> {
        self.update_from(value)
    }

    fn get_schema(&self) -> RootSchema {
        schema_for!(Self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[enum_delegate::implement(ErasedPipelineAction)]
pub enum PipelineAction {
    DisplayConfig(DisplayConfig),
    VirtualScreen(VirtualScreen),
    MultiWindow(MultiWindow),
    CitraConfig(CitraConfig),
    CemuConfig(CemuConfig),
    MelonDSConfig(MelonDSConfig),
}

impl<T: Into<PipelineAction>> From<T> for Selection {
    fn from(value: T) -> Self {
        Selection::Action(value.into())
    }
}
