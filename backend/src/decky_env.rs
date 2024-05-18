use std::path::PathBuf;

use include_dir::{include_dir, Dir};
use serde::{Deserialize, Serialize};

use anyhow::Result;

use crate::{asset::AssetManager, consts::PACKAGE_NAME, AppModes};

use usdpl_back::api::decky;

static ASSETS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets");

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeckyEnv {
    pub decky_user: String,
    pub deck_user_home: PathBuf,
    pub decky_plugin_settings_dir: PathBuf,
    pub decky_plugin_runtime_dir: PathBuf,
    pub decky_plugin_log_dir: PathBuf,
}

impl DeckyEnv {
    pub fn from_mode(mode: &AppModes) -> Self {
        let default = Self::default();

        match mode {
            AppModes::Autostart { env_source } => std::fs::read_to_string(env_source)
                .inspect_err(|err| log::warn!("Failed to read env source file {env_source}: {err}"))
                .map(|v| {
                    serde_json::from_str(&v)
                        .inspect_err(|err| {
                            log::warn!("Failed to parse env source file {env_source}: {err}")
                        })
                        .unwrap_or(default.clone())
                })
                .unwrap_or(default),
            AppModes::Serve => Self {
                decky_user: decky::user().unwrap_or(default.decky_user),
                deck_user_home: decky::home()
                    .map(PathBuf::from)
                    .unwrap_or(default.deck_user_home),
                decky_plugin_settings_dir: decky::settings_dir()
                    .map(PathBuf::from)
                    .unwrap_or(default.decky_plugin_settings_dir),
                decky_plugin_runtime_dir: decky::runtime_dir()
                    .map(PathBuf::from)
                    .unwrap_or(default.decky_plugin_runtime_dir),
                decky_plugin_log_dir: decky::log_dir()
                    .map(PathBuf::from)
                    .unwrap_or(default.decky_plugin_log_dir),
            },
            AppModes::Schema { .. } => default,
        }
    }

    pub fn write(&self) -> Result<()> {
        let contents = serde_json::to_string_pretty(&self)?;
        std::fs::write(self.decky_env_path(), contents)?;

        Ok(())
    }

    pub fn decky_env_path(&self) -> PathBuf {
        self.decky_plugin_runtime_dir.join("decky.env")
    }

    pub fn asset_manager(&self) -> AssetManager<'static> {
        AssetManager::new(&ASSETS_DIR, self.decky_plugin_settings_dir.join("assets"))
    }
}

impl Default for DeckyEnv {
    fn default() -> Self {
        let home = dirs::home_dir().expect("default home dir should exist");
        // These defaults aren't great, but its better than nothing. Ideally, they never get used.

        let log_dir = if home.join("homebrew").exists() {
            home.join("homebrew/logs/").join(PACKAGE_NAME)
        } else {
            PathBuf::from("/tmp")
        };

        Self {
            decky_user: "deck".to_string(), // TODO::better default
            decky_plugin_settings_dir: home.join("homebrew/settings/").join(PACKAGE_NAME),
            decky_plugin_runtime_dir: home.join("homebrew/data/").join(PACKAGE_NAME),
            deck_user_home: home,
            decky_plugin_log_dir: log_dir,
        }
    }
}

#[cfg(test)]
impl DeckyEnv {
    pub fn new_test(name: &str) -> Self {
        let home = PathBuf::from(format!("test/out/env/{name}"));
        Self {
            decky_user: "deck".to_string(),
            decky_plugin_settings_dir: home.join("homebrew/settings").join(PACKAGE_NAME),
            decky_plugin_runtime_dir: home.join("homebrew/data").join(PACKAGE_NAME),
            decky_plugin_log_dir: home.join("homebrew/logs").join(PACKAGE_NAME),
            deck_user_home: home,
        }
    }
}
