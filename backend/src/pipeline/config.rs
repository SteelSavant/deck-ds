use indexmap::IndexMap;
use schemars::{
    gen::SchemaGenerator,
    schema::{InstanceType, RootSchema, Schema, SchemaObject},
    schema_for, JsonSchema,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::action::{ErasedPipelineAction, PipelineAction, PipelineActionId};

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
    pub actions: Vec<PipelineActionDefinition>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct PipelineActionDefinition {
    /// The id of the action
    pub id: PipelineActionDefinitionId,
    /// The name of the action
    pub name: String,
    /// The schema of the pipeline action, serialized to json
    schemas: IndexMap<PipelineActionId, RootSchema>,
    /// The value of the pipeline action
    pub selection: SelectionType,
    /// Flags whether the selection is optional. If None, not optional. If Some(true), optional and enabled, else disabled.
    pub optional: Option<bool>,
}

impl PipelineActionDefinition {
    pub fn new<A: ErasedPipelineAction>(
        name: String,
        id: PipelineActionDefinitionId,
        selection: SelectionType,
        optional: Option<bool>,
    ) -> Self {
        let schemas = selection.schemas();
        Self {
            schemas,
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

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SelectionType {
    selected: PipelineActionId,
    #[schemars(schema_with = "value_schema")]
    values: IndexMap<PipelineActionId, PipelineAction>,
}

impl SelectionType {
    pub fn single(action: PipelineAction) -> Self {
        Self::one_of(vec![action])
    }

    pub fn one_of(values: Vec<PipelineAction>) -> Self {
        let selected = values
            .first()
            .expect("selection type values should contain at least one action")
            .id();
        Self {
            selected,
            values: IndexMap::from_iter(values.into_iter().map(|v| (v.id(), v))),
        }
    }

    pub fn schemas(&self) -> IndexMap<PipelineActionId, RootSchema> {
        self.values
            .iter()
            .map(|(id, v)| (id, v.get_schema()))
            .collect()
    }
}
