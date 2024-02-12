use std::fmt::Debug;

use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::macros::newtype_uuid;

use self::display_config::DisplayConfig;
use self::{
    cemu_layout::CemuLayout, citra_layout::CitraLayout, melonds_layout::MelonDSLayout,
    multi_window::MultiWindow, session_handler::DesktopSessionHandler, source_file::SourceFile,
    virtual_screen::VirtualScreen,
};

use super::{data::Selection, dependency::Dependency, executor::PipelineContext};
use anyhow::Result;

pub mod cemu_layout;
pub mod citra_layout;
mod desktop;
pub mod melonds_layout;
pub mod multi_window;
pub mod source_file;
pub mod virtual_screen;

pub use desktop::display_config;
pub use desktop::session_handler;

pub trait ActionImpl: DeserializeOwned + Serialize {
    /// Type of runtime state of the action
    type State: 'static + Debug + DeserializeOwned + Serialize;

    /// Essentially a more stable, hardcoded typename.
    const NAME: &'static str;

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

    /// Essentially a more stable, hardcoded typename.
    fn get_name(&self) -> &'static str {
        Self::NAME
    }
}

#[enum_delegate::register]
pub trait ErasedPipelineAction {
    fn setup(&self, ctx: &mut PipelineContext) -> Result<()>;
    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()>;
    fn get_dependencies(&self, ctx: &mut PipelineContext) -> Vec<Dependency>;
    fn get_id(&self) -> ActionId;
    /// Essentially a more stable, hardcoded typename.
    fn get_name(&self) -> &'static str;
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

    fn get_name(&self) -> &'static str {
        self.get_name()
    }
}

newtype_uuid!(ActionId);

#[derive(Debug, Clone, Serialize, PartialEq, Deserialize, JsonSchema)]
#[enum_delegate::implement(ErasedPipelineAction)]
#[serde(tag = "type", content = "value")]
pub enum Action {
    DesktopSessionHandler(DesktopSessionHandler),
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

impl Action {
    pub fn cloned_with_id(&self, id: ActionId) -> Self {
        match self {
            Action::DesktopSessionHandler(a) => {
                Action::DesktopSessionHandler(DesktopSessionHandler { id, ..*a })
            }
            Action::DisplayConfig(a) => Action::DisplayConfig(DisplayConfig { id, ..a.clone() }),
            Action::VirtualScreen(_) => Action::VirtualScreen(VirtualScreen { id }),
            Action::MultiWindow(a) => Action::MultiWindow(MultiWindow { id, ..*a }),
            Action::CitraLayout(a) => Action::CitraLayout(CitraLayout { id, ..*a }),
            Action::CemuLayout(a) => Action::CemuLayout(CemuLayout { id, ..*a }),
            Action::MelonDSLayout(a) => Action::MelonDSLayout(MelonDSLayout { id, ..*a }),
            Action::SourceFile(a) => Action::SourceFile(SourceFile { id, ..a.clone() }),
        }
    }
}
