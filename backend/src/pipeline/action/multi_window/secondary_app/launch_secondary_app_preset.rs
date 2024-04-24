use anyhow::{Context, Result};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    pipeline::{
        action::{Action, ActionId, ActionImpl, ActionType, ErasedPipelineAction},
        dependency::Dependency,
    },
    secondary_app::SecondaryAppPresetId,
};

use super::{
    LaunchSecondaryFlatpakApp, SecondaryAppScreenPreference, SecondaryAppWindowingBehavior,
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize, JsonSchema)]
pub struct LaunchSecondaryAppPreset {
    pub id: ActionId,
    pub preset: SecondaryAppPresetId,
    pub windowing_behavior: SecondaryAppWindowingBehavior,
    pub screen_preference: SecondaryAppScreenPreference,
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
            crate::secondary_app::SecondaryApp::Flatpak(app) => {
                Action::from(LaunchSecondaryFlatpakApp {
                    id: self.id,
                    app,
                    windowing_behavior: self.windowing_behavior,
                    screen_preference: self.screen_preference,
                })
                .setup(ctx)
            }
        }
    }

    fn teardown(&self, ctx: &mut crate::pipeline::executor::PipelineContext) -> Result<()> {
        let mut presets = ctx.secondary_app.get_presets();

        let preset = presets
            .remove(&self.preset)
            .with_context(|| format!("Secondary app preset {:?} should exist", self.preset))?;

        match preset.app {
            crate::secondary_app::SecondaryApp::Flatpak(app) => {
                Action::from(LaunchSecondaryFlatpakApp {
                    id: self.id,
                    app,
                    windowing_behavior: self.windowing_behavior,
                    screen_preference: self.screen_preference,
                })
                .teardown(ctx)
            }
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
