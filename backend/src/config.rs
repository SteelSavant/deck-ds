use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineDefinition {
    name: String,
    description: String,
    pipeline: Vec<Selection<SelectionType<PipelineAction>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selection<T> {
    /// The value being selected
    value: T,
    /// Flags whether the selection is optional. If None, not optional. If Some(true), optional and enabled, else disabled.
    optional: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionType<T> {
    Single(T),
    OneOf(Vec<(String, T)>),
    AnyOf(Vec<(String, T)>),
}

pub type PipelineArgs = Vec<Selection<SelectionType<ArgumentType>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineAction {
    name: String,
    args: PipelineArgs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArgumentType {
    Bool(bool),
    Int(usize),
    String(String),
    Object(Vec<Selection<SelectionType<ArgumentType>>>),
    PipelineSelection(SelectionType<PipelineAction>),
}
