use serde_json::Value;

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
