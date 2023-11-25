use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::Result;

use crate::{
    asset::AssetManager,
    pipeline::{
        data::{PipelineTarget, ReifiablePipeline},
        executor::PipelineExecutor,
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
    ) -> Result<PipelineExecutor<'a>> {
        let settings = self
            .settings
            .lock()
            .expect("settings mutex should be lockable");

        let pipeline = self
            .autostart
            .pipeline
            .reify(&settings.get_profiles()?.as_slice())?;

        PipelineExecutor::new(
            self.autostart.app_id,
            pipeline,
            self.target,
            assets_manager,
            home_dir,
            config_dir,
        )
    }
}
