use anyhow::Result;
use schemars::JsonSchema;
use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};

use migrate::migrate;
use model::{DbMonitorDisplaySettings, MODELS};
use native_db::Database;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, TimestampMicroSeconds};

use crate::{
    macros::newtype_strid,
    pipeline::action::display_config::{ExternalDisplaySettings, RelativeLocation},
    sys::display_info::DisplayInfo,
    util::create_dir_all,
};

mod convert;
mod migrate;
mod model;

newtype_strid!(
    "Id to uniquely identify a monitor using the model + serial",
    MonitorId
);

impl MonitorId {
    pub fn from_display_info(info: &DisplayInfo) -> Self {
        Self::new(&format!("{}-{}", info.model, info.serial))
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, JsonSchema)]
/// Display settings for an individual monitor.
///
/// The following fields can be overridden by profiles:
/// - deck_is_enabled,
/// - external_display_settings
pub struct MonitorDisplaySettings {
    pub id: MonitorId,
    /// Display settings for the external monitor.
    pub external_display_settings: ExternalDisplaySettings,
    /// Location of the deck in physical space in relation to the external monitor.
    pub deck_location: RelativeLocation,
    /// The display to make the primary display in KDE (the one with the taskbar).
    /// If the application launched is single-window, it will appear on the OTHER display,
    /// to allow the user to access the taskbar, etc. on the `system_display``
    pub system_display: SystemDisplay,
    /// If `true`, the deck screen is enabled. Otherwise, the deck screen is disabled.
    pub deck_is_enabled: bool,
    /// Tracks the last time this monitor was set as the one to use.
    /// Used to sort monitor settings to find the most recent used
    /// of the available monitors.
    #[serde_as(as = "TimestampMicroSeconds<i64>")]
    pub last_updated_at: SystemTime,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub enum SystemDisplay {
    External,
    Embedded,
}

/// Database for per-monitor display (and possibly other) settings
#[derive(derive_more::Debug)]
pub struct SettingsDb {
    #[debug(skip)]
    db: Database<'static>,
}

impl SettingsDb {
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

        let mut db = if db_path.to_string_lossy() == "memory" {
            native_db::Builder::new()
                .create_in_memory(&MODELS)
                .expect("should be able to create in-memory db")
        } else if db_path.is_file() {
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

impl SettingsDb {
    pub fn get_monitor_display_settings(&self) -> Result<Vec<MonitorDisplaySettings>> {
        let r = self.db.r_transaction()?;

        let mut settings: Vec<MonitorDisplaySettings> = r
            .scan()
            .primary::<DbMonitorDisplaySettings>()?
            .all()?
            .map(|v| v.map(|v| v.into()))
            .map(|v| Ok(v?))
            .collect::<Result<Vec<_>>>()?;

        settings.sort_by(|a, b| b.last_updated_at.cmp(&a.last_updated_at));

        Ok(settings)
    }

    pub fn set_monitor_display_setting(
        &mut self,
        mut settings: MonitorDisplaySettings,
    ) -> Result<()> {
        settings.last_updated_at = SystemTime::now();

        let rw = self.db.rw_transaction()?;
        rw.upsert::<DbMonitorDisplaySettings>(settings.into())?;
        rw.commit()?;
        Ok(())
    }
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
