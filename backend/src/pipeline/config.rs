use std::collections::HashMap;

use anyhow::Result;

use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::action::PipelineAction;

macro_rules! newtype_id {
    ($id: ident) => {
        #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PipelineDefinition {
    pub name: String,
    pub tags: Vec<String>,
    pub id: PipelineDefinitionId,
    pub description: String,
    pub action: PipelineActionDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]

pub enum Selection {
    Action(PipelineAction),
    OneOf {
        selection: PipelineActionDefinitionId,
        actions: IndexMap<PipelineActionDefinitionId, PipelineActionDefinition>,
    },
    AllOf(Vec<PipelineActionDefinition>),
}

/// Overrides for a pipeline definition.
///
/// Json is in the format
///
/// ```json
/// {
///     "guid_for_action_selection": {
///         "overridden_field1": "value1",
///         "overridden_field2": 2,
///         "overridden_field3": {
///             "nested_field": 4.5
///         }
///     },
///     "guid_for_oneof": {
///         "selection": "some_guid",
///     },
/// }
/// ```
///
/// All guids are flattened top-level, so [Selection::AllOf] and [Selection::OneOf].actions will not exist.
#[derive(Debug, Serialize, Deserialize)]
pub struct Overrides {
    pub id: String,
    pub template: PipelineDefinitionId,
    pub overrides: HashMap<PipelineActionDefinitionId, Value>,
    pub enabled: HashMap<PipelineActionDefinitionId, bool>,
}

impl PipelineDefinition {
    pub fn new(
        id: PipelineDefinitionId,
        name: String,
        description: String,
        tags: Vec<String>,
        selection: Selection,
    ) -> Self {
        let action = PipelineActionDefinition {
            id: PipelineActionDefinitionId::new(id.0.clone()),
            name: "root".to_string(),
            description: None,
            selection,
            optional: None,
        };
        Self {
            action,
            name,
            tags,
            id,
            description,
        }
    }

    pub fn patched_with(&self, overrides: Overrides) {
        let mut patched = (*self).clone();
        for (id, value) in overrides.enabled.into_iter() {
            patched.patch_enabled(&id, value);
        }

        for (id, value) in overrides.overrides.into_iter() {
            patched.patch_override(&id, value)
        }
    }

    fn patch_enabled(&mut self, id: &PipelineActionDefinitionId, value: bool) {
        let def = self.get_definition_mut(id);
        if let Some(def) = def {
            def.optional = def.optional.map(|_| value);
        }
    }

    fn patch_override(&mut self, id: &PipelineActionDefinitionId, value: Value) {
        let def = self.get_definition_mut(id);
        if let Some(def) = def {
            def.selection = match &def.selection {
                Selection::Action(action) => {
                    let current_json = serde_json::to_value(action).unwrap();
                    Selection::Action(
                        serde_json::from_value(patch_json(current_json, value)).unwrap(),
                    )
                }
                Selection::OneOf { actions, .. } => {
                    let new_selection = value["selection"].as_str().unwrap();
                    Selection::OneOf {
                        selection: PipelineActionDefinitionId::parse(new_selection),
                        actions: actions.clone(), // TODO::avoid this clone
                    }
                }
                Selection::AllOf(_) => panic!("AllOf definitions are not patchable!"),
            }
        }
    }

    fn get_definition_mut(
        &mut self,
        id: &PipelineActionDefinitionId,
    ) -> Option<&mut PipelineActionDefinition> {
        fn get_definition_rec<'a>(
            def: &'a mut PipelineActionDefinition,
            id: &PipelineActionDefinitionId,
        ) -> Option<&'a mut PipelineActionDefinition> {
            if def.id == *id {
                return Some(def);
            }

            match def.selection {
                Selection::Action(_) => None,
                Selection::OneOf {
                    ref mut actions, ..
                } => actions
                    .iter_mut()
                    .fold(None, |acc, a| if a.0 == id { Some(a.1) } else { acc }),
                Selection::AllOf(ref mut definitions) => definitions
                    .iter_mut()
                    .fold(None, |acc, d| if d.id == *id { Some(d) } else { acc }),
            }
        }

        get_definition_rec(&mut self.action, id)
    }
}

fn patch_json(current: Value, patch: Value) -> Value {
    match current {
        Value::Object(mut obj) => match patch {
            Value::Object(patch) => {
                for (k, v) in patch.into_iter() {
                    match &v {
                        Value::Object(_) => obj[&k] = patch_json(obj[&k].clone(), v),
                        _ => obj[&k] = v,
                    }
                }

                serde_json::to_value(obj).unwrap()
            }
            _ => patch,
        },
        _ => patch,
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn patch_null() {
        let expected = Value::Null;
        let actual = patch_json(json!(false), Value::Null);

        assert_eq!(expected, actual);
    }

    #[test]
    fn patch_onto_null() {
        let expected = json!(5);
        let actual = patch_json(Value::Null, expected.clone());

        assert_eq!(expected, actual);
    }

    #[test]
    fn patch_bool() {
        let expected = json!(true);
        let actual = patch_json(json!(false), expected.clone());
        assert_eq!(expected, actual);
    }

    #[test]
    fn patch_number() {
        let expected = json!(5);
        let actual = patch_json(json!(10), expected.clone());
        assert_eq!(expected, actual);
    }

    #[test]
    fn patch_str() {
        let expected = json!("expected");
        let actual = patch_json(json!("actual"), expected.clone());
        assert_eq!(expected, actual);
    }

    #[test]
    fn patch_arr() {
        let expected = json!(vec!["hello", "goodbye"]);
        let actual = patch_json(json!(vec!["actual"]), expected.clone());
        assert_eq!(expected, actual);
    }

    #[test]
    fn patch_obj() {
        let expected = json!({
            "ignored": "ignored",
            "null": None::<()>,
            "bool": true,
            "num": 1,
            "str": "Hello1",
            "arr": ["hello1", "world1"],
            "obj": {
                "ignored": "ignored",
                "null": None::<()>,
                "bool": true,
                "num": 2,
                "str": "Hello2",
                "arr": ["hello2", "world2"],
                "obj": {
                    "ignored": "ignored",
                    "null": None::<()>,
                    "bool": true,
                    "num": 3,
                    "str": "Hello3",
                    "arr": ["hello3", "world3"],
                }
            }
        });

        let actual = patch_json(
            json!({
                "ignored": "ignored",
                "null": 5,
                "bool": false,
                "num": 42,
                "str": "str",
                "arr": ["arr"],
                "obj": {
                    "ignored": "ignored",
                    "null": { "hello": "world"},
                    "bool": false,
                    "num": 42,
                    "str": "garble",
                    "arr": ["hello", "world"],
                    "obj": {
                        "ignored": "ignored",
                        "null": ["hello"],
                        "bool": false,
                        "num": 42,
                        "str": "Hello",
                        "arr": ["hello", "world"],
                    }
                }
            }),
            json!({
                "null": None::<()>,
                "bool": true,
                "num": 1,
                "str": "Hello1",
                "arr": ["hello1", "world1"],
                "obj": {
                    "null": None::<()>,
                    "bool": true,
                    "num": 2,
                    "str": "Hello2",
                    "arr": ["hello2", "world2"],
                    "obj": {
                        "null": None::<()>,
                        "bool": true,
                        "num": 3,
                        "str": "Hello3",
                        "arr": ["hello3", "world3"],
                    }
                }
            }),
        );

        assert_eq!(expected, actual);
    }
}
