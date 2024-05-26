use std::{
    collections::HashMap,
    fmt::Debug,
    path::{Path, PathBuf},
};

use either::Either;
use smart_default::SmartDefault;

use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

newtype_uuid!(ProfileId);
newtype_strid!("The AppId in Steam", AppId);
newtype_strid!("The GameId in Steam", GameId);
newtype_strid!("The UserId in steam", SteamUserId64);

use crate::{
    decky_env::DeckyEnv,
    macros::{newtype_strid, newtype_uuid},
    pipeline::{
        action::session_handler::DesktopSessionHandler,
        data::{ExitHooks, Pipeline, PipelineDefinition, PipelineTarget},
    },
    util::create_dir_all,
    PACKAGE_NAME,
};

pub struct Settings {
    // Path vars
    system_autostart_dir: PathBuf,
    global_config_path: PathBuf,
    autostart_path: PathBuf,
    exe_path: PathBuf,
    env_source_path: PathBuf,
}

#[derive(Debug, SmartDefault, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GlobalConfig {
    pub display_restoration: DesktopSessionHandler,
    #[default(false)]
    pub restore_displays_if_not_executing_pipeline: bool,
    /// If true, inject buttons onto app action bar
    #[default(true)]
    pub enable_ui_inject: bool,
    /// If `enable_ui_inject` is true, set the "Play" button to this target
    #[default(PipelineTarget::Gamemode)]
    pub primary_ui_target: PipelineTarget,
    /// Button chord to be used to exit profiles that register for exit hooks.
    pub exit_hooks: ExitHooks,
    /// Overwrite the desktop layout with the game layout
    pub use_desktop_controller_layout_hack: bool,
}

impl Settings {
    pub fn new<P: AsRef<Path>>(exe_path: P, decky_env: &DeckyEnv) -> Self {
        Self {
            autostart_path: decky_env.decky_plugin_runtime_dir.join("autostart.json"),
            global_config_path: decky_env.decky_plugin_settings_dir.join("config.json"),
            system_autostart_dir: decky_env.deck_user_home.join(".config/autostart"),
            env_source_path: decky_env.decky_env_path().clone(),
            exe_path: exe_path.as_ref().to_owned(),
        }
    }

    // File data

    pub fn get_autostart_cfg(&self) -> Option<AutoStartConfig> {
        std::fs::read_to_string(&self.autostart_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
    }

    pub fn delete_autostart_cfg(&self) -> Result<()> {
        if self.autostart_path.exists() {
            std::fs::remove_file(&self.autostart_path)
                .with_context(|| "failed to remove autostart config")
        } else {
            Ok(())
        }
    }

    pub fn set_autostart_cfg(&self, autostart: &AutoStartConfig) -> Result<()> {
        // always set system autostart, since we (eventually) want to be able to auto-configure displays
        // whether or not an app is run
        create_dir_all(&self.system_autostart_dir)?;

        let desktop_contents = self.create_desktop_contents();

        let autostart_parent = self
            .autostart_path
            .parent()
            .expect("autostart.json path should have parent");

        // set autostart config

        create_dir_all(autostart_parent)?;

        let autostart_cfg = serde_json::to_string_pretty(autostart)?;

        std::fs::write(
            self.system_autostart_dir
                .join(format!("{PACKAGE_NAME}.desktop")),
            desktop_contents,
        )
        .with_context(|| "failed to create autostart desktop file")
        .and_then(move |_| {
            std::fs::write(&self.autostart_path, autostart_cfg)
                .with_context(|| "failed to create autostart config file")
        })
    }

    pub fn get_global_cfg(&self) -> GlobalConfig {
        std::fs::read_to_string(&self.global_config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn delete_global_cfg(&self) -> Result<()> {
        if self.global_config_path.exists() {
            std::fs::remove_file(&self.global_config_path)
                .with_context(|| "failed to remove global config")
        } else {
            Ok(())
        }
    }

    pub fn set_global_cfg(&self, global: &GlobalConfig) -> Result<()> {
        let global_parent = self
            .global_config_path
            .parent()
            .expect("config.json path should have parent");

        // set global config

        create_dir_all(global_parent)?;

        let global_cfg = serde_json::to_string_pretty(global)?;

        std::fs::write(&self.global_config_path, global_cfg)
            .with_context(|| "failed to create autostart config file")
    }

    fn create_desktop_contents(&self) -> String {
        r"[Desktop Entry]
Comment=Runs DeckDS plugin autostart script for dual screen applications.
Exec=$Exec
Path=$Path
Name=DeckDS
Type=Application"
            .replace(
                "$Exec",
                &format!(
                    "{} autostart \"{}\"",
                    self.exe_path
                        .to_str()
                        .expect("DeckDS server path should be valid unicode"),
                    self.env_source_path
                        .to_str()
                        .expect("DeckDS env source path should be valid unicode")
                ),
            )
            .replace(
                "$Path",
                self.exe_path
                    .parent()
                    .expect("DeckDS server path should have parent")
                    .to_str()
                    .expect("DeckDS server path should be valid unicode"),
            )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutoStartConfig {
    pub game_id: Either<AppId, GameId>,
    pub pipeline: Pipeline,
    pub env: DeckyEnv,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CategoryProfile {
    pub id: ProfileId,
    pub tags: Vec<String>,
    pub pipeline: PipelineDefinition,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]

pub struct AppProfile {
    pub id: AppId,
    pub default_profile: Option<ProfileId>,
    pub overrides: HashMap<ProfileId, PipelineDefinition>,
}

#[cfg(test)]
mod tests {

    use std::path::Path;

    use pretty_assertions::assert_eq;

    use crate::{consts::PACKAGE_NAME, decky_env::DeckyEnv, settings::Settings};

    #[test]
    fn test_desktop_contents_correct() {
        let settings = Settings::new(
            Path::new("test/out/homebrew/plugins")
                .join(PACKAGE_NAME)
                .join("bin/backend"),
            &DeckyEnv::new_test("desktop_contents"),
        );

        let actual = settings.create_desktop_contents();
        let expected = r#"[Desktop Entry]
Comment=Runs DeckDS plugin autostart script for dual screen applications.
Exec=test/out/homebrew/plugins/DeckDS/bin/backend autostart "test/out/env/desktop_contents/homebrew/data/DeckDS/decky.env"
Path=test/out/homebrew/plugins/DeckDS/bin
Name=DeckDS
Type=Application"#;

        assert_eq!(expected, actual);
    }
}
