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

            #[allow(dead_code)]
            pub fn nil() -> Self {
                Self(uuid::Uuid::nil())
            }

            #[allow(dead_code)]
            pub fn from_uuid(uuid: uuid::Uuid) -> Self {
                Self(uuid)
            }

            #[allow(dead_code)]
            pub fn parse(string: &str) -> Self {
                Self(uuid::Uuid::parse_str(string).expect("uuid should be valid"))
            }
        }

        impl Default for $id {
            fn default() -> Self {
                Self::new()
            }
        }

        impl native_db::ToKey for $id {
            fn to_key(&self) -> native_db::Key {
                native_db::Key::new(self.0.as_bytes().to_vec())
            }

            fn key_names() -> Vec<String> {
                vec![format!("Uuid#{}", stringify!($id))]
            }
        }
    };
}

macro_rules! newtype_strid {
    ( $doc: literal, $id: ident) => {
        #[doc = $doc]
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
            #[allow(dead_code)]
            pub fn new(id: &str) -> Self {
                Self(id.to_string())
            }

            #[allow(dead_code)]
            pub fn raw(&self) -> &str {
                &self.0
            }
        }

        impl native_db::ToKey for $id {
            fn to_key(&self) -> native_db::Key {
                self.0.to_key()
            }

            fn key_names() -> Vec<String> {
                vec![format!("Strid#{}", stringify!($id))]
            }
        }
    };
}

pub(crate) use newtype_strid;
pub(crate) use newtype_uuid;
