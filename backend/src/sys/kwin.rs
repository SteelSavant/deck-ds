use anyhow::Result;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

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

    pub fn install_script(&self, bundle_name: &str) -> Result<()> {
        let bundle = self.get_bundle(bundle_name).ok_or(anyhow::anyhow!(
            "could not find bundle {bundle_name} to install"
        ))?;
        let bundle_path = bundle.external_file_path()?;

        let output = Command::new("kpackagetool5")
            .args([&OsStr::new("-i"), bundle_path.as_os_str()])
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !stdout.contains("kpackagetool5 [options]")
            && (output.status.success()
                || stdout.contains("already exists")
                || stderr.contains("already exists"))
        {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "failed to install kwin script bundle {bundle_name}"
            ))
        }
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

    fn get_bundle<P: AsRef<Path>>(&self, bundle_name: P) -> Option<Asset> {
        self.assets_manager.get(self.bundles_dir.join(bundle_name))
    }

    fn reconfigure(&self) -> Result<()> {
        // TODO::inspect output/status for errors
        Ok(Command::new("qdbus")
            .args(["org.kde.KWin", "/KWin", "reconfigure"])
            .status()
            .map(|_| ())?)
    }
}
