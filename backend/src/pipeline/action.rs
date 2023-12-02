use std::fmt::Debug;

use schemars::{schema::RootSchema, schema_for, JsonSchema};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use self::{
    cemu_layout::CemuLayout, citra_layout::CitraLayout, display_config::DisplayConfig,
    melonds_layout::MelonDSLayout, multi_window::MultiWindow, source_file::SourceFile,
    virtual_screen::VirtualScreen,
};

use super::{data::Selection, dependency::Dependency, executor::PipelineContext};
use anyhow::Result;

pub mod cemu_layout;
pub mod citra_layout;
pub mod display_config;
pub mod melonds_layout;
pub mod multi_window;
pub mod source_file;
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

    fn get_dependencies(&self, _ctx: &mut PipelineContext) -> Vec<Dependency> {
        // default to no dependencies
        vec![]
    }
}

#[enum_delegate::register]
pub trait ErasedPipelineAction {
    fn setup(&self, ctx: &mut PipelineContext) -> Result<()>;
    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()>;
    fn get_dependencies(&self, ctx: &mut PipelineContext) -> Vec<Dependency>;
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

    fn get_dependencies(&self, ctx: &mut PipelineContext) -> Vec<Dependency> {
        self.get_dependencies(ctx)
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
    CitraLayout(CitraLayout),
    CemuLayout(CemuLayout),
    MelonDSLayout(MelonDSLayout),
    SourceFile(SourceFile),
}

impl<T: Into<Action>, R> From<T> for Selection<R> {
    fn from(value: T) -> Self {
        Selection::Action(value.into())
    }
}
