use std::fmt::Debug;

use schemars::{
    schema::{RootSchema, Schema},
    JsonSchema,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use uuid::Uuid;

use super::{dependency::DependencyId, executor::PipelineContext};
use anyhow::Result;

pub mod display_teardown;
pub mod virtual_screen;

#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct PipelineActionId(Uuid);

pub trait PipelineAction: DeserializeOwned + Serialize {
    /// Type of runtime state of the action
    type State: 'static;

    fn id(&self) -> PipelineActionId;

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        // default to no setup
        Ok(())
    }

    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()> {
        // default to no Teardown
        Ok(())
    }

    fn get_dependencies(&self) -> Vec<DependencyId> {
        // default to no dependencies
        vec![]
    }

    fn update_from(&mut self, value: &str) -> Result<()> {
        let de: Self = serde_json::from_str(&value)?;
        *self = de;

        Ok(())
    }

    fn get_schema(&self) -> Schema;
}

#[typetag::serialize(tag = "type")]
pub trait ErasedPipelineAction {
    fn id(&self) -> PipelineActionId;
    fn setup(&self, ctx: &mut PipelineContext) -> Result<()>;
    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()>;
    fn get_dependencies(&self) -> Vec<DependencyId>;
    fn update_from(&mut self, value: &str) -> Result<()>;
    fn get_schema(&self) -> Schema;
}

#[typetag::serialize]
impl<T> ErasedPipelineAction for T
where
    T: PipelineAction + JsonSchema + Serialize + Debug + Clone,
{
    fn id(&self) -> PipelineActionId {
        self.id()
    }

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

    fn get_schema(&self) -> Schema {
        self.get_schema()
    }
}
