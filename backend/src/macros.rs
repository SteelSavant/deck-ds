macro_rules! newtype_uuid {
    ($id: ident) => {
        #[derive(
            Debug,
            Copy,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            schemars::JsonSchema,
            PartialEq,
            Eq,
            Hash,
        )]
        #[serde(transparent)]
        pub struct $id(uuid::Uuid);

        impl $id {
            pub fn new() -> Self {
                Self(uuid::Uuid::new_v4())
            }

            pub fn from_uuid(uuid: uuid::Uuid) -> Self {
                Self(uuid)
            }

            pub fn try_parse(string: &str) -> Result<Self> {
                Ok(Self(uuid::Uuid::parse_str(string)?))
            }

            pub fn parse(string: &str) -> Self {
                Self(uuid::Uuid::parse_str(string).expect("uuid should be valid"))
            }

            pub fn raw(&self) -> String {
                self.0.to_string()
            }
        }

        impl Default for $id {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

macro_rules! newtype_strid {
    ($id: ident) => {
        #[derive(
            Debug,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            schemars::JsonSchema,
            PartialEq,
            Eq,
            Hash,
        )]
        #[serde(transparent)]
        pub struct $id(String);

        impl $id {
            pub fn new(id: &str) -> Self {
                Self(id.to_string())
            }

            pub fn raw(&self) -> &str {
                &self.0
            }
        }
    };
}

pub(crate) use newtype_strid;
pub(crate) use newtype_uuid;
