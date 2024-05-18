use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{consts::PACKAGE_NAME, Modes};

use usdpl_back::api::decky;

#[derive(Clone, Debutg, Deserialize, Serialize)]
pub struct DeckyEnv {
    pub decky_user: String,
    pub deck_user_home: PathBuf,
    pub decky_plugin_settings_dir: PathBuf,
    pub decky_plugin_runtime_dir: PathBuf,
    pub decky_plugin_log_dir: PathBuf,
}

impl DeckyEnv {
    pub fn from_mode(mode: &Modes) -> Self {
        let default = Self::default();

        match mode {
            Modes::Autostart { env_source } => std::fs::read_to_string(&env_source)
                .inspect_err(|err| log::warn!("Failed to read env source file {env_source}: {err}"))
                .map(|v| {
                    serde_json::from_str(&v)
                        .inspect_err(|err| {
                            log::warn!("Failed to parse env source file {env_source}: {err}")
                        })
                        .unwrap_or(default.clone())
                })
                .unwrap_or(default),
            Modes::Serve => Self {
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
            Modes::Schema { .. } => default,
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

// settings/config
// settings/autostart.env
// settings/current.autostart -> source
// settings/previous.autostart?
