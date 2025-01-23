use std::path::PathBuf;

use include_dir::{include_dir, Dir};
use serde::{Deserialize, Serialize};

use anyhow::Result;

use crate::{asset::AssetManager, config::PathLocator, consts::PACKAGE_NAME, AppModes};

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

        let env = match mode {
            AppModes::Autostart { env_source } => {
                let env = std::fs::read_to_string(env_source)
                    .inspect_err(|err| {
                        log::warn!("Failed to read env source file {env_source}: {err}")
                    })
                    .map(|v| {
                        serde_json::from_str(&v)
                            .inspect_err(|err| {
                                log::warn!("Failed to parse env source file {env_source}: {err}")
                            })
                            .unwrap_or(default.clone())
                    })
                    .unwrap_or(default);

                // Use the settings saved in the autostart if possible, the default env settings otherwise

                PathLocator::new("", &env)
                    .get_autostart_cfg()
                    .map(|v| v.env)
                    .unwrap_or(env)
            }
            AppModes::Serve => Self {
                decky_user: decky::user().unwrap_or(default.decky_user),
                deck_user_home: std::env::var("DECKY_USER_HOME")
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
        };

        env.create_dirs();

        env
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

    pub fn steam_dir(&self) -> PathBuf {
        self.deck_user_home.join(".steam").join("steam")
    }

    fn create_dirs(&self) {
        for dir in [
            &self.deck_user_home,
            &self.decky_plugin_log_dir,
            &self.decky_plugin_runtime_dir,
            &self.decky_plugin_log_dir,
        ] {
            if !dir.exists() {
                crate::util::create_dir_all(dir).unwrap_or_else(|err| {
                    panic!("should be able to create env dir {dir:?}: {err:#?}")
                });
            }
        }
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

        // ensure clean environment before executing test
        if home.exists() {
            std::fs::remove_dir_all(&home).expect("should be able to clean up old test env");
        }

        let s = Self {
            decky_user: "deck".to_string(),
            decky_plugin_settings_dir: home.join("homebrew/settings").join(PACKAGE_NAME),
            decky_plugin_runtime_dir: home.join("homebrew/data").join(PACKAGE_NAME),
            decky_plugin_log_dir: home.join("homebrew/logs").join(PACKAGE_NAME),
            deck_user_home: home,
        };

        s.create_dirs();

        s
    }
}
