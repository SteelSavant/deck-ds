use indexmap::IndexMap;
use schemars::{
    gen::SchemaGenerator,
    schema::{InstanceType, Schema, SchemaObject},
    JsonSchema,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::action::PipelineAction;

#[derive(Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct PipelineDefinitionId(pub Uuid);

#[derive(Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct PipelineActionDefinitionId(Uuid);

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct PipelineDefinition {
    pub name: String,
    pub id: PipelineDefinitionId,
    pub description: String,
    pub selection: Selection,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct PipelineActionDefinition {
    /// The id of the action
    pub id: PipelineActionDefinitionId,
    /// The name of the action
    pub name: String,
    /// The value of the pipeline action
    pub selection: Selection,
    /// Flags whether the selection is optional. If None, not optional. If Some(true), optional and enabled, else disabled.
    pub optional: Option<bool>,
}

impl PipelineActionDefinition {
    pub fn new(
        name: String,
        id: PipelineActionDefinitionId,
        optional: Option<bool>,
        selection: Selection,
    ) -> Self {
        Self {
            name,
            id,
            optional,
            selection,
        }
    }
}

// fn value_schema(gen: &mut SchemaGenerator) -> Schema {
//     let mut schema: SchemaObject = String::json_schema(gen).into();
//     schema.instance_type = Some(schemars::schema::SingleOrVec::Single(Box::new(
//         InstanceType::Object,
//     )));
//     schema.object = None;
//     schema.string = None;
//     schema.into()
// }

#[derive(Serialize, Deserialize, JsonSchema)]

pub enum Selection {
    Action(PipelineAction),
    OneOf {
        selection: PipelineActionDefinitionId,
        actions: IndexMap<PipelineActionDefinitionId, PipelineActionDefinition>,
    },
    AllOf(Vec<PipelineActionDefinition>),
}
