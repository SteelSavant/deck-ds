/// Contains migrations between model versions
use anyhow::Result;
use native_db::Database;

pub trait Migrate {
    fn migrate(&self) -> Result<()>;
}

impl<'a> Migrate for Database<'a> {
    fn migrate(&self) -> Result<()> {
        // no migrations right now
        Ok(())
    }
}
