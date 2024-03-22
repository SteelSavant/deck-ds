use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::sys::kwin::KWin;

use super::{LimitedMultiWindowLayout, MultiWindowLayout, OptionsRW, SCRIPT};
use smart_default::SmartDefault;

#[derive(Debug, Clone, SmartDefault, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DolphinWindowOptions {
    pub single_screen_layout: LimitedMultiWindowLayout,
    pub multi_screen_single_secondary_layout: MultiWindowLayout,
    #[default(MultiWindowLayout::ColumnRight)]
    pub multi_screen_multi_secondary_layout: MultiWindowLayout,
    // GBA ids to blacklist, 1,2,3 or 4
    pub gba_blacklist: Vec<u8>,
}

impl OptionsRW for DolphinWindowOptions {
    fn load(kwin: &KWin) -> Result<Self>
    where
        Self: Sized,
    {
        let single_screen_layout = kwin
            .get_script_string_setting(SCRIPT, "dolphinSingleScreenLayout")?
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or(LimitedMultiWindowLayout::ColumnRight);
        let multi_screen_single_secondary_layout = kwin
            .get_script_string_setting(SCRIPT, "dolphinMultiScreenSingleSecondaryLayout")?
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or(MultiWindowLayout::Separate);
        let multi_screen_multi_secondary_layout = kwin
            .get_script_string_setting(SCRIPT, "dolphinMultiScreenMultiSecondaryLayout")?
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or(MultiWindowLayout::ColumnRight);

        let gba_blacklist = kwin
            .get_script_string_setting(SCRIPT, "dolphinBlacklist")?
            .map(|s| {
                s.split(',')
                    .filter_map(|v| {
                        let trimmed = v.trim().to_ascii_uppercase();
                        if trimmed.starts_with("GBA") && trimmed.len() == 4 {
                            trimmed.chars().last().unwrap().to_string().parse().ok()
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(Self {
            single_screen_layout,
            multi_screen_single_secondary_layout,
            multi_screen_multi_secondary_layout,
            gba_blacklist,
        })
    }

    fn write(&self, kwin: &KWin) -> Result<()> {
        kwin.set_script_string_setting(
            SCRIPT,
            "dolphinSingleScreenLayout",
            &serde_json::to_string(&self.single_screen_layout)?,
        )?;

        kwin.set_script_string_setting(
            SCRIPT,
            "dolphinMultiScreenSingleSecondaryLayout",
            &serde_json::to_string(&self.multi_screen_single_secondary_layout)?,
        )?;

        kwin.set_script_string_setting(
            SCRIPT,
            "dolphinMultiScreenMultiSecondaryLayout",
            &serde_json::to_string(&self.multi_screen_multi_secondary_layout)?,
        )?;

        kwin.set_script_string_setting(
            SCRIPT,
            "dolphinBlacklist",
            &self
                .gba_blacklist
                .iter()
                .map(|v| format!("GBA{v}"))
                .collect::<Vec<_>>()
                .join(","),
        )?;

        Ok(())
    }
}
