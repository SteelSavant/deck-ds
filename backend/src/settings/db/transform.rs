/// Data transforms between working data and the database format
//
use native_db::{
    transaction::{RTransaction, RwTransaction},
    Database,
};

use super::model::DbCategoryProfile;
use crate::settings::CategoryProfile;
use anyhow::Result;

impl CategoryProfile {
    pub fn save_all(&self, rw: &RwTransaction) -> Result<()> {
        todo!()
    }
}

impl DbCategoryProfile {
    pub fn reconstruct(self, ro: &RTransaction) -> Result<CategoryProfile> {
        todo!()
    }
}
