use serde_json::Value;

use crate::pipeline::config::{PipelineActionDefinition, PipelineActionDefinitionId, Selection};

use self::internal::Patch;

use super::Overrides;

pub fn patch_json(current: Value, patch: Value) -> Value {
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

#[doc(hidden)]
mod internal {
    pub struct Patch;
}

pub trait Patchable: Clone {
    fn get_definition_mut(
        &mut self,
        id: &PipelineActionDefinitionId,
    ) -> Option<&mut PipelineActionDefinition>;

    fn patched_with(&self, overrides: Overrides) -> Self {
        let mut patched = (*self).clone();
        for (id, value) in overrides.enabled.into_iter() {
            patched.patch_enabled(&id, value, Patch);
        }

        for (id, value) in overrides.fields.into_iter() {
            patched.patch_override(&id, value, Patch);
        }
        patched
    }

    #[doc(hidden)]
    fn patch_enabled(&mut self, id: &PipelineActionDefinitionId, value: bool, _: Patch) {
        let def = self.get_definition_mut(id);
        if let Some(def) = def {
            def.optional = def.optional.map(|_| value);
        }
    }

    #[doc(hidden)]
    fn patch_override(&mut self, id: &PipelineActionDefinitionId, value: Value, _: Patch) {
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
