use std::{thread::sleep, time::Duration};

use anyhow::Context;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    pipeline::{
        action::{ActionId, ActionImpl, ActionType},
        dependency::Dependency,
        executor::PipelineContext,
    },
    settings_db::MonitorDisplaySettings,
    sys::{
        display_info::get_display_info,
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

        let display_info = get_display_info()?;

        let display_settings = ctx
            .settings_db
            .get_monitor_display_settings(&display_info)?;

        update_touch(touch_mode, &display_settings);

        let handle = ctx
            .screen_tracking
            .as_mut()
            .context("TouchConfig requires kwin to be running")?
            .register_update(Box::new(move |_update| {
                update_touch(touch_mode, &display_settings);
            }));

        ctx.set_state::<Self>(TouchConfigState { handle });

        Ok(())
    }

    fn teardown(&self, ctx: &mut PipelineContext) -> anyhow::Result<()> {
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

fn update_touch(touch_mode: TouchSelectionMode, prefs: &MonitorDisplaySettings) {
    sleep(Duration::from_millis(100));
    let xdisplay = XDisplay::new();
    match xdisplay {
        Ok(mut xdisplay) => {
            let res = xdisplay.reconfigure_touch(touch_mode, prefs);
            if let Err(err) = res {
                log::warn!("failed to reconfigure touch after change event: {err}");
            }
        }
        Err(err) => log::warn!("failed to open xdisplay after change event: {err}"),
    }
}
