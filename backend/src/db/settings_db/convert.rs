use super::{
    model::{DbMonitorDisplaySettings, DbPrimaryDisplayLocation},
    MonitorDisplaySettings, SystemDisplay,
};

impl From<DbMonitorDisplaySettings> for MonitorDisplaySettings {
    fn from(value: DbMonitorDisplaySettings) -> Self {
        Self {
            id: value.id,
            external_display_settings: value.external_display_settings.into(),
            system_display: match value.system_display {
                DbPrimaryDisplayLocation::External => SystemDisplay::External,
                DbPrimaryDisplayLocation::Embedded => SystemDisplay::Embedded,
            },
            deck_location: value.deck_location.into(),
            deck_is_enabled: value.deck_is_enabled.into(),
            last_updated_at: value.last_updated_at.into(),
        }
    }
}

impl From<MonitorDisplaySettings> for DbMonitorDisplaySettings {
    fn from(value: MonitorDisplaySettings) -> Self {
        Self {
            id: value.id,
            external_display_settings: value.external_display_settings.into(),
            system_display: match value.system_display {
                SystemDisplay::External => DbPrimaryDisplayLocation::External,
                SystemDisplay::Embedded => DbPrimaryDisplayLocation::Embedded,
            },
            deck_location: value.deck_location.into(),
            deck_is_enabled: value.deck_is_enabled.into(),
            last_updated_at: value.last_updated_at.into(),
        }
    }
}
