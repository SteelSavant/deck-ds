use std::{process::Command, time::Duration};

use anyhow::{Context, Result};
use nix::unistd::Pid;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::thread::sleep;

use crate::{
    pipeline::{
        action::{multi_window::OptionsRW, ActionId, ActionImpl, ActionType},
        dependency::Dependency,
        executor::PipelineContext,
    },
    secondary_app::FlatpakApp,
    sys::{
        flatpak::check_running_flatpaks, windowing::get_window_info_from_pid_default_active_after,
    },
    util::escape_string_for_regex,
};

use super::{
    secondary_app_options::SecondaryAppWindowOptions, SecondaryAppState,
    SecondaryAppWindowingBehavior,
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize, JsonSchema)]
pub struct LaunchSecondaryFlatpakApp {
    pub id: ActionId,
    pub app: FlatpakApp,
    pub windowing_behavior: SecondaryAppWindowingBehavior,
}

impl ActionImpl for LaunchSecondaryFlatpakApp {
    type State = SecondaryAppState;

    const TYPE: ActionType = ActionType::LaunchSecondaryFlatpakApp;

    fn get_id(&self) -> ActionId {
        self.id
    }

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        let index = ctx.get_state_index();

        todo!("set secondary app settings per-index");

        let pid = self
            .app
            .setup()?
            .with_context(|| format!("secondary app {self:?} not running"))?;
        let options = SecondaryAppWindowOptions::load(&ctx.kwin)
            .with_context(|| "failed to load kwin secondary window options")?;

        ctx.set_state::<Self>(SecondaryAppState {
            pid: Some(pid),
            options,
        });

        let window_info =
            get_window_info_from_pid_default_active_after(pid, Duration::from_secs(2))?; // TODO::find a better way to link a flatpak pid to its actual window (sandboxing means x11 sees the pid as either 0 or 2, instead of the one reported)

        SecondaryAppWindowOptions {
            window_matcher: escape_string_for_regex(window_info.name),
            classes: window_info.classes,
            windowing_behavior: self.windowing_behavior,
        }
        .write(&ctx.kwin)
        .with_context(|| "failed to write kwin secondary window options")
    }

    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()> {
        if let Some(pid) = ctx.get_state::<Self>().and_then(|state| {
            let _ = state.options.write(&ctx.kwin); // ignore result for now

            state.pid
        }) {
            self.app.teardown(pid)
        } else {
            Ok(())
        }
    }

    fn get_dependencies(&self, _ctx: &PipelineContext) -> Vec<Dependency> {
        self.app.get_dependencies()
    }
}

impl FlatpakApp {
    fn setup(&self) -> Result<Option<Pid>> {
        log::info!(
            "launching secondary flatpak app: {:?} {:?}",
            self.app_id,
            self.args
        );

        let mut child = Command::new("flatpak")
            .args(
                [
                    vec!["run".to_string(), "-p".to_string(), self.app_id.to_string()],
                    self.args.clone(),
                ]
                .concat(),
            )
            .spawn()?;

        sleep(Duration::from_millis(200));

        match child.try_wait() {
            Ok(Some(v)) => {
                if v.success() {
                    Ok(None)
                } else {
                    Err(anyhow::anyhow!(
                        "flatpak run {} exited with error",
                        self.app_id
                    ))
                }
            }
            Ok(None) => Ok(Some(Pid::from_raw(child.id() as i32))),
            Err(err) => Err(err)?,
        }
    }

    fn teardown(&self, pid: Pid) -> Result<()> {
        let running = check_running_flatpaks()?;

        if running
            .iter()
            .any(|v| v.pid == pid && v.app_id == self.app_id)
        {
            let status = Command::new("flatpak")
                .args(["kill", &self.app_id])
                .status()?;

            if status.success() {
                log::debug!("Closed flatpak with pid {pid}");

                Ok(())
            } else {
                Err(anyhow::anyhow!("failed to kill flatpak {}", self.app_id))
            }
        } else {
            log::debug!("Failed to find running flatpak with pid {pid} in {running:?}");
            Ok(())
        }
    }

    fn get_dependencies(&self) -> Vec<Dependency> {
        vec![
            Dependency::Flatpak(self.app_id.clone()),
            Dependency::System("xdotool".to_string()),
        ]
    }
}
