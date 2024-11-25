/// Contains migrations between model versions
use anyhow::Result;
use native_db::transaction::RwTransaction;

/// Runs migrations using the provided `RwTransaction`.
/// Committing the migration is handled externally.
pub fn migrate(_rw: &RwTransaction) -> Result<()> {
    // no migrations right now
    Ok(())
}
