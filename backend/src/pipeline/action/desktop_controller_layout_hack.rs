// TODO::this
// Allow - global, enabled, disabled -- PER TARGET

use anyhow::Ok;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::sys::steam;

use super::{ActionId, ActionImpl, ActionType};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
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
            let hack_steam = self
                .steam_override
                .unwrap_or(ctx.global_config.use_steam_desktop_controller_layout_hack);
            let hack_nonsteam = self.nonsteam_override.unwrap_or(
                ctx.global_config
                    .use_nonsteam_desktop_controller_layout_hack,
            );

            steam::set_desktop_controller_hack(
                hack_steam,
                hack_nonsteam,
                launch_info,
                &ctx.decky_env.steam_dir(),
            )
        } else {
            Ok(())
        }
    }

    fn teardown(&self, ctx: &mut crate::pipeline::executor::PipelineContext) -> anyhow::Result<()> {
        steam::unset_desktop_controller_hack(&ctx.decky_env.steam_dir())
    }
}
