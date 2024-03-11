use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::sys::kwin::KWin;

use super::{LimitedMultiWindowLayout, MultiWindowLayout, OptionsRW, SCRIPT};
use anyhow::Result;
#[derive(Debug, Clone, SmartDefault, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CustomWindowOptions {
    pub primary_window_override: Option<String>,
    // secondary windows will be scraped from secondary apps after launch,
    // but secondary windows for the main app are registered here
    pub secondary_window_matcher: Option<String>,
    pub classes: Vec<String>,
    pub single_screen_layout: LimitedMultiWindowLayout,
    pub multi_screen_single_secondary_layout: MultiWindowLayout,
    pub multi_screen_multi_secondary_layout: MultiWindowLayout,
}

impl OptionsRW for CustomWindowOptions {
    fn load(kwin: &KWin) -> Result<Self>
    where
        Self: Sized,
    {
        let primary_window_override = kwin
            .get_script_string_setting(SCRIPT, "customPrimaryWindowMatcher")?
            .map(|v| v.trim().to_string())
            .and_then(|v| if v.is_empty() { None } else { Some(v) });

        let secondary_window_matcher = kwin
            .get_script_string_setting(SCRIPT, "customSecondaryWindowMatcher")?
            .map(|v| v.trim().to_string())
            .and_then(|v| if v.is_empty() { None } else { Some(v) });

        let classes = kwin
            .get_script_string_setting(SCRIPT, "customWindowClasses")?
            .map(|v| v.trim().to_string())
            .map(|v| {
                if v.is_empty() {
                    vec![]
                } else {
                    v.split(',').map(|s| s.trim().to_string()).collect()
                }
            })
            .unwrap_or_default();

        let single_screen_layout = kwin
            .get_script_string_setting(SCRIPT, "customSingleScreenLayout")?
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or(LimitedMultiWindowLayout::ColumnRight);
        let multi_screen_single_secondary_layout = kwin
            .get_script_string_setting(SCRIPT, "customMultiScreenSingleSecondaryLayout")?
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or(MultiWindowLayout::Separate);
        let multi_screen_multi_secondary_layout = kwin
            .get_script_string_setting(SCRIPT, "customMultiScreenMultiSecondaryLayout")?
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or(MultiWindowLayout::Separate);

        Ok(Self {
            primary_window_override,
            secondary_window_matcher,
            classes,
            single_screen_layout,
            multi_screen_single_secondary_layout,
            multi_screen_multi_secondary_layout,
        })
    }

    fn write(&self, kwin: &KWin) -> Result<()> {
        kwin.set_script_string_setting(
            SCRIPT,
            "customSingleScreenLayout",
            &serde_json::to_string(&self.single_screen_layout)?,
        )?;

        kwin.set_script_string_setting(
            SCRIPT,
            "customMultiScreenSingleSecondaryLayout",
            &serde_json::to_string(&self.multi_screen_single_secondary_layout)?,
        )?;

        kwin.set_script_string_setting(
            SCRIPT,
            "customMultiScreenMultiSecondaryLayout",
            &serde_json::to_string(&self.multi_screen_multi_secondary_layout)?,
        )?;

        kwin.set_script_string_setting(
            SCRIPT,
            "customPrimaryWindowMatcher",
            self.primary_window_override.as_deref().unwrap_or(&""),
        )?;

        kwin.set_script_string_setting(
            SCRIPT,
            "customSecondaryWindowMatcher",
            self.secondary_window_matcher.as_deref().unwrap_or(&""),
        )?;

        kwin.set_script_string_setting(SCRIPT, "customWindowClasses", &self.classes.join(","))?;

        Ok(())
    }
}
