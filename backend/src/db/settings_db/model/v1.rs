use std::time::SystemTime;

use native_db::*;
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, TimestampMicroSeconds};

use crate::{
    db::common::{
        codec::rmp_serde_1_3::RmpSerde,
        model::{DbExternalDisplaySettings, DbRelativeLocation},
    },
    settings_db::MonitorId,
};

#[native_db]
#[native_model(id = 1, version = 1, with = RmpSerde)]
#[derive(Serialize, Deserialize)]

pub struct DbEmbeddedDisplaySettings {
    #[primary_key]
    key: _DbEmbeddedDisplaySettingsKey,
    pub embedded_display_id: MonitorId,
    // TODO::add settings for calibration, like associated touch device, and whether or not the display is calibrated
}

impl DbEmbeddedDisplaySettings {
    pub const KEY: _DbEmbeddedDisplaySettingsKey = _DbEmbeddedDisplaySettingsKey;

    pub fn new(id: MonitorId) -> Self {
        Self {
            key: _DbEmbeddedDisplaySettingsKey,
            embedded_display_id: id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct _DbEmbeddedDisplaySettingsKey;

impl native_db::ToKey for _DbEmbeddedDisplaySettingsKey {
    fn to_key(&self) -> native_db::Key {
        Key::new(vec![0])
    }

    fn key_names() -> Vec<String> {
        vec!["DbEmbeddedDisplaySettings0".to_string()]
    }
}

#[serde_as]
#[native_db]
#[native_model(id = 2, version = 1, with = RmpSerde)]
#[derive(Serialize, Deserialize)]
pub struct DbMonitorDisplaySettings {
    #[primary_key]
    pub id: MonitorId,
    pub external_display_settings: DbExternalDisplaySettings,
    pub deck_location: DbRelativeLocation,
    pub system_display: DbSystemDisplay,
    pub deck_is_enabled: bool,
    #[serde_as(as = "TimestampMicroSeconds<i64>")]
    pub last_updated_at: SystemTime,
}

#[derive(Serialize, Deserialize)]
pub enum DbSystemDisplay {
    External,
    Embedded,
}
