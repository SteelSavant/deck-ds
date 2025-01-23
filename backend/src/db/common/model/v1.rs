use serde::{Deserialize, Serialize};
use typetag::serde;

use crate::{
    pipeline::action::display_config::{ExternalDisplaySettings, RelativeLocation},
    sys::x_display::{AspectRatioOption, ModeOption, ModePreference, Resolution},
};

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DbModePreference {
    pub resolution: DbModeOption<DbResolution>,
    pub aspect_ratio: DbAspectRatioOption,
    pub refresh: DbModeOption<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DbModeOption<T> {
    Exact(T),
    AtLeast(T),
    AtMost(T),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DbResolution {
    pub w: u32, // TODO::enforce w is multiple of 8 for CVT
    pub h: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DbAspectRatioOption {
    Any,
    Native,
    Exact(f32),
}

impl<T, R> From<ModeOption<T>> for DbModeOption<R>
where
    R: From<T>,
{
    fn from(value: ModeOption<T>) -> Self {
        match value {
            ModeOption::Exact(v) => DbModeOption::Exact(v.into()),
            ModeOption::AtLeast(v) => DbModeOption::AtLeast(v.into()),
            ModeOption::AtMost(v) => DbModeOption::AtMost(v.into()),
        }
    }
}

impl From<Resolution> for DbResolution {
    fn from(value: Resolution) -> Self {
        Self {
            w: value.w,
            h: value.h,
        }
    }
}

impl From<AspectRatioOption> for DbAspectRatioOption {
    fn from(value: AspectRatioOption) -> Self {
        match value {
            AspectRatioOption::Any => DbAspectRatioOption::Any,
            AspectRatioOption::Native => DbAspectRatioOption::Native,
            AspectRatioOption::Exact(v) => DbAspectRatioOption::Exact(v),
        }
    }
}

impl<T, R> From<DbModeOption<T>> for ModeOption<R>
where
    R: From<T>,
{
    fn from(value: DbModeOption<T>) -> Self {
        match value {
            DbModeOption::Exact(v) => ModeOption::Exact(v.into()),
            DbModeOption::AtLeast(v) => ModeOption::AtLeast(v.into()),
            DbModeOption::AtMost(v) => ModeOption::AtMost(v.into()),
        }
    }
}

impl From<DbResolution> for Resolution {
    fn from(value: DbResolution) -> Self {
        Self {
            w: value.w,
            h: value.h,
        }
    }
}

impl From<DbAspectRatioOption> for AspectRatioOption {
    fn from(value: DbAspectRatioOption) -> Self {
        match value {
            DbAspectRatioOption::Any => AspectRatioOption::Any,
            DbAspectRatioOption::Native => AspectRatioOption::Native,
            DbAspectRatioOption::Exact(v) => AspectRatioOption::Exact(v),
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub enum DbRelativeLocation {
    Above,
    #[default]
    Below,
    LeftOf,
    RightOf,
}

impl From<RelativeLocation> for DbRelativeLocation {
    fn from(value: RelativeLocation) -> Self {
        match value {
            RelativeLocation::Above => DbRelativeLocation::Above,
            RelativeLocation::Below => DbRelativeLocation::Below,
            RelativeLocation::LeftOf => DbRelativeLocation::LeftOf,
            RelativeLocation::RightOf => DbRelativeLocation::RightOf,
        }
    }
}

impl From<DbRelativeLocation> for RelativeLocation {
    fn from(value: DbRelativeLocation) -> Self {
        match value {
            DbRelativeLocation::Above => RelativeLocation::Above,
            DbRelativeLocation::Below => RelativeLocation::Below,
            DbRelativeLocation::LeftOf => RelativeLocation::LeftOf,
            DbRelativeLocation::RightOf => RelativeLocation::RightOf,
        }
    }
}
