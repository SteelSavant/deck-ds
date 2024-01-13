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

    fn get_id(&self) -> ActionId;
}

#[enum_delegate::register]
pub trait ErasedPipelineAction {
    fn setup(&self, ctx: &mut PipelineContext) -> Result<()>;
    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()>;
    fn get_dependencies(&self, ctx: &mut PipelineContext) -> Vec<Dependency>;
    fn get_id(&self) -> ActionId;
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

    fn get_id(&self) -> ActionId {
        self.get_id()
    }
}

newtype_uuid!(ActionId);

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

    pub fn cloned_with_id(&self, id: ActionId) -> Self {
        match self {
            Action::DisplayRestoration(a) => {
                Action::DisplayRestoration(DisplayRestoration { id, ..a.clone() })
            }
            Action::VirtualScreen(a) => Action::VirtualScreen(VirtualScreen { id, ..a.clone() }),
            Action::MultiWindow(a) => Action::MultiWindow(MultiWindow { id, ..a.clone() }),
            Action::CitraLayout(a) => Action::CitraLayout(CitraLayout { id, ..a.clone() }),
            Action::CemuLayout(a) => Action::CemuLayout(CemuLayout { id, ..a.clone() }),
            Action::MelonDSLayout(a) => Action::MelonDSLayout(MelonDSLayout { id, ..a.clone() }),
            Action::SourceFile(a) => Action::SourceFile(SourceFile { id, ..a.clone() }),
        }
    }
}
