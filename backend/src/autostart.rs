use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::Result;

use crate::{
    asset::AssetManager,
    pipeline::{data::PipelineTarget, executor::PipelineExecutor},
    settings::Settings,
};

#[derive(Debug)]
pub struct AutoStart {
    settings: Arc<Mutex<Settings>>,
}

#[derive(Debug)]
pub struct LoadedAutoStart {
    autostart: crate::settings::AutoStart,
    target: PipelineTarget,
}

impl AutoStart {
    pub fn new(settings: Arc<Mutex<Settings>>) -> Self {
        Self { settings }
    }

    pub fn load(self) -> Option<LoadedAutoStart> {
        let autostart = {
            let settings = self
                .settings
                .lock()
                .expect("settings mutex should be lockable");
            settings.get_autostart_cfg()
        };

        autostart.map(|autostart| LoadedAutoStart {
            autostart,
            target: PipelineTarget::Desktop, // autostart load only invoked from desktop; gamemode has settings in memory
        })
    }
}

impl LoadedAutoStart {
    pub fn new(autostart: crate::settings::AutoStart, target: PipelineTarget) -> Self {
        Self { autostart, target }
    }

    // TODO::teardown leftover

    pub fn build_executor(
        self,
        assets_manager: AssetManager,
        home_dir: PathBuf,
        config_dir: PathBuf,
    ) -> Result<PipelineExecutor> {
        PipelineExecutor::new(
            self.autostart.app_id,
            self.autostart.pipeline,
            self.target,
            assets_manager,
            home_dir,
            config_dir,
        )
    }
}
