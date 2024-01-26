use anyhow::Result;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{
    asset::AssetManager,
    pipeline::{
        data::{PipelineAction, PipelineActionId, PipelineTarget, Selection},
        executor::PipelineExecutor,
    },
    settings::Settings,
};

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
            let config = settings.get_global_cfg();
            let autostart = settings.get_autostart_cfg();
            autostart.map(|mut a| {
                // Add global pipeline actions
                let desktop = a.pipeline.targets.remove(&PipelineTarget::Desktop);
                if let Some(desktop) = desktop {
                    a.pipeline.targets.insert(
                        PipelineTarget::Desktop,
                        Selection::AllOf(
                            vec![config.display_restoration.into(), desktop]
                                .into_iter()
                                .enumerate()
                                .map(|(index, action)| {
                                    let id = format!("internal:{index}");
                                    PipelineAction {
                                        id: PipelineActionId::new(&id),
                                        name: id,
                                        description: None,
                                        enabled: None,
                                        profile_override: None,
                                        selection: action,
                                        is_visible_on_qam: false,
                                    }
                                })
                                .collect(),
                        ),
                    );
                }

                a
            })
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
            self.autostart.game_id,
            self.autostart.pipeline,
            self.target,
            assets_manager,
            home_dir,
            config_dir,
        )
    }
}
