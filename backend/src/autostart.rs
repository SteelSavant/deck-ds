use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, Result};

use crate::{
    asset::AssetManager,
    pipeline::{
        config::PipelineTarget, executor::PipelineExecutor, registar::PipelineActionRegistar,
    },
    settings::Settings,
};

#[derive(Debug)]
pub struct AutoStart {
    settings: Arc<Mutex<Settings>>,
}

#[derive(Debug)]
pub struct LoadedAutoStart {
    autostart: crate::settings::AutoStart,
    settings: Arc<Mutex<Settings>>,
    target: PipelineTarget,
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
                target: PipelineTarget::Desktop, // autostart load only invoked from desktop; gamemode has settings in memory
            }))
        } else {
            Ok(None)
        }
    }
}

impl LoadedAutoStart {
    pub fn new(
        autostart: crate::settings::AutoStart,
        settings: Arc<Mutex<Settings>>,
        target: PipelineTarget,
    ) -> Self {
        Self {
            autostart,
            settings,
            target,
        }
    }

    // TODO::teardown leftover

    pub fn build_executor<'a>(
        self,
        assets_manager: AssetManager<'a>,
        home_dir: PathBuf,
        config_dir: PathBuf,
        action_registrar: &PipelineActionRegistar,
    ) -> Result<PipelineExecutor<'a>> {
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

            let patched = definition.patched_with(profile.overrides, self.target, action_registrar);
            let patched = app_settings
                .map(|o| patched.patched_with(o, self.target, action_registrar))
                .unwrap_or(patched);

            PipelineExecutor::new(
                self.autostart.app_id,
                patched,
                self.target,
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
