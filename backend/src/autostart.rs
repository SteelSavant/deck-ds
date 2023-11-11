use std::path::PathBuf;

use anyhow::{anyhow, Result};

use crate::{asset::AssetManager, pipeline::executor::PipelineExecutor, settings::Settings};

#[derive(Debug)]
pub struct AutoStart {
    settings: Settings,
}

#[derive(Debug)]
pub struct LoadedAutoStart {
    autostart: crate::settings::AutoStart,
    settings: Settings,
}

impl AutoStart {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    pub fn load(self) -> Result<Option<LoadedAutoStart>> {
        let autostart = self.settings.get_autostart()?;

        self.teardown_leftovers();

        if let Some(autostart) = autostart {
            Ok(Some(LoadedAutoStart {
                autostart,
                settings: self.settings,
            }))
        } else {
            Ok(None)
        }
    }

    fn teardown_leftovers(&self) {
        // TODO::this
    }
}

impl LoadedAutoStart {
    pub fn build_executor(
        self,
        assets_manager: AssetManager,
        home_dir: PathBuf,
        config_dir: PathBuf,
    ) -> Result<PipelineExecutor> {
        let profile = self.settings.get_profile(&self.autostart.profile_id)?;

        if let Some(definition) = self
            .settings
            .get_templates()
            .iter()
            .find(|pd| pd.id == profile.template)
        {
            let app_settings = self
                .settings
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
