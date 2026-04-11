use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
struct SchemaTag(String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
struct ConfigSchema {
    name: String,
    description: String,
    settings: Vec<ConfigSetting>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
struct ConfigSetting {
    field: String,
    name: String,
    description: String,
    ui: ConfigSettingUI,
    is_configurable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "value")]
enum ConfigSettingUI {
    Toggle(bool),
    Checkbox(bool),
    IntInput(NumSetting<i128>),
    FloatInput(NumSetting<f32>),
    StringInput(String), // Maybe include regex/validator?
    IntSlider(NumSetting<i128>),
    FloatSlider(NumSetting<f32>),
    Path(PathSetting),
    Section(Section),
    DropDown(Box<dyn DropDown>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
struct Section {
    tag: SchemaTag,
    settings: Vec<ConfigSetting>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
struct NumSetting<V> {
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
struct DropDownImpl<V> {
    selected: Option<DropdownItem<V>>,
    options: Vec<DropdownItem<V>>,
}

trait DropDownBounds = std::fmt::Debug + Clone;
trait DropDown: DropDownBounds {}

impl<V> DropDown for DropDownImpl<V> where V: DropDownBounds {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
struct DropdownItem<V> {
    label: String,
    tag: SchemaTag,
    value: V,
    is_editable: bool,
}

fn melonds_layout_config() -> ConfigSchema {
    ConfigSchema {
        name: "melonDS Layout".into(),
        description: "".into(),
        settings: vec![],
    }
}
