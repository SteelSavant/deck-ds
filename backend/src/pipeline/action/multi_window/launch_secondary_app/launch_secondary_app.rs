use std::process::Command;

use anyhow::{Context, Result};
use nix::unistd::Pid;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    pipeline::action::{multi_window::OptionsRW, ActionId, ActionImpl, ActionType},
    secondary_app::{FlatpakApp, SecondaryApp},
    sys::windowing::get_window_info_from_pid,
};

use super::secondary_app_options::SecondaryAppWindowOptions;

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize, JsonSchema)]
pub struct LaunchSecondaryApp {
    pub id: ActionId,
    pub app: SecondaryApp,
    pub windowing_behavior: SecondaryAppWindowingBehavior,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Deserialize, JsonSchema)]
pub enum SecondaryAppWindowingBehavior {
    PreferSecondary,
    PreferPrimary,
    Hidden,
    Unmanaged,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SecondaryAppState {
    pid: Option<Pid>,
    options: SecondaryAppWindowOptions,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SerializableSecondaryAppState {
    options: SecondaryAppWindowOptions,
}

impl Serialize for SecondaryAppState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerializableSecondaryAppState {
            options: self.options.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SecondaryAppState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        SerializableSecondaryAppState::deserialize(deserializer).map(|v| SecondaryAppState {
            pid: None,
            options: v.options,
        })
    }
}

impl ActionImpl for LaunchSecondaryApp {
    type State = SecondaryAppState;

    const TYPE: ActionType = ActionType::LaunchSecondaryApp;

    fn get_id(&self) -> ActionId {
        self.id
    }

    fn setup(&self, ctx: &mut crate::pipeline::executor::PipelineContext) -> Result<()> {
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

    fn teardown(&self, ctx: &mut crate::pipeline::executor::PipelineContext) -> Result<()> {
        if let Some(pid) = ctx.get_state::<Self>().and_then(|state| {
            let _ = state.options.write(&ctx.kwin); // ignore result for now

            state.pid
        }) {
            self.app.teardown(pid)
        } else {
            Ok(())
        }
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
}

struct FlatpakStatus {
    app_id: String,
    pid: Pid,
    _instance: u32,
}

fn check_running_flatpaks() -> Result<Vec<FlatpakStatus>> {
    let output = Command::new("flatpak").arg("ps").output()?;
    if output.status.success() {
        let status = String::from_utf8_lossy(&output.stdout)
            .lines()
            .skip(1)
            .map(|v| v.split_ascii_whitespace())
            .map(|mut v| {
                let status = FlatpakStatus {
                    _instance: v
                        .next()
                        .with_context(|| "instance number expected")?
                        .parse()?,
                    pid: Pid::from_raw(v.next().with_context(|| "pid expected")?.parse()?),
                    app_id: v
                        .next()
                        .with_context(|| "expected flatpak app id")?
                        .to_string(),
                };
                Ok(status)
            })
            .collect::<Result<_>>()?;
        Ok(status)
    } else {
        Err(anyhow::anyhow!(
            String::from_utf8_lossy(&output.stderr).to_string()
        ))
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
