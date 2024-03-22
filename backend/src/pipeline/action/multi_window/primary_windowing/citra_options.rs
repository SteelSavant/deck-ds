use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::sys::kwin::KWin;

use super::{LimitedMultiWindowLayout, MultiWindowLayout, OptionsRW, SCRIPT};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CitraWindowOptions {
    pub single_screen_layout: LimitedMultiWindowLayout,
    pub multi_screen_layout: MultiWindowLayout,
}

impl OptionsRW for CitraWindowOptions {
    fn load(kwin: &KWin) -> Result<Self>
    where
        Self: Sized,
    {
        let single_screen_layout = kwin
            .get_script_string_setting(SCRIPT, "citraSingleScreenLayout")?
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or(LimitedMultiWindowLayout::ColumnRight);
        let multi_screen_layout = kwin
            .get_script_string_setting(SCRIPT, "citraMultiScreenSingleSecondaryLayout")?
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or(MultiWindowLayout::Separate);

        Ok(Self {
            single_screen_layout,
            multi_screen_layout,
        })
    }

    fn write(&self, kwin: &KWin) -> Result<()> {
        kwin.set_script_string_setting(
            SCRIPT,
            "citraSingleScreenLayout",
            &serde_json::to_string(&self.single_screen_layout)?,
        )?;

        kwin.set_script_string_setting(
            SCRIPT,
            "citraMultiScreenSingleSecondaryLayout",
            &serde_json::to_string(&self.multi_screen_layout)?,
        )?;

        Ok(())
    }
}
