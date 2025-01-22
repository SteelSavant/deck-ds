#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum DbExternalDisplaySettings {
    /// Previous resolution, before setup
    #[default]
    Previous,
    /// Native resolution
    Native,
    /// Resolution based on specific settings
    Preference(DbModePreference),
}

impl From<ExternalDisplaySettings> for DbExternalDisplaySettings {
    fn from(value: ExternalDisplaySettings) -> Self {
        match value {
            ExternalDisplaySettings::Previous => DbExternalDisplaySettings::Previous,
            ExternalDisplaySettings::Native => DbExternalDisplaySettings::Native,
            ExternalDisplaySettings::Preference(v) => {
                DbExternalDisplaySettings::Preference(DbModePreference {
                    resolution: v.resolution.into(),
                    aspect_ratio: v.aspect_ratio.into(),
                    refresh: v.refresh.into(),
                })
            }
        }
    }
}

impl From<DbExternalDisplaySettings> for ExternalDisplaySettings {
    fn from(value: DbExternalDisplaySettings) -> Self {
        match value {
            DbExternalDisplaySettings::Previous => ExternalDisplaySettings::Previous,
            DbExternalDisplaySettings::Native => ExternalDisplaySettings::Native,
            DbExternalDisplaySettings::Preference(v) => {
                ExternalDisplaySettings::Preference(ModePreference {
                    resolution: v.resolution.into(),
                    aspect_ratio: v.aspect_ratio.into(),
                    refresh: v.refresh.into(),
                })
            }
        }
    }
}
