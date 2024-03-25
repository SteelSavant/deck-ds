use std::process::Command;

use anyhow::Result;
use nix::unistd::Pid;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    pipeline::{
        action::{multi_window::OptionsRW, ActionId, ActionImpl, ActionType},
        dependency::Dependency,
        executor::PipelineContext,
    },
    secondary_app::{FlatpakApp, SecondaryApp},
    sys::{flatpak::check_running_flatpaks, windowing::get_window_info_from_pid},
};

use super::{
    secondary_app_options::SecondaryAppWindowOptions, SecondaryAppState,
    SecondaryAppWindowingBehavior,
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize, JsonSchema)]
pub struct LaunchSecondaryApp {
    pub id: ActionId,
    pub app: SecondaryApp,
    pub windowing_behavior: SecondaryAppWindowingBehavior,
}

impl ActionImpl for LaunchSecondaryApp {
    type State = SecondaryAppState;

    const TYPE: ActionType = ActionType::LaunchSecondaryApp;

    fn get_id(&self) -> ActionId {
        self.id
    }

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        let pid = self.app.setup()?;
        let options = SecondaryAppWindowOptions::load(&ctx.kwin)?;
        ctx.set_state::<Self>(SecondaryAppState {
            pid: Some(pid),
            options,
        });

        let window_info = get_window_info_from_pid(pid)?;

        SecondaryAppWindowOptions {
            window_matcher: escape_string_for_regex(window_info.name),
            classes: window_info.classes,
            windowing_behavior: self.windowing_behavior,
        }
        .write(&ctx.kwin)
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

    fn get_dependencies(&self, ctx: &mut PipelineContext) -> Vec<Dependency> {
        self.app.get_dependencies(ctx)
    }
}

impl SecondaryApp {
    fn setup(&self) -> Result<Pid> {
        match self {
            SecondaryApp::Flatpak(app) => app.setup(),
            // SecondaryAppType::CliCmd { setup, .. } => setup.exec(),
        }
    }

    fn teardown(&self, pid: Pid) -> Result<()> {
        match self {
            SecondaryApp::Flatpak(app) => app.teardown(pid),
            // SecondaryApp::CliCmd { teardown, .. } => {
            //     // Kill old pid, ignore status in case its already exited
            //     let _ = Command::new("kill").arg(&pid.as_raw().to_string()).status();

            //     if let Some(teardown) = teardown {
            //         teardown.exec().map(|_| ())?
            //     }

            //     Ok(())
            // }
        }
    }

    fn get_dependencies(&self, _ctx: &mut PipelineContext) -> Vec<Dependency> {
        match self {
            SecondaryApp::Flatpak(flatpak) => flatpak.get_dependencies(),
        }
    }
}

impl FlatpakApp {
    fn setup(&self) -> Result<Pid> {
        let child = Command::new("flatpak")
            .args([vec!["run".to_string()], self.args.clone()].concat())
            .spawn()?;

        // TODO::maybe check exit status

        // match status.try_wait() {
        //     Ok(Some(v)) => {
        //         if v.success() {
        //             Ok(None)
        //         } else {
        //             Err(anyhow::anyhow!(
        //                 "flatpak run {} exited with error",
        //                 self.app_id
        //             ))
        //         }
        //     }
        //     Ok(None) => Ok(Some(Pid::from_raw(status.id() as i32))),
        //     Err(err) => Err(err)?,
        // }

        Ok(Pid::from_raw(child.id() as i32))
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
                Ok(())
            } else {
                Err(anyhow::anyhow!("failed to kill flatpak {}", self.app_id))
            }
        } else {
            Ok(())
        }
    }

    fn get_dependencies(&self) -> Vec<Dependency> {
        vec![Dependency::Flatpak(self.app_id.clone())]
    }
}

fn escape_string_for_regex(mut s: String) -> String {
    for c in [
        '\\', '^', '$', '*', '+', '?', '.', '(', ')', '|', '{', '}', '[', ']',
    ] {
        s = s.replace(c, &format!("\\{c}"));
    }

    s
}
