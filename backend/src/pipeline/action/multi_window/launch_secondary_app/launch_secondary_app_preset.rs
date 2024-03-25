use std::process::Command;

use anyhow::{Context, Result};
use nix::unistd::Pid;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    pipeline::{
        action::{multi_window::OptionsRW, ActionId, ActionImpl, ActionType},
        dependency::Dependency,
    },
    secondary_app::{FlatpakApp, SecondaryApp, SecondaryAppPresetId},
    sys::{flatpak::check_running_flatpaks, windowing::get_window_info_from_pid},
};

use super::{LaunchSecondaryApp, SecondaryAppState, SecondaryAppWindowingBehavior};

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize, JsonSchema)]
pub struct LaunchSecondaryAppPreset {
    pub id: ActionId,
    pub preset: SecondaryAppPresetId,
    pub windowing_behavior: SecondaryAppWindowingBehavior,
}

impl ActionImpl for LaunchSecondaryAppPreset {
    type State = ();

    const TYPE: ActionType = ActionType::LaunchSecondaryAppPreset;

    fn setup(&self, ctx: &mut crate::pipeline::executor::PipelineContext) -> Result<()> {
        let mut presets = ctx.secondary_app.get_presets();

        let preset = presets
            .remove(&self.preset)
            .with_context(|| format!("Secondary app preset {:?} should exist", self.preset))?;

        LaunchSecondaryApp {
            id: self.id,
            app: preset.app,
            windowing_behavior: self.windowing_behavior,
        }
        .setup(ctx)
    }

    fn teardown(&self, ctx: &mut crate::pipeline::executor::PipelineContext) -> Result<()> {
        let mut presets = ctx.secondary_app.get_presets();

        let preset = presets
            .remove(&self.preset)
            .with_context(|| format!("Secondary app preset {:?} should exist", self.preset))?;

        LaunchSecondaryApp {
            id: self.id,
            app: preset.app,
            windowing_behavior: self.windowing_behavior,
        }
        .teardown(ctx)
    }

    fn get_id(&self) -> ActionId {
        self.id
    }

    fn get_dependencies(
        &self,
        _ctx: &mut crate::pipeline::executor::PipelineContext,
    ) -> Vec<crate::pipeline::dependency::Dependency> {
        vec![Dependency::SecondaryAppPreset(self.preset.clone())]
    }
}
