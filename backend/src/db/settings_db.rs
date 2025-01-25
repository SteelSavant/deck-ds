use anyhow::Result;
use edid::EDID;
use itertools::Itertools;
use schemars::JsonSchema;
use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};

use migrate::migrate;
use model::{DbEmbeddedDisplaySettings, DbMonitorDisplaySettings, MODELS};
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
    pub const UNKNOWN_STR: &'static str = "Unknown";

    pub fn from_display_info(info: &DisplayInfo) -> Self {
        let largest_mode = info.display_modes.first();
        let width = largest_mode.map(|v| v.width).unwrap_or_default();
        let height = largest_mode.map(|v| v.height).unwrap_or_default();

        Self::new(&Self::format_id(&info.model, &info.serial, width, height))
    }

    pub fn from_edid(edid: &EDID, max_width: u32, max_height: u32) -> Self {
        let mut serial = None;
        let mut model = None;

        for d in &edid.descriptors {
            match d {
                edid::Descriptor::SerialNumber(s) => serial = Some(s),
                edid::Descriptor::ProductName(p) => model = Some(p),
                _ => (),
            }
        }

        Self::new(&Self::format_id(
            &model.unwrap_or(&Self::UNKNOWN_STR.to_string()),
            &serial.unwrap_or(&Self::UNKNOWN_STR.to_string()),
            max_width,
            max_height,
        ))
    }

    fn format_id(model: &str, serial: &str, max_width: u32, max_height: u32) -> String {
        format!("{}-{}_{}x{}", model, serial, max_width, max_height)
    }
}

impl Default for MonitorId {
    fn default() -> Self {
        Self::new(&Self::format_id(
            Self::UNKNOWN_STR,
            Self::UNKNOWN_STR,
            Default::default(),
            Default::default(),
        ))
    }
}

#[derive(Debug, Default)]
pub struct MonitorDisplaySettings {
    pub embedded_display_id: Option<MonitorId>,
    pub monitor_display_settings: Vec<MonitorDisplaySetting>,
}

/// Display settings for an individual monitor.
///
/// The following fields can be overridden by profiles:
/// - deck_is_enabled,
/// - external_display_settings
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct MonitorDisplaySetting {
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
    // TODO::add settings for calibration, like associated touch device, and whether or not the display is calibrated
}

impl Default for MonitorDisplaySetting {
    fn default() -> Self {
        Self {
            id: Default::default(),
            external_display_settings: Default::default(),
            deck_location: Default::default(),
            system_display: SystemDisplay::Embedded,
            deck_is_enabled: true,
            last_updated_at: SystemTime::UNIX_EPOCH,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum SystemDisplay {
    External,
    Embedded,
}

/// Database for per-monitor display (and possibly other) settings
#[derive(derive_more::Debug)]
pub struct SettingsRepository {
    #[debug(skip)]
    db: Database<'static>,
}

impl SettingsRepository {
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

impl SettingsRepository {
    /// Gets the stored display settings.
    /// Will guess an embedded display from the `display_info` if none is present.
    /// All items in `display_info` that do not match the embedded display, and
    /// are not in the stored settings, will be added to the stored settings
    pub fn get_monitor_display_settings(
        &self,
        display_info: &[DisplayInfo],
    ) -> Result<MonitorDisplaySettings> {
        let rw = self.db.rw_transaction()?;

        // Find best embedded display candidate

        let embedded_display_settings = rw
            .get()
            .primary::<DbEmbeddedDisplaySettings>(DbEmbeddedDisplaySettings::KEY)?;

        let maybe_embedded_display = display_info
            .iter()
            .find(|v| v.sys_path.to_string_lossy().contains("eDP"))
            .or_else(|| display_info.last());

        let embedded_id = embedded_display_settings
            .map(|v| v.embedded_display_id)
            .or_else(|| maybe_embedded_display.map(|v| MonitorId::from_display_info(v)));

        // insert best candidate as
        if embedded_id.is_none() {
            if let Some(info) = maybe_embedded_display {
                rw.upsert(DbEmbeddedDisplaySettings::new(info.get_id()));
            }
        }

        // Load existing settings

        let mut settings: Vec<MonitorDisplaySetting> = rw
            .scan()
            .primary::<DbMonitorDisplaySettings>()?
            .all()?
            .map(|v| v.map(|v| v.into()))
            .map(|v| Ok(v?))
            .collect::<Result<Vec<_>>>()?;

        // Add missing settings

        let missing = display_info
            .iter()
            .map(|v| MonitorId::from_display_info(&v))
            .filter(|v| !settings.iter().any(|s| s.id == *v))
            .collect_vec();

        for m in missing {
            settings.push(MonitorDisplaySetting {
                id: m,
                ..Default::default()
            });
        }

        // Sort and filter settings by extant devices

        settings = settings
            .into_iter()
            .filter(|v| {
                Some(&v.id) != embedded_id.as_ref()
                    && display_info.iter().any(|d| d.get_id() == v.id)
            })
            .collect();

        settings.sort_by(|a, b| b.last_updated_at.cmp(&a.last_updated_at));

        Ok(MonitorDisplaySettings {
            embedded_display_id: embedded_id,
            monitor_display_settings: settings,
        })
    }

    pub fn set_monitor_display_setting(
        &mut self,
        mut settings: MonitorDisplaySetting,
    ) -> Result<()> {
        settings.last_updated_at = SystemTime::now();

        let rw = self.db.rw_transaction()?;
        rw.upsert::<DbMonitorDisplaySettings>(settings.into())?;
        rw.commit()?;
        Ok(())
    }

    pub fn set_embedded_monitor_setting(&mut self, id: MonitorId) -> Result<()> {
        let rw = self.db.rw_transaction()?;
        let item = rw.get().primary::<DbMonitorDisplaySettings>(id.clone())?;
        if let Some(item) = item {
            let _ = rw.remove(item);
        }

        rw.upsert(DbEmbeddedDisplaySettings::new(id));

        rw.commit()?;
        Ok(())
    }
}

/*
TODO::

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
