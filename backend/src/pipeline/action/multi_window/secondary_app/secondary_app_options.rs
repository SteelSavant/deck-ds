use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pipeline::action::multi_window::SCRIPT;

use super::{SecondaryAppScreenPreference, SecondaryAppWindowingBehavior};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SecondaryAppWindowOptions {
    pub window_matcher: String,
    pub classes: Vec<String>,
    pub windowing_behavior: SecondaryAppWindowingBehavior,
    pub screen_preference: SecondaryAppScreenPreference,
}

impl SecondaryAppWindowOptions {
    pub fn load(kwin: &crate::sys::kwin::KWin, index: usize) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let window_matcher = kwin
            .get_script_string_setting(SCRIPT, &format!("secondaryAppWindowMatcher{index}"))?
            .unwrap_or_default();

        let classes = kwin
            .get_script_string_setting(SCRIPT, &format!("secondaryAppWindowClasses{index}"))?
            .map(|s| {
                s.split(',')
                    .map(|v| v.trim().to_lowercase().to_string())
                    .collect()
            })
            .unwrap_or_default();

        let windowing_behavior = kwin
            .get_script_string_setting(SCRIPT, &format!("secondaryAppWindowingBehavior{index}"))?
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or(Default::default());

        let screen_preference = kwin
            .get_script_string_setting(SCRIPT, &format!("secondaryAppScreenPreference{index}"))?
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or(Default::default());

        Ok(Self {
            window_matcher,
            classes,
            windowing_behavior,
            screen_preference,
        })
    }

    pub fn write(&self, kwin: &crate::sys::kwin::KWin, index: usize) -> anyhow::Result<()> {
        kwin.set_script_string_setting(
            SCRIPT,
            &format!("secondaryAppWindowMatcher{index}"),
            &self.window_matcher,
        )?;

        kwin.set_script_string_setting(
            SCRIPT,
            &format!("secondaryAppWindowClasses{index}"),
            &self.classes.join(",").to_lowercase(),
        )?;

        kwin.set_script_string_setting(
            SCRIPT,
            &format!("secondaryAppWindowingBehavior{index}"),
            &serde_json::to_string(&self.windowing_behavior).unwrap(),
        )?;

        kwin.set_script_string_setting(
            SCRIPT,
            &format!("secondaryAppScreenPreference{index}"),
            &serde_json::to_string(&self.screen_preference).unwrap(),
        )?;

        Ok(())
    }
}
