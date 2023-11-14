use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, Result};

use crate::{asset::AssetManager, pipeline::executor::PipelineExecutor, settings::Settings};

#[derive(Debug)]
pub struct AutoStart {
    settings: Arc<Mutex<Settings>>,
}

#[derive(Debug)]
pub struct LoadedAutoStart {
    autostart: crate::settings::AutoStart,
    settings: Arc<Mutex<Settings>>,
}

impl AutoStart {
    pub fn new(settings: Arc<Mutex<Settings>>) -> Self {
        Self { settings }
    }

    pub fn load(self) -> Result<Option<LoadedAutoStart>> {
        let autostart = {
            let settings = self
                .settings
                .lock()
                .expect("settings mutex should be lockable");
            settings.get_autostart()?
        };

        if let Some(autostart) = autostart {
            Ok(Some(LoadedAutoStart {
                autostart,
                settings: self.settings,
            }))
        } else {
            Ok(None)
        }
    }
}

impl LoadedAutoStart {
    pub fn new(autostart: crate::settings::AutoStart, settings: Arc<Mutex<Settings>>) -> Self {
        Self {
            autostart,
            settings,
        }
    }

    // TODO::teardown leftover

    pub fn build_executor(
        self,
        assets_manager: AssetManager,
        home_dir: PathBuf,
        config_dir: PathBuf,
    ) -> Result<PipelineExecutor> {
        let settings = self
            .settings
            .lock()
            .expect("settings mutex should be lockable");

        let profile = settings.get_profile(&self.autostart.profile_id)?;

        if let Some(definition) = settings
            .get_templates()
            .iter()
            .find(|pd| pd.id == profile.template)
        {
            let app_settings = settings
                .get_app(&self.autostart.app_id)?
                .and_then(|s| s.overrides.get(&definition.id).cloned());

            let patched = definition.patched_with(profile.overrides);
            let patched = app_settings
                .map(|o| patched.patched_with(o))
                .unwrap_or(patched);

            PipelineExecutor::new(
                self.autostart.app_id,
                patched,
                assets_manager,
                home_dir,
                config_dir,
            )
        } else {
            Err(anyhow!(
                "pipeline definition {:?} not found",
                profile.template
            ))
        }
    }
}
