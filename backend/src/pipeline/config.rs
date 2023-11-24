use derive_more::Display;
use std::collections::HashMap;

use crate::macros::{newtype_strid, newtype_uuid};
use anyhow::Result;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::action::PipelineAction;

newtype_strid!(
    r#"Id in the form "plugin:group:action" | "plugin:group:action:variant""#,
    PipelineActionDefinitionId
);
newtype_uuid!(TemplateDefinitionId);

#[derive(Copy, Debug, Display, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema)]
pub enum PipelineTarget {
    Desktop,
    Gamemode,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TemplateDefinition {
    pub name: String,
    pub tags: Vec<String>,
    pub id: TemplateDefinitionId,
    pub description: String,
    pub targets: HashMap<PipelineTarget, Selection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PipelineActionDefinition {
    /// The name of the action
    pub name: String,
    /// An optional description of what the action does.
    pub description: Option<String>,
    /// The value of the pipeline action
    pub selection: Selection,

    /// Flags whether the selection is exported for use in other actions.
    pub exported: bool,
}

impl PipelineActionDefinition {
    pub fn new(
        name: String,
        description: Option<String>,
        exported: bool,
        selection: Selection,
    ) -> Self {
        Self {
            name,
            description,
            selection,
            exported,
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]

pub enum Selection {
    Action(PipelineAction),
    OneOf {
        selection: PipelineActionDefinitionId,
        actions: Vec<PipelineActionDefinitionId>,
    },
    AllOf(Vec<Enabled<PipelineActionDefinitionId>>),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Enabled<T> {
    /// Flags whether the selection is enabled. If None, not optional. If Some(true), optional and enabled, else disabled.
    pub enabled: Option<bool>,
    pub selection: T,
}

impl<T> Enabled<T> {
    pub fn force(selection: T) -> Self {
        Self {
            enabled: None,
            selection,
        }
    }

    pub fn default_true(selection: T) -> Self {
        Self {
            enabled: Some(true),
            selection,
        }
    }

    pub fn default_false(selection: T) -> Self {
        Self {
            enabled: Some(false),
            selection,
        }
    }
}

impl TemplateDefinition {
    pub fn new(
        id: TemplateDefinitionId,
        name: String,
        description: String,
        tags: Vec<String>,
        targets: HashMap<PipelineTarget, Selection>,
    ) -> Self {
        Self {
            targets,
            name,
            tags,
            id,
            description,
        }
    }
}
