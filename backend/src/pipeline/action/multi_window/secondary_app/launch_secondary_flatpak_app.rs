use std::{process::Command, time::Duration};

use anyhow::{Context, Result};
use nix::unistd::Pid;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::thread::sleep;

use crate::{
    pipeline::{
        action::{ActionId, ActionImpl, ActionType},
        dependency::Dependency,
        executor::PipelineContext,
    },
    secondary_app::FlatpakApp,
    sys::{
        flatpak::{check_running_flatpaks, list_installed_flatpaks},
        kwin::KWinClientMatcher,
    },
    util::{escape_string_for_regex, get_maybe_window_names_classes_from_title},
};

use super::{
    secondary_app_options::SecondaryAppWindowOptions, SecondaryAppScreenPreference,
    SecondaryAppState, SecondaryAppWindowingBehavior,
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize, JsonSchema)]
pub struct LaunchSecondaryFlatpakApp {
    pub id: ActionId,
    pub app: FlatpakApp,
    pub windowing_behavior: SecondaryAppWindowingBehavior,
    pub screen_preference: SecondaryAppScreenPreference,
}

impl ActionImpl for LaunchSecondaryFlatpakApp {
    type State = SecondaryAppState;

    const TYPE: ActionType = ActionType::LaunchSecondaryFlatpakApp;

    fn get_id(&self) -> ActionId {
        self.id
    }

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        let index = ctx
            .get_state_index::<Self>()
            .expect("state slot should exist");

        let window_ctx = ctx.kwin.start_tracking_new_windows()?;

        let pid = self
            .app
            .setup()?
            .with_context(|| format!("secondary app {self:?} not running"))?;
        let options = SecondaryAppWindowOptions::load(&ctx.kwin, index)
            .with_context(|| "failed to load kwin secondary window options")?;

        ctx.set_state::<Self>(SecondaryAppState {
            pid: Some(pid),
            options,
        });

        let best_window = window_ctx
            .get_best_window_client(KWinClientMatcher {
                min_delay: Duration::from_secs(2),
                max_delay: Duration::from_secs(30),
                preferred_ord_if_no_match: std::cmp::Ordering::Less,
                maybe_strings: self.app.get_maybe_window_names_classes(),
                // match_fn: Box::new(move |clients| clients.into_iter().next().cloned()),
            })?
            .context("automatic windowing expected to find a window")?;

        SecondaryAppWindowOptions {
            window_matcher: escape_string_for_regex(best_window.caption),
            classes: best_window.window_classes,
            windowing_behavior: self.windowing_behavior,
            screen_preference: self.screen_preference,
        }
        .write(&ctx.kwin, index)
        .with_context(|| "failed to write kwin secondary window options")
    }

    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()> {
        if let Some(pid) = ctx.get_state::<Self>().and_then(|state| {
            let index = ctx
                .get_state_index::<Self>()
                .expect("state slot should exist");

            let _ = state.options.write(&ctx.kwin, index); // ignore result for now

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
            .spawn()
            .context("flatpak process failed to spawn")?;

        sleep(Duration::from_millis(200));

        match child.try_wait() {
            Ok(Some(v)) => {
                if v.success() {
                    log::warn!("flatpak process for {} exited immediately...", self.app_id);
                    Ok(None)
                } else {
                    Err(anyhow::anyhow!(
                        "flatpak run {} exited with error",
                        self.app_id
                    ))
                }
            }
            Ok(None) => Ok(Some(Pid::from_raw(child.id() as i32))),
            Err(err) => Err(err).context("error waiting for flatpak process"),
        }
    }

    fn teardown(&self, _pid: Pid) -> Result<()> {
        // Don't teardown; -p arg already does it; TODO::clean this up
        // let running = check_running_flatpaks()?;

        // if running
        //     .iter()
        //     .any(|v| v.pid == pid && v.app_id == self.app_id)
        // {
        //     let status = Command::new("flatpak")
        //         .args(["kill", &self.app_id])
        //         .status()?;

        //     if status.success() {
        //         log::debug!("Closed flatpak with pid {pid}");

        //         Ok(())
        //     } else {
        //         Err(anyhow::anyhow!("failed to kill flatpak {}", self.app_id))
        //     }
        // } else {
        //     log::debug!("Failed to find running flatpak with pid {pid} in {running:?}");
        //     Ok(())
        // }

        Ok(())
    }

    fn get_dependencies(&self) -> Vec<Dependency> {
        vec![
            Dependency::Flatpak(self.app_id.clone()),
            Dependency::System("xdotool".to_string()),
        ]
    }

    fn get_maybe_window_names_classes(&self) -> Vec<String> {
        let title = list_installed_flatpaks()
            .unwrap_or_default()
            .into_iter()
            .find(|v| v.app_id == self.app_id)
            .map(|v| v.name);
        let id = self.app_id.to_string();
        let mut parts = self
            .app_id
            .split(".")
            .map(|v| v.to_string())
            .collect::<Vec<_>>();

        parts.push(id);

        if let Some(title) = title {
            parts.append(&mut get_maybe_window_names_classes_from_title(&title));
        }

        parts
    }
}
