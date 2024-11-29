use anyhow::{Context, Result};
use regex::Regex;

use std::{
    ffi::OsStr, path::PathBuf, process::Command, str::FromStr, thread::sleep, time::Duration,
};

use crate::asset::{Asset, AssetManager};

pub use window_tracking::KWinClientMatcher;

pub mod screen_tracking;
mod window_tracking;

pub struct KWin {
    assets_manager: AssetManager<'static>,
    bundles_dir: PathBuf,
}

impl KWin {
    pub fn new(assets_manager: AssetManager<'static>) -> Self {
        Self {
            assets_manager,
            bundles_dir: PathBuf::from_str("kwin").expect("kwin path should be valid"),
        }
    }

    pub fn install_script(&self, script_name: &str) -> Result<()> {
        let bundle = self.get_bundle(script_name).ok_or(anyhow::anyhow!(
            "could not find bundle {script_name} to install"
        ))?;
        let bundle_path = bundle.file_path()?;

        let output = Command::new("kpackagetool5")
            .args([OsStr::new("-i"), bundle_path.as_os_str()])
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() && !stdout.contains("kpackagetool5 [options]") {
            Ok(())
        } else if stdout.contains("already exists") || stderr.contains("already exists") {
            let status = Command::new("kpackagetool5")
                .args([OsStr::new("-u"), bundle_path.as_os_str()])
                .status()
                .ok()
                .map(|s| s.success());

            if matches!(status, Some(true)) {
                Ok(())
            } else {
                Err(anyhow::anyhow!(
                    "failed to update kwin script bundle {script_name}"
                ))
            }
        } else {
            Err(anyhow::anyhow!(
                "failed to install kwin script bundle {script_name}"
            ))
        }
    }

    pub fn get_script_enabled(&self, script_name: &str) -> Result<bool> {
        let output = Command::new("kreadconfig5")
            .args([
                "--file",
                "kwinrc",
                "--group",
                "Plugins",
                "--key",
                &format!("{script_name}Enabled"),
            ])
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout == "true")
    }

    /// Enables/disables script and reconfigures KWin with current settings. Should be called after changing settings, not before.
    pub fn set_script_enabled(&self, script_name: &str, is_enabled: bool) -> Result<()> {
        let set_cmd_out = Command::new("kwriteconfig5")
            .args([
                "--file",
                "kwinrc",
                "--group",
                "Plugins",
                "--key",
                &format!("{}Enabled", script_name),
                &is_enabled.to_string(),
            ])
            .output()?;

        if !set_cmd_out.status.success() {
            Err(anyhow::anyhow!(
                "Unable to {} {}: {}",
                if is_enabled { "enable" } else { "disable" },
                script_name,
                String::from_utf8_lossy(&set_cmd_out.stderr)
            ))
        } else {
            Ok(())
        }
    }

    pub fn get_script_bool_setting(&self, script_name: &str, key: &str) -> Result<Option<bool>> {
        self.get_script_setting(script_name, key)
            .and_then(|v| v.map(|s: String| Ok(str::parse(&s)?)).transpose())
            .with_context(|| {
                format!(
                    "failed to get kwin bool setting: {} for {}",
                    key, script_name
                )
            })
    }

    pub fn get_script_string_setting(
        &self,
        script_name: &str,
        key: &str,
    ) -> Result<Option<String>> {
        self.get_script_setting(script_name, key).with_context(|| {
            format!(
                "failed to get kwin string setting: {} for {}",
                key, script_name
            )
        })
    }

    fn get_script_setting(&self, script_name: &str, key: &str) -> Result<Option<String>> {
        let output = Command::new("kreadconfig5")
            .args([
                "--file",
                "kwinrc",
                "--group",
                &format!("Script-{script_name}"),
                "--key",
                key,
            ])
            .output()?;

        output.status.exit_ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let trimmed = stdout.trim();
        if trimmed.is_empty() {
            Ok(None)
        } else {
            log::trace!("got kwinrc {script_name} {key} as {trimmed}");

            Ok(Some(trimmed.to_string()))
        }
    }

    pub fn set_script_bool_setting(&self, script_name: &str, key: &str, value: bool) -> Result<()> {
        self.set_script_setting(script_name, key, &value.to_string(), Some("bool"))
            .with_context(|| {
                format!(
                    "failed to set kwin bool setting: {} for {}",
                    key, script_name
                )
            })
    }

    pub fn set_script_string_setting(
        &self,
        script_name: &str,
        key: &str,
        value: &str,
    ) -> Result<()> {
        self.set_script_setting(script_name, key, value, None)
            .with_context(|| {
                format!(
                    "failed to set kwin string setting: {} for {}",
                    key, script_name
                )
            })
    }

    fn set_script_setting(
        &self,
        script_name: &str,
        key: &str,
        value: &str,
        ktype: Option<&str>,
    ) -> Result<()> {
        log::trace!("setting kwinrc {script_name} {key} to {value}");
        Command::new("kwriteconfig5")
            .args([
                "--file",
                "kwinrc",
                "--group",
                &format!("Script-{script_name}"),
                "--key",
                key,
                "--type",
                ktype.unwrap_or("string"),
                value,
            ])
            .status()?
            .exit_ok()?;

        Ok(())
    }

    pub fn get_bundle(&self, script_name: &str) -> Option<Asset> {
        self.assets_manager.get_file(
            self.bundles_dir
                .join(script_name)
                .with_extension("kwinscript"),
        )
    }

    /// Reconfigure KWin. Only works in Desktop mode.
    pub fn reconfigure(&self) -> Result<()> {
        let res = Command::new("qdbus")
            .args(["org.kde.KWin", "/KWin", "reconfigure"])
            .status()?;

        if res.success() {
            Ok(())
        } else {
            // TODO::get the actual error
            Err(anyhow::anyhow!("KWin failed to reconfigure"))
        }
    }

    pub fn start_tracking_new_windows(
        &self,
    ) -> Result<window_tracking::KWinNewWindowTrackingScope> {
        window_tracking::KWinNewWindowTrackingScope::new()
    }
}

