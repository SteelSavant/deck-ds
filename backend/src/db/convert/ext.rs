use anyhow::Result;
use native_db::transaction::RwTransaction;

pub trait RwExt {
    fn remove_blind<T>(&self, item: T) -> Result<()>
    where
        T: native_db::ToInput;
}

impl RwExt for RwTransaction<'_> {
    fn remove_blind<T>(&self, item: T) -> Result<()>
    where
        T: native_db::ToInput,
    {
        Ok(self.remove(item).map(|_| ())?)
    }
}
