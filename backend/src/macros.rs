macro_rules! newtype_uuid {
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
        pub struct $id(uuid::Uuid);

        impl $id {
            pub fn new(uuid: uuid::Uuid) -> Self {
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
            pub fn new(id: String) -> Self {
                Self(id)
            }

            pub fn raw(&self) -> String {
                self.0.to_string()
            }
        }
    };
}

pub(crate) use newtype_strid;
pub(crate) use newtype_uuid;
