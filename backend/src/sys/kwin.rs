use anyhow::Result;
use include_dir::{Dir, File};
use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone)]
pub struct KWin<'a> {
    bundles_dir: &'a Dir<'a>,
    scripts: HashMap<String, KWinScriptConfig>,
}

#[derive(Debug, Clone)]
pub struct KWinScriptConfig {
    pub enabled_key: String,
    pub bundle_name: PathBuf,
}

impl<'a> KWin<'a> {
    pub fn new(bundles_dir: &'a Dir<'a>) -> Self {
        println!("creating KWin with bundles at {:?}", bundles_dir);

        Self {
            bundles_dir,
            scripts: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: String, config: KWinScriptConfig) -> Result<&mut Self> {
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
        let output = Command::new("kpackagetool5")
            .args([&OsStr::new("i"), bundle.path().as_os_str()])
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

    fn get_bundle<P: AsRef<Path>>(&self, bundle_name: P) -> Option<&'a File> {
        let rf = bundle_name.as_ref();
        self.bundles_dir
            .files()
            .filter(move |f| f.path().ends_with(rf))
            .next()
        // self.bundles_dir.get_file(bundle_name)
    }

    fn reconfigure(&self) -> Result<()> {
        // TODO::inspect output/status for errors
        Ok(Command::new("qdbus")
            .args(["org.kde.KWin", "/KWin", "reconfigure"])
            .status()
            .map(|_| ())?)
    }
}