pub fn next_active_window() -> Result<()> {
    let out = Command::new("qdbus")
        .args([
            "org.kde.kglobalaccel",
            "/component/kwin",
            "invokeShortcut",
            "Walk Through Windows",
        ])
        .output()?;

    if out.status.success() {
        // This is the best way, but broken in the current version of KDE; for now, using xdotool instead

        // let out = Command::new("qdbus")
        // .args([
        //     "org.kde.kglobalaccel",
        //     "/component/kwin",
        //     "invokeShortcut",
        //     "MoveMouseToFocus",
        // ])
        // .output()?;

        sleep(Duration::from_secs(2));

        if let Ok(out) = Command::new("xdotool")
            .args(["getwindowfocus", "getwindowgeometry"])
            .output()
        {
            if out.status.success() {
                let out = String::from_utf8_lossy(&out.stdout);
                let position = Regex::new(r"Position: (\d+),(\d+)")
                    .unwrap()
                    .captures(&out)
                    .and_then(|v| v.get(1).and_then(|x| v.get(2).map(|y| (x, y))));
                let geometry = Regex::new(r"Geometry: (\d+)x(\d+)")
                    .unwrap()
                    .captures(&out)
                    .and_then(|v| v.get(1).and_then(|w| v.get(2).map(|h| (w, h))));
                if let (Some(p), Some(g)) = (position, geometry) {
                    let x: u32 = p.0.as_str().parse().unwrap();
                    let y: u32 = p.1.as_str().parse().unwrap();
                    let w: u32 = g.0.as_str().parse().unwrap();
                    let h: u32 = g.1.as_str().parse().unwrap();

                    let mx = x + (w / 2);
                    let my = y + (h / 2);
                    let _ = Command::new("xdotool")
                        .args(["movemouse", &mx.to_string(), &my.to_string()])
                        .output();
                }
            }
        }

        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Failed to switch task windows: {}",
            String::from_utf8_lossy(&out.stderr).to_string()
        ))
    }
}
