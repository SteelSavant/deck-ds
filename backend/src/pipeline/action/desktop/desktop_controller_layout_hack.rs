// TODO::Allow - global, enabled, disabled -- PER TARGET

use anyhow::{Context, Ok};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::pipeline::executor::PipelineContext;

use super::super::{ActionId, ActionImpl, ActionType};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct DesktopControllerLayoutHack {
    pub id: ActionId,
    pub steam_override: Option<bool>,
    pub nonsteam_override: Option<bool>,
}

impl ActionImpl for DesktopControllerLayoutHack {
    type State = ();

    const TYPE: ActionType = ActionType::DesktopControllerLayoutHack;

    fn get_id(&self) -> ActionId {
        self.id
    }

    fn setup(&self, ctx: &mut PipelineContext) -> anyhow::Result<()> {
        if let Some(launch_info) = ctx.launch_info.as_ref() {
            let hack_steam = self
                .steam_override
                .unwrap_or(ctx.global_config.use_steam_desktop_controller_layout_hack);
            let hack_nonsteam = self.nonsteam_override.unwrap_or(
                ctx.global_config
                    .use_nonsteam_desktop_controller_layout_hack,
            );

            if (!hack_steam && launch_info.is_steam_game)
                || (!hack_nonsteam && !launch_info.is_steam_game)
            {
                return Ok(());
            }

            ctx.register_on_launch_callback(Box::new(|_pid, ctx: &mut PipelineContext| {
                let app_id = ctx
                    .launch_info
                    .as_ref()
                    .expect("executing pipeline should have launch info")
                    .app_id
                    .clone();
                let status = std::process::Command::new("steam")
                    .arg(format!("steam://forceinputappid/{}", app_id.raw()))
                    .output()
                    .with_context(|| format!("Error starting application {:?}", app_id))?;

                if !status.status.success() {
                    log::error!(
                        "Failed to force input after: {}",
                        String::from_utf8_lossy(&status.stderr)
                    )
                }

                Ok(())
            }));
        }

        Ok(())
    }
}
