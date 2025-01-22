use std::path::Path;

use migrate::migrate;
use model::MODELS;
use native_db::Database;

use crate::{macros::newtype_strid, util::create_dir_all};

mod migrate;
mod model;

/// Database for per-monitor display (and possibly other) settings
pub struct SettingsDb<'a> {
    db: Database<'a>,
}

impl<'a> SettingsDb<'a> {
    pub fn new<P>(db_path: P) -> Self
    where
        P: AsRef<Path>,
    {
        let db_path = db_path.as_ref();
        let parent = db_path
            .parent()
            .expect("db_path should have parent directory");

        if !parent.exists() {
            create_dir_all(parent).expect("should be able to create db dir");
        }

        let mut db = if db_path.is_file() {
            native_db::Builder::new()
                .open(&MODELS, db_path)
                .expect("should be able to open settings db")
        } else {
            native_db::Builder::new()
                .create(&MODELS, db_path)
                .expect("should be able to create settings db")
        };

        let rw = db
            .rw_transaction()
            .expect("initial migration transaction should be valid");
        migrate(&rw).expect("db migrations should succeed");
        rw.commit().expect("migrations should commit");

        db.compact().expect("db compact should succeed");

        Self { db }
    }
}

newtype_strid!(
    "Id to uniquely identify a monitor using the model + serial",
    MonitorId
);

impl MonitorId {
    // TODO::from_edid
}

/*

Fixed Hardware Settings
- Deck Screen Location

Software Settings Related to Hardware
- External Display Resolution
- Touch Config
- Deck is Primary Display (w/KDE taskbar)

- KWin Swap Screens



Make separate hardware DB that, for each deviceId stores:
- Deck Screen Location
- External Display Resolution
- Touch Config
- Deck is Primary Display (w/KDE taskbar)
- KWin Swap Screens

Everything but Deck Screen Location is overridable per-config
Make Kwin actions that depend on knowing swap-screens depend on display config action


Store monitor list, sorted by last selected

*/
