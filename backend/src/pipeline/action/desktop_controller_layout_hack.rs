// TODO::this
// Allow - global, enabled, disabled -- PER TARGET

use anyhow::Ok;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::sys::steam;

use super::{ActionId, ActionImpl, ActionType};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct DesktopControllerLayoutHack {
    pub id: ActionId,
    pub steam_override: Option<bool>,
    pub nonsteam_override: Option<bool>,
}

impl ActionImpl for DesktopControllerLayoutHack {
    type State = ();

    const TYPE: ActionType = ActionType::DesktopControllerLayoutHack;

    fn get_id(&self) -> super::ActionId {
        self.id
    }

    fn setup(&self, ctx: &mut crate::pipeline::executor::PipelineContext) -> anyhow::Result<()> {
        if let Some(launch_info) = ctx.launch_info.as_ref() {
            steam::set_desktop_controller_hack(launch_info, &ctx.decky_env.steam_dir())
        } else {
            Ok(())
        }
    }

    fn teardown(&self, ctx: &mut crate::pipeline::executor::PipelineContext) -> anyhow::Result<()> {
        steam::unset_desktop_controller_hack(&ctx.decky_env.steam_dir())
    }
}
