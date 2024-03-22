use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pipeline::action::multi_window::{OptionsRW, SCRIPT};

use super::launch_secondary_app::SecondaryAppWindowingBehavior;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SecondaryAppWindowOptions {
    pub window_matcher: String,
    pub classes: Vec<String>,
    pub windowing_behavior: SecondaryAppWindowingBehavior,
}

impl OptionsRW for SecondaryAppWindowOptions {
    fn load(kwin: &crate::sys::kwin::KWin) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let window_matcher = kwin
            .get_script_string_setting(SCRIPT, "secondaryAppWindowMatcher")?
            .unwrap_or_default();

        let classes = kwin
            .get_script_string_setting(SCRIPT, "secondaryAppWindowClasses")?
            .map(|s| s.split(',').map(|v| v.trim()).collect())
            .unwrap_or_default();

        let windowing_behavior = kwin
            .get_script_string_setting(SCRIPT, "secondaryAppWindowingBehavior")?
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or(SecondaryAppWindowingBehavior::PreferSecondary);

        Ok(Self {
            window_matcher,
            classes,
            windowing_behavior,
        })
    }

    fn write(&self, kwin: &crate::sys::kwin::KWin) -> anyhow::Result<()> {
        kwin.set_script_string_setting(SCRIPT, "secondaryAppWindowMatcher", &self.window_matcher)?;

        kwin.set_script_string_setting(
            SCRIPT,
            "secondaryAppClasses",
            &serde_json::to_string(&self.classes.join(",")),
        )?;

        kwin.set_script_string_setting(
            SCRIPT,
            "secondaryAppWindowingBehavior",
            &serde_json::to_string(&self.windowing_behavior),
        )?;

        Ok(())
    }
}
