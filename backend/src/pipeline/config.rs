use indexmap::IndexMap;
use schemars::{
    gen::SchemaGenerator,
    schema::{InstanceType, Schema, SchemaObject},
    JsonSchema,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::action::{ErasedPipelineAction, PipelineActionId};

#[derive(Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct PipelineDefinitionId(pub Uuid);

#[derive(Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct PipelineActionDefinitionId(Uuid);

#[derive(Serialize, JsonSchema)]
pub struct PipelineDefinition {
    pub name: String,
    pub id: PipelineDefinitionId,
    pub description: String,
    pub actions: Vec<PipelineActionDefinition>,
}

#[derive(Serialize, JsonSchema)]
pub struct PipelineActionDefinition {
    /// The id of the action
    pub id: PipelineActionDefinitionId,
    /// The name of the action
    pub name: String,
    /// The schema of the pipeline action, serialized to json
    schema: String,
    /// The value of the pipeline action
    pub selection: SelectionType,
    /// Flags whether the selection is optional. If None, not optional. If Some(true), optional and enabled, else disabled.
    pub optional: Option<bool>,
}

impl PipelineActionDefinition {
    pub fn new<A: ErasedPipelineAction>(name: String, id: PipelineActionDefinitionId, selection: SelectionType, optional: Option<bool>) -> Self {
        let schema = serde_json::to_string_pretty(selection.)?
        Self {
            schema:  
            name,
            id,
            selection,
            optional,
        }
    }
}

fn value_schema(gen: &mut SchemaGenerator) -> Schema {
    let mut schema: SchemaObject = String::json_schema(gen).into();
    schema.instance_type = Some(schemars::schema::SingleOrVec::Single(Box::new(
        InstanceType::Object,
    )));
    schema.object = None;
    schema.string = None;
    schema.into()
}

#[derive(Serialize, JsonSchema)]
pub struct SelectionType {
    selected: PipelineActionId,
    #[schemars(schema_with = "value_schema")]
    values: IndexMap<PipelineActionId, Box<dyn ErasedPipelineAction>>,
}

impl SelectionType {
    pub fn single<A: ErasedPipelineAction + 'static>(action: A) -> Self {
        Self::one_of(vec![action])
    }

    pub fn one_of<A: ErasedPipelineAction + 'static>(values: Vec<A>) -> Self {
        let selected = values
            .first()
            .expect("selection type values should contain at least one action")
            .id();
        Self {
            selected,
            values: IndexMap::from_iter(
                values
                    .into_iter()
                    .map(|v| (v.id(), Box::new(v) as Box<dyn ErasedPipelineAction>)),
            ),
        }
    }
}
