use anyhow::Result;

use indexmap::IndexMap;
use schemars::{
    gen::SchemaGenerator,
    schema::{InstanceType, Schema, SchemaObject},
    JsonSchema,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::action::PipelineAction;

macro_rules! newtype_id {
    ($id: ident) => {
        #[derive(Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
        #[serde(transparent)]
        pub struct $id(Uuid);

        impl $id {
            pub fn new(uuid: Uuid) -> Self {
                Self(uuid)
            }

            pub fn try_parse(string: &str) -> Result<Self> {
                Ok(Self(Uuid::parse_str(string)?))
            }

            pub fn parse(string: &str) -> Self {
                Self(Uuid::parse_str(string).expect("uuid should be valid"))
            }
        }
    };
}

newtype_id!(PipelineActionId);
newtype_id!(PipelineActionDefinitionId);
newtype_id!(PipelineDefinitionId);

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GenericPipelineDefinition<T> {
    pub name: String,
    pub tags: T,
    pub id: PipelineDefinitionId,
    pub description: String,
    pub selection: Selection,
}

/// Pipeline definition that is applied to all apps with any tag matching one of the stored tags.
pub type PipelineDefinition = GenericPipelineDefinition<Vec<String>>;

/// Templates are not meant to be modified or used directly, so they don't have usable tags.
pub type PipelineDefinitionTemplate = GenericPipelineDefinition<()>;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct PipelineActionDefinition {
    /// The id of the action
    pub id: PipelineActionDefinitionId,
    /// The name of the action
    pub name: String,
    /// An optional description of what the action does.
    pub description: Option<String>,
    /// The value of the pipeline action
    pub selection: Selection,
    /// Flags whether the selection is optional. If None, not optional. If Some(true), optional and enabled, else disabled.
    pub optional: Option<bool>,
}

impl PipelineActionDefinition {
    pub fn new(
        name: String,
        description: Option<String>,
        id: PipelineActionDefinitionId,
        optional: Option<bool>,
        selection: Selection,
    ) -> Self {
        Self {
            name,
            description,
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
