use anyhow::{Context, Result};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    pipeline::{
        action::{ActionId, ActionImpl, ActionType},
        dependency::Dependency,
    },
    secondary_app::SecondaryAppPresetId,
};

use super::{LaunchSecondaryFlatpakApp, SecondaryAppWindowingBehavior};

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

        match preset.app {
            crate::secondary_app::SecondaryApp::Flatpak(app) => LaunchSecondaryFlatpakApp {
                id: self.id,
                app,
                windowing_behavior: self.windowing_behavior,
            }
            .setup(ctx),
        }
    }

    fn teardown(&self, ctx: &mut crate::pipeline::executor::PipelineContext) -> Result<()> {
        let mut presets = ctx.secondary_app.get_presets();

        let preset = presets
            .remove(&self.preset)
            .with_context(|| format!("Secondary app preset {:?} should exist", self.preset))?;

        match preset.app {
            crate::secondary_app::SecondaryApp::Flatpak(app) => LaunchSecondaryFlatpakApp {
                id: self.id,
                app,
                windowing_behavior: self.windowing_behavior,
            }
            .teardown(ctx),
        }
    }

    fn get_id(&self) -> ActionId {
        self.id
    }

    fn get_dependencies(
        &self,
        _ctx: &crate::pipeline::executor::PipelineContext,
    ) -> Vec<crate::pipeline::dependency::Dependency> {
        vec![Dependency::SecondaryAppPreset(self.preset)]
    }
}
