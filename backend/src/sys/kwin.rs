use anyhow::Result;
use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
};

use crate::asset::{Asset, AssetManager};

#[derive(Debug)]
pub struct KWin<'a> {
    assets_manager: AssetManager<'a>,
    bundles_dir: PathBuf,
    scripts: HashMap<String, KWinScriptConfig>,
}

#[derive(Debug, Clone)]
pub struct KWinScriptConfig {
    pub enabled_key: String,
    pub bundle_name: PathBuf,
}

impl<'a> KWin<'a> {
    pub fn preregistered(assets_manager: AssetManager<'a>) -> Result<KWin<'a>> {
        Ok(KWin::new(assets_manager, "kwin".into())
            .register(
                "TrueVideoWall".to_string(),
                KWinScriptConfig {
                    enabled_key: "truevideowallEnabled".to_string(),
                    bundle_name: Path::new("truevideowall-1.0.kwinscript").to_path_buf(),
                },
            )
            .expect("TrueVideoWall script should exist")
            .register(
                "EmulatorWindowing".to_string(),
                KWinScriptConfig {
                    enabled_key: "emulatorwindowingEnabled".to_string(),
                    bundle_name: Path::new("emulatorwindowing-1.0.kwinscript").to_path_buf(),
                },
            )
            .expect("EmulatorWindowing script should exist"))
    }

    fn new(assets_manager: AssetManager<'a>, bundles_dir: PathBuf) -> Self {
        println!("creating KWin with bundles at {:?}", bundles_dir);

        Self {
            assets_manager,
            bundles_dir,
            scripts: HashMap::new(),
        }
    }

    pub fn register(mut self, name: String, config: KWinScriptConfig) -> Result<Self> {
        if self.get_bundle(&config.bundle_name).is_some() {
            self.scripts.insert(name, config);
            Ok(self)
        } else {
            Err(anyhow::anyhow!(
                "Could not find kwin bundle {}",
                config.bundle_name.display()
            ))
        }
    }

    pub fn install_script(&self, script_name: &str) -> Result<()> {
        let script = self.scripts.get(script_name).ok_or(anyhow::anyhow!(
            "No kwin script named {script_name} registered"
        ))?;
        let bundle = self.get_bundle(&script.bundle_name).ok_or(anyhow::anyhow!(
            "could not find bundle {script_name} to install"
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
                "failed to install kwin script {script_name}"
            ))
        }
    }

    pub fn set_script_enabled(&self, script_name: &str, is_enabled: bool) -> Result<()> {
        let script = self.scripts.get(script_name).ok_or(anyhow::anyhow!(
            "No kwin script named {script_name} registered"
        ))?;

        let set_cmd_out = Command::new("kwriteconfig5")
            .args([
                "--file",
                "kwinrc",
                "--group",
                "Plugins",
                "--key",
                &script.enabled_key,
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
