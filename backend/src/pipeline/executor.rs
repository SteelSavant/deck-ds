use anyhow::{anyhow, Context, Result};
use gilrs::{Button, Event, EventType, Gamepad, GamepadId};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use std::sync::mpsc::RecvTimeoutError;
use std::time::Instant;
use typemap::TypeMap;

use super::dependency::{Dependency, DependencyId};

use super::{
    action::{PipelineAction, PipelineActionExecutor},
    config::{PipelineDefinition, SelectionType},
    dependency::{true_video_wall::TrueVideoWall, DependencyExecutor},
};

pub struct PipelineExecutor {
    ctx: PipelineContext,
}

pub struct PipelineContext {
    /// path to directory containing contents of decky "defaults" folder
    pub defaults_dir: PathBuf,
    /// path to directory containing user configuration files
    pub config_dir: PathBuf,
    /// known dependencies
    pub dependencies: HashMap<DependencyId, Dependency>,
    /// pipeline state
    pub state: TypeMap,
}

impl PipelineExecutor {
    pub fn new(defaults_dir: PathBuf, config_dir: PathBuf) -> Self {
        Self {
            ctx: PipelineContext {
                defaults_dir,
                config_dir,
                dependencies: HashMap::from([(
                    TrueVideoWall::id(),
                    Dependency::TrueVideoWall(TrueVideoWall),
                )]),
                state: TypeMap::new(),
            },
        }
    }

