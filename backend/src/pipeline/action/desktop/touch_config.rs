use std::{thread::sleep, time::Duration};

use anyhow::Context;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    pipeline::{
        action::{ActionId, ActionImpl, ActionType},
        dependency::Dependency,
    },
    sys::{
        kwin::screen_tracking::KWinScreenTrackingUpdateHandle,
        x_display::{x_touch::TouchSelectionMode, XDisplay},
    },
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TouchConfig {
    pub id: ActionId,
    pub touch_mode: TouchSelectionMode,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TouchConfigState {
    handle: KWinScreenTrackingUpdateHandle,
}

impl ActionImpl for TouchConfig {
    type State = TouchConfigState;

    const TYPE: crate::pipeline::action::ActionType = ActionType::TouchConfig;

    fn setup(&self, ctx: &mut crate::pipeline::executor::PipelineContext) -> anyhow::Result<()> {
        let touch_mode = self.touch_mode;
        let handle = ctx
            .screen_tracking
            .as_mut()
            .context("TouchConfig requires kwin to be running")?
            .register_update(Box::new(move |_update| {
                sleep(Duration::from_millis(100));
                let xdisplay = XDisplay::new();
                match xdisplay {
                    Ok(mut xdisplay) => {
                        let res = xdisplay.reconfigure_touch(touch_mode);
                        if let Err(err) = res {
                            log::warn!("failed to reconfigure touch after change event: {err}");
                        }
                    }
                    Err(err) => log::warn!("failed to open xdisplay after change event: {err}"),
                }
            }));

        ctx.set_state::<Self>(TouchConfigState { handle });

        ctx.display
            .as_mut()
            .context("TouchConfig requires x11 to be running")?
            .reconfigure_touch(self.touch_mode)
    }

    fn teardown(&self, ctx: &mut crate::pipeline::executor::PipelineContext) -> anyhow::Result<()> {
        let handle = match ctx.get_state::<Self>() {
            Some(state) => state.handle,
            None => todo!(),
        };

        if let Some(screen_tracking) = ctx.screen_tracking.as_mut() {
            screen_tracking.unregister_update(handle)
        }

        Ok(())
    }

    fn get_dependencies(
        &self,
        _ctx: &crate::pipeline::executor::PipelineContext,
    ) -> Vec<Dependency> {
        vec![Dependency::Display]
    }

    #[inline]
    fn get_id(&self) -> ActionId {
        self.id
    }
}
