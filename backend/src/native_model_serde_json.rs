// Taken from https://github.com/vincent-herlemont/native_model/blob/main/tests_crate/tests/example/custom_codec/bincode_serde.rs

use serde::{Deserialize, Serialize};

pub struct NativeModelJSON;

impl<T: Serialize> native_model::Encode<T> for NativeModelJSON {
    type Error = serde_json::Error;
    fn encode(obj: &T) -> Result<Vec<u8>, Self::Error> {
        Ok(serde_json::to_string(obj)?.as_bytes().to_vec())
    }
}

impl<T: for<'a> Deserialize<'a>> native_model::Decode<T> for NativeModelJSON {
    type Error = serde_json::Error;
    fn decode(data: Vec<u8>) -> Result<T, Self::Error> {
        let res = serde_json::from_slice::<T>(&data);

        res
    }
}

use native_model::native_model;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[native_model(id = 1, version = 1, with = NativeModelJSON)]
struct DotV1(u32, u32);

#[test]
fn test_serde_json_serialize_deserialize() {
    // Application 1
    let dot = DotV1(1, 2);
    let bytes = native_model::encode(&dot).unwrap();
    // Application 1
    let (dot, _) = native_model::decode::<DotV1>(bytes).unwrap();
    assert_eq!(dot, DotV1(1, 2));
}
