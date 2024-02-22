use anyhow::{Context, Result};

use std::{ffi::OsStr, path::PathBuf, process::Command, str::FromStr};

use crate::asset::{Asset, AssetManager};

#[derive(Debug)]
pub struct KWin<'a> {
    assets_manager: AssetManager<'a>,
    bundles_dir: PathBuf,
}

impl<'a> KWin<'a> {
    pub fn new(assets_manager: AssetManager<'a>) -> Self {
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
            .args([&OsStr::new("-i"), bundle_path.as_os_str()])
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let out = if output.status.success() && !stdout.contains("kpackagetool5 [options]") {
            Ok(())
        } else if stdout.contains("already exists") || stderr.contains("already exists") {
            let status = Command::new("kpackagetool5")
                .args([&OsStr::new("-u"), bundle_path.as_os_str()])
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
        };

        if out.is_ok() {
            self.reconfigure()?;
        }
        out
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
        if set_cmd_out.status.success() {
            self.reconfigure()
        } else {
            Err(anyhow::anyhow!(
                "Unable to {} {}: {}",
                if is_enabled { "enable" } else { "disable" },
                script_name,
                String::from_utf8_lossy(&set_cmd_out.stderr)
            ))
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
            log::debug!("got kwinrc {script_name} {key} as {trimmed}");

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
        log::debug!("setting kwinrc {script_name} {key} to {value}");
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

        Ok(self.reconfigure()?)
    }

    pub fn get_bundle(&self, script_name: &str) -> Option<Asset> {
        self.assets_manager.get(
            self.bundles_dir
                .join(script_name)
                .with_extension("kwinscript"),
        )
    }

    fn reconfigure(&self) -> Result<()> {
        // TODO::inspect output/status for errors
        Ok(Command::new("qdbus")
            .args(["org.kde.KWin", "/KWin", "reconfigure"])
            .status()
            .map(|_| ())?)
    }
}
