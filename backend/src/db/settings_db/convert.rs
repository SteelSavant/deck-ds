use super::{
    model::{DbMonitorDisplaySettings, DbSystemDisplay},
    MonitorDisplaySetting, SystemDisplay,
};

impl From<DbMonitorDisplaySettings> for MonitorDisplaySetting {
    fn from(value: DbMonitorDisplaySettings) -> Self {
        Self {
            id: value.id,
            external_display_settings: value.external_display_settings.into(),
            system_display: match value.system_display {
                DbSystemDisplay::External => SystemDisplay::External,
                DbSystemDisplay::Embedded => SystemDisplay::Embedded,
            },
            deck_location: value.deck_location.into(),
            deck_is_enabled: value.deck_is_enabled.into(),
            last_updated_at: value.last_updated_at.into(),
        }
    }
}

impl From<MonitorDisplaySetting> for DbMonitorDisplaySettings {
    fn from(value: MonitorDisplaySetting) -> Self {
        Self {
            id: value.id,
            external_display_settings: value.external_display_settings.into(),
            system_display: match value.system_display {
                SystemDisplay::External => DbSystemDisplay::External,
                SystemDisplay::Embedded => DbSystemDisplay::Embedded,
            },
            deck_location: value.deck_location.into(),
            deck_is_enabled: value.deck_is_enabled.into(),
            last_updated_at: value.last_updated_at.into(),
        }
    }
}
