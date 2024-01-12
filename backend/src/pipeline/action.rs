use std::fmt::Debug;

use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::macros::newtype_uuid;

use self::{
    cemu_layout::CemuLayout, citra_layout::CitraLayout, display_restoration::DisplayRestoration,
    melonds_layout::MelonDSLayout, multi_window::MultiWindow, source_file::SourceFile,
    virtual_screen::VirtualScreen,
};

use super::{data::Selection, dependency::Dependency, executor::PipelineContext};
use anyhow::Result;

pub mod cemu_layout;
pub mod citra_layout;
pub mod display_restoration;
pub mod melonds_layout;
pub mod multi_window;
pub mod source_file;
pub mod virtual_screen;

pub trait ActionImpl: DeserializeOwned + Serialize {
    /// Type of runtime state of the action
    type State: 'static + Debug;

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
}

impl<T> ErasedPipelineAction for T
where
    T: ActionImpl + JsonSchema + Serialize + DeserializeOwned + Debug + Clone + 'static,
{
    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        log::info!("Setting up {}: {:?}", std::any::type_name::<T>(), self);
        self.setup(ctx)
    }

    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()> {
        log::info!(
            "Tearing down {}: {:?} -- State({:?})",
            std::any::type_name::<T>(),
            &self,
            ctx.get_state::<T>()
        );

        self.teardown(ctx)
    }

    fn get_dependencies(&self, ctx: &mut PipelineContext) -> Vec<Dependency> {
        self.get_dependencies(ctx)
    }
}

newtype_uuid!(ActionId);
pub type Action = v1::Action;

pub mod v1 {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    #[enum_delegate::implement(ErasedPipelineAction)]
    #[serde(tag = "type", content = "value")]
    pub enum Action {
        DisplayRestoration(DisplayRestoration),
        VirtualScreen(VirtualScreen),
        MultiWindow(MultiWindow),
        CitraLayout(CitraLayout),
        CemuLayout(CemuLayout),
        MelonDSLayout(MelonDSLayout),
        SourceFile(SourceFile),
    }
}

impl<T: Into<Action>, R> From<T> for Selection<R> {
    fn from(value: T) -> Self {
        Selection::Action(value.into())
    }
}

impl Action {
    pub fn name(&self) -> &'static str {
        match self {
            Action::DisplayRestoration(_) => "DisplayRestoration",
            Action::VirtualScreen(_) => "VirtualScreen",
            Action::MultiWindow(_) => "MultiWindow",
            Action::CitraLayout(_) => "CitraLayout",
            Action::CemuLayout(_) => "CemuLayout",
            Action::MelonDSLayout(_) => "MelonDSLayout",
            Action::SourceFile(_) => "SourceFile",
        }
    }
}