    pub fn exec(&mut self, game_id: String, pipeline: &PipelineDefinition) -> Result<()> {
        let res = self.build(pipeline);

        match res {
            Err(err) => Err(anyhow::anyhow!(
                "Encountered errors assembling pipeline: {:?}",
                vec![err]
            )),
            Ok(pipeline) => {
                let mut run = vec![];
                let mut errors = vec![];

                for action in pipeline.iter() {
                    if let Err(err) = action.exec(&mut self.ctx, ActionType::Dependencies) {
                        return Err(err).with_context(|| format!("Error installing dependencies"));
                    }
                }

                for action in pipeline {
                    run.push(action);
                    let res = run
                        .last()
                        .expect("action should exist")
                        .exec(&mut self.ctx, ActionType::Setup);

                    if let Err(err) = res {
                        errors.push(err);
                        break;
                    }
                }

                if errors.is_empty() {
                    if let Err(err) = self.run_app(game_id) {
                        errors.push(err);
                    }
                }

                for action in run.into_iter().rev() {
                    let ctx = &mut self.ctx;

                    let res = action.exec(ctx, ActionType::Teardown);
                    if let Err(err) = res {
                        errors.push(err);
                    }
                }

                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!(
                        "Encountered errors executing pipeline: {:?}",
                        errors
                    ))
                }
            }
        }
    }

    #[allow(unused_variables)]
    fn run_app(&self, game_id: String,) -> Result<ExitStatus> {
        let mut app_handle = Command::new("steam")
            .arg(format!("steam://rungameid/{game_id}"))
            .spawn()
            .with_context(|| format!("Error starting application {game_id}"))?;

        let (send, recv) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let mut gilrs = gilrs::Gilrs::new().unwrap();
            let mut state = IndexMap::<GamepadId, (bool, bool, Option<Instant>)>::new();

            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));

                while let Some(Event { id, event, time }) = gilrs.next_event() {
                    match event {
                        EventType::ButtonPressed(
                            btn @ (gilrs::Button::Start | gilrs::Button::Select),
                            _,
                        ) => {
                            let entry = state.entry(id).or_default();
                            if btn == Button::Start {
                                entry.0 = true;
                            } else {
                                entry.1 = true;
                            }

                            if let &mut (true, true, None) = entry {
                                entry.2 = Some(Instant::now())
                            }
                        }
                        EventType::ButtonReleased(
                            btn @ (gilrs::Button::Start | gilrs::Button::Select),
                            _,
                        ) => {
                            let entry = state.entry(id).or_default();
                            if btn == Button::Start {
                                entry.0 = false;
                            } else {
                                entry.1 = false;
                            }
                            entry.2 = None;
                        }
                        EventType::Connected => {
                            let gamepad = gilrs.gamepad(id);

                            fn check_pressed(gamepad: Gamepad, btn: Button) -> bool {
                                gamepad
                                    .button_data(btn)
                                    .map(|data| data.is_pressed())
                                    .unwrap_or_default()
                            }

                            let start_pressed = check_pressed(gamepad, Button::Start);
                            let select_pressed = check_pressed(gamepad, Button::Select);
                            let instant = if start_pressed && select_pressed {
                                Some(Instant::now())
                            } else {
                                None
                            };

                            state.insert(id, (start_pressed, select_pressed, instant));
                        }
                        EventType::Disconnected => {
                            state.remove(&id);
                        }
                        _ => (),
                    }
                }

                println!("Gamepad State: {state:?}");

                for (_, _, instant) in state.values() {
                    let hold_duration = std::time::Duration::from_secs(3);
                    if matches!(instant, &Some(i) if i.elapsed() > hold_duration) {
                        println!("Received exit signal. Closing application...");
                        let _ = send.send(());
                        return;
                    }
                }
            }
        });

        loop {
            // TODO::this is not correct; it waits for steam, not the application
            if let Ok(Some(exit_status)) = app_handle.try_wait() {
                return Ok(exit_status);
            }
            let timeout = std::time::Duration::from_secs(2);
            if matches!(
                recv.recv_timeout(timeout),
                Ok(()) | Err(RecvTimeoutError::Disconnected)
            ) {
                // TODO::this is not correct; it kills the bash script/steam, not the launched application.
                let _ = app_handle.kill();
            }
        }
    }

    fn build(&self, pipeline: &PipelineDefinition) -> Result<Vec<PipelineAction>> {
        pipeline
            .actions
            .iter()
            .map(|s| {
                if matches!(s.optional, Some(true) | None) {
                    match &s.value {
                        SelectionType::Single(a) => Ok(vec![a.clone()]),
                        SelectionType::OneOf(values, key) => values
                            .get(key)
                            .ok_or(anyhow!("missing action {key}"))
                            .map(|a| vec![a.clone()]),
                        SelectionType::AnyOf(values, keys) => {
                            let mut ordered = keys
                                .iter()
                                .map(|k| {
                                    values
                                        .get_index_of(k)
                                        .map(|i| (i, k))
                                        .ok_or_else(|| anyhow!("missing action {k}"))
                                })
                                .collect::<Result<Vec<_>, _>>()?;
                            ordered.sort_by_key(|v| v.0);

                            Ok(ordered
                                .into_iter()
                                .map(|(_, k)| values[k].clone())
                                .collect())
                        }
                    }
                } else {
                    Ok(vec![])
                }
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|v| v.into_iter().flatten().collect())
    }
}

enum ActionType {
    Dependencies,
    Setup,
    Teardown,
}

impl PipelineAction {
    fn exec(&self, ctx: &mut PipelineContext, action: ActionType) -> Result<()> {
        match action {
            ActionType::Dependencies => {
                let ids = self.get_dependencies();

                let deps = ids
                    .iter()
                    .map(|id: &super::dependency::DependencyId| {
                        ctx.dependencies
                            .get(id)
                            .map(|d| (*d).clone())
                            .ok_or_else(|| anyhow!("missing dependency {id:?}"))
                    })
                    .collect::<Result<Vec<_>>>()?;

                for d in deps {
                    // TODO::consider tracking installs to avoid reinstalling dependencies
                    d.install(ctx)?;
                }

                Ok(())
            }
            ActionType::Setup => self.setup(ctx),
            ActionType::Teardown => self.tear_down(ctx),
        }
    }
}
