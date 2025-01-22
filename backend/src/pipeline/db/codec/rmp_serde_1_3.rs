// Taken from https://github.com/vincent-herlemont/native_model/blob/main/src/codec/rmp_serde_1_3.rs
/*
MIT License

Copyright (c) 2023 Vincent Herlemont

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

/// Used to specify the
/// [rmp-serde 1.3](https://crates.io/crates/rmp-serde/1.3.0)
/// crate for serialization & deserialization, using arrays to serialize structs.
///
/// Do not use this if you plan to use serde features that skip serializing fields,
/// use [RmpSerdeNamed] instead.
///
/// # Basic usage
///
/// After enabling the `rmp_serde_1_3` feature in your `Cargo.toml`, use the
/// [`with`](crate::native_model) attribute on your type to instruct
/// `native_model` to use `RmpSerde` for serialization & deserialization.
///
/// Example usage:
///
/// ```rust
/// # use native_model::*;
/// #[derive(Clone, Default, serde::Deserialize, serde::Serialize)]
/// #[native_model(id = 1, version = 1, with = native_model::rmp_serde_1_3::RmpSerde)]
/// struct MyStruct {
///     my_string: String
/// }
/// ```

pub struct RmpSerde;

impl<T: serde::Serialize> native_model::Encode<T> for RmpSerde {
    type Error = rmp_serde::encode::Error;
    /// Serializes a type into bytes using the `rmp-serde` `1.3` crate.
    fn encode(obj: &T) -> Result<Vec<u8>, Self::Error> {
        rmp_serde::encode::to_vec(obj)
    }
}

impl<T: for<'de> serde::Deserialize<'de>> native_model::Decode<T> for RmpSerde {
    type Error = rmp_serde::decode::Error;
    /// Deserializes a type from bytes using the `rmp-serde` `1.3` crate.
    fn decode(data: Vec<u8>) -> Result<T, Self::Error> {
        rmp_serde::decode::from_slice(&data)
    }
}

/// Used to specify the
/// [rmp-serde 1.3](https://crates.io/crates/rmp-serde/1.3.0)
/// crate for serialization & deserialization, using maps to serialize structs.
///
/// # Basic usage
///
/// After enabling the `rmp_serde_1_3` feature in your `Cargo.toml`, use the
/// [`with`](crate::native_model) attribute on your type to instruct
/// `native_model` to use `RmpSerdeNamed` for serialization & deserialization.
///
/// Example usage:
///
/// ```rust
/// # use native_model::*;
/// #[derive(Clone, Default, serde::Deserialize, serde::Serialize)]
/// #[native_model(id = 1, version = 1, with = native_model::rmp_serde_1_3::RmpSerdeNamed)]
/// struct MyStruct {
///     #[serde(skip_serializing_if = "String::is_empty")]
///     my_string: String
/// }
/// ```

pub struct RmpSerdeNamed;

impl<T: serde::Serialize> native_model::Encode<T> for RmpSerdeNamed {
    type Error = rmp_serde::encode::Error;
    /// Serializes a type into bytes using the `rmp-serde` `1.3` crate.
    fn encode(obj: &T) -> Result<Vec<u8>, Self::Error> {
        rmp_serde::encode::to_vec_named(obj)
    }
}

impl<T: for<'de> serde::Deserialize<'de>> native_model::Decode<T> for RmpSerdeNamed {
    type Error = rmp_serde::decode::Error;
    /// Deserializes a type from bytes using the `rmp-serde` `1.3` crate.
    fn decode(data: Vec<u8>) -> Result<T, Self::Error> {
        rmp_serde::decode::from_slice(&data)
    }
}
