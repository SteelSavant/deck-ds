use std::time::Duration;

use anyhow::Context;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    pipeline::action::{Action, ActionId, ActionImpl, ActionType, ErasedPipelineAction},
    util::escape_string_for_regex,
};

use super::primary_windowing::{CustomWindowOptions, GeneralOptions, MultiWindow};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MainAppAutomaticWindowing {
    pub id: ActionId,
    pub general: GeneralOptions,
}

impl ActionImpl for MainAppAutomaticWindowing {
    type State = Action;

    const TYPE: ActionType = ActionType::MainAppAutomaticWindowing;

    fn get_id(&self) -> ActionId {
        self.id
    }

    fn setup(&self, ctx: &mut crate::pipeline::executor::PipelineContext) -> anyhow::Result<()> {
        let id = self.id;
        let general = self.general.clone();

        let window_ctx = ctx.kwin.start_tracking_new_windows()?;

        ctx.register_on_launch_callback(Box::new(move |_pid, ctx| {
            let new_windows = window_ctx.get_new_window_clients(Duration::from_secs(60))?;

            // TODO::actually get best window

            let best_window = new_windows
                .into_iter()
                .last()
                .context("automatic windowing expected to find a window")?;

            let multi = Action::from(MultiWindow {
                id,
                general: general.clone(),
                cemu: None,
                citra: None,
                dolphin: None,
                custom: Some(CustomWindowOptions {
                    primary_window_matcher: Some(escape_string_for_regex(best_window.caption)),
                    secondary_window_matcher: None,
                    classes: best_window.window_classes,
                    ..Default::default()
                }),
            });
            multi.setup(ctx)?;
            ctx.set_state::<Self>(multi);
            Ok(())
        }));

        Ok(())
    }

    fn teardown(&self, ctx: &mut crate::pipeline::executor::PipelineContext) -> anyhow::Result<()> {
        if let Some(multi) = ctx.get_state::<Self>().cloned() {
            multi.teardown(ctx)
        } else {
            Ok(())
        }
    }

    fn get_dependencies(
        &self,
        ctx: &crate::pipeline::executor::PipelineContext,
    ) -> Vec<crate::pipeline::dependency::Dependency> {
        ActionImpl::get_dependencies(
            &MultiWindow {
                id: ActionId::nil(),
                general: GeneralOptions::default(),
                cemu: None,
                citra: None,
                dolphin: None,
                custom: None,
            },
            ctx,
        )
    }
}
