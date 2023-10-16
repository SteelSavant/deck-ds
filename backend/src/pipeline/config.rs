use std::{collections::HashSet, fmt::Debug};

use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::action::PipelineAction;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PipelineDefinition {
    pub name: String,
    pub description: String,
    pub actions: Vec<Selection<SelectionType<PipelineAction>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Selection<T> {
    /// The value being selected
    pub value: T,
    /// Flags whether the selection is optional. If None, not optional. If Some(true), optional and enabled, else disabled.
    pub optional: Option<bool>,
    pub hidden_in_ui: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum SelectionType<T> where T: JsonSchema + Clone + Debug   {
    Single(T),
    OneOf(IndexMap<String, T>, String),
    AnyOf(IndexMap<String, T>, HashSet<String>),
}
