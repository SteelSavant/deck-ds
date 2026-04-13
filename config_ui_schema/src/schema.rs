use std::{collections::HashMap, path::PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
struct SchemaTag(String);

/// Describes a config object
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
struct ConfigUISchema {
    /// (Display) name of the config
    name: String,
    /// Description of the config
    description: String,
    /// Map of config fields to settings
    settings: HashMap<String, ConfigUISetting>,
    // TODO::if we ever support plugins, have a "source" flag for where/how to send updates,
    // since plugins will likely take  { path, value } rather than the full UI;
    // we only take the full UI in Rust land because we have clean conversion macros
    // to real types, which can't exist in a plugin
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
struct ConfigUISetting {
    /// Display name in UI
    name: String,
    /// Description in UI
    description: String,
    /// The type of UI to build
    ui: ConfigUIType,
    /// Extra info on how to build the UI at runtime
    ui_meta: Option<String>,
    /// Whether or not the user can edit the value at runtime
    is_configurable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "ui_type")]
enum ConfigUIType {
    Toggle(bool),
    Checkbox(bool),
    IntInput(NumSetting<i128>),
    FloatInput(NumSetting<f32>),
    StringInput(String), // Maybe include regex/validator?
    IntSlider(NumSetting<i128>),
    FloatSlider(NumSetting<f32>),
    Path(PathSetting),
    List(List),
    Section(Section),
    DropDown(DropDown),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
struct List {
    values: Vec<Value>,
    /// List of config UI used to add values to the list.
    /// Toggle and Checkbox not supported.
    value_builder: Vec<ConfigUIType>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
struct Section {
    tag: SchemaTag,
    settings: HashMap<String, ConfigUISetting>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
struct NumSetting<V> {
    value: V,
    min: V,
    max: V,
    step: V,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
struct PathSetting {
    path: PathBuf,
    valid_extensions: Option<Vec<String>>,
}

/// Represents a UI dropdown. Currently, all values must be unique and comparable,
/// since the raw value is stored directly. This is in case options are resolved
/// dynamically, and shift indexes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
struct DropDown {
    selected: Option<DropdownItem<Value>>,
    options: Vec<DropdownItem<Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
enum Value {
    Bool(bool),
    Int(i128),
    Float(f32),
    String(String),
    Struct(Section),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
struct DropdownItem<V> {
    label: String,
    description: Option<String>,
    tag: SchemaTag,
    value: V,
    is_editable: bool,
}
