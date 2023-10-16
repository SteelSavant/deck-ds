use std::collections::HashSet;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::action::PipelineAction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineDefinition {
    pub name: String,
    pub description: String,
    pub actions: Vec<Selection<SelectionType<PipelineAction>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selection<T> {
    /// The value being selected
    pub value: T,
    /// Flags whether the selection is optional. If None, not optional. If Some(true), optional and enabled, else disabled.
    pub optional: Option<bool>,
    pub hidden_in_ui: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionType<T> {
    Single(T),
    OneOf(IndexMap<String, T>, String),
    AnyOf(IndexMap<String, T>, HashSet<String>),
}
