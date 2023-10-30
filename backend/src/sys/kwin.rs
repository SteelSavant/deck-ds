use anyhow::Result;
use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone)]
pub struct KWin {
    bundles_path: PathBuf,
    scripts: HashMap<String, KWinScriptConfig>,
}

#[derive(Debug, Clone)]
pub struct KWinScriptConfig {
    pub enabled_key: String,
    pub bundle_name: PathBuf,
}

impl KWin {
    pub fn new(bundles_path: PathBuf) -> Self {
        println!("creating KWin with bundles at {:?}", bundles_path);

        assert!(bundles_path.is_dir());

        Self {
            bundles_path,
            scripts: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: String, config: KWinScriptConfig) -> Result<()> {
        if self.bundle_exists(&config.bundle_name) {
            self.scripts.insert(name, config);
            Ok(())
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
        let bundle_path = self.get_bundle_path(&script.bundle_name);
        let output = Command::new("kpackagetool5")
            .args([&OsStr::new("i"), bundle_path.as_os_str()])
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        if output.status.success() || stdout.contains("already installed") {
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

    fn bundle_exists<P: AsRef<Path>>(&self, bundle_name: P) -> bool {
        self.get_bundle_path(bundle_name).exists()
    }

    fn get_bundle_path<P: AsRef<Path>>(&self, bundle_name: P) -> PathBuf {
        self.bundles_path.join(bundle_name)
    }

    fn reconfigure(&self) -> Result<()> {
        // TODO::inspect output/status for errors
        Ok(Command::new("qdbus")
            .args(["org.kde.KWin", "/KWin", "reconfigure"])
            .status()
            .map(|_| ())?)
    }
}
