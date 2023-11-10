use anyhow::{anyhow, Context, Result};
use gilrs::{Button, Event, EventType, Gamepad, GamepadId};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant, SystemTime};
use typemap::{Key, TypeMap};

use crate::asset::AssetManager;
use crate::pipeline::config::Selection;
use crate::settings::AppId;
use crate::sys::kwin::KWin;
use crate::sys::process::AppProcess;
use crate::sys::x_display::XDisplay;

use super::action::{ErasedPipelineAction, PipelineAction};
use super::config::PipelineDefinition;
use super::dependency::emulator_windowing::EmulatorWindowing;
use super::dependency::{Dependency, DependencyExecutor, DependencyId};

use super::{action::PipelineActionImpl, dependency::true_video_wall::TrueVideoWall};

pub struct PipelineExecutor<'a> {
    app_id: AppId,
    definition: PipelineDefinition,
    ctx: PipelineContext<'a>,
}

pub struct PipelineContext<'a> {
    /// path to directory containing user configuration files
    pub config_dir: PathBuf,
    /// known dependencies
    pub dependencies: HashMap<DependencyId, Dependency>,
    /// KWin script handler
    pub kwin: KWin<'a>,
    /// Display handler,
    pub display: XDisplay,
    /// pipeline state
    state: TypeMap,
}

// state impl

struct StateKey<S: Sized, T>(PhantomData<S>, PhantomData<T>);

impl<S, T> Key for StateKey<S, T>
where
    S: 'static,
    T: 'static,
{
    type Value = T;
}

impl<'a> PipelineContext<'a> {
    pub fn get_state<P: PipelineActionImpl + 'static>(&self) -> Option<&P::State> {
        self.state.get::<StateKey<P, P::State>>()
    }

    pub fn get_state_mut<P: PipelineActionImpl + 'static>(&mut self) -> Option<&mut P::State> {
        self.state.get_mut::<StateKey<P, P::State>>()
    }

    pub fn set_state<P: PipelineActionImpl + 'static>(
        &mut self,
        state: P::State,
    ) -> Option<P::State> {
        self.state.insert::<StateKey<P, P::State>>(state)
    }
}

impl<'a> PipelineExecutor<'a> {
    pub fn new(
        app_id: AppId,
        pipeline_definition: PipelineDefinition,
        assets_manager: AssetManager<'a>,
        config_dir: PathBuf,
    ) -> Result<Self> {
        let s = Self {
            app_id,
            definition: pipeline_definition,
            ctx: PipelineContext {
                config_dir,
                dependencies: HashMap::from([
                    (
                        TrueVideoWall::id(),
                        Dependency::TrueVideoWall(TrueVideoWall),
                    ),
                    (
                        EmulatorWindowing::id(),
                        Dependency::EmulatorWindowing(EmulatorWindowing),
                    ),
                ]),
                kwin: KWin::preregistered(assets_manager)?,
                display: XDisplay::new()?,
                state: TypeMap::new(),
            },
        };

        Ok(s)
    }

    pub fn exec(&mut self) -> Result<()> {
        let pipeline = self.definition.build_actions();

        // Install dependencies
        for action in pipeline.iter() {
            if let Err(err) = action.exec(&mut self.ctx, ActionType::Dependencies) {
                return Err(err).with_context(|| "Error installing dependencies");
            }
        }

        // Set up pipeline
        let mut run = vec![];
        let mut errors = vec![];

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
            // Run app
            if let Err(err) = self.run_app(self.app_id.raw()) {
                errors.push(err);
            }
        }

        // Teardown pipeline
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

    fn run_app(&self, app_id: &str) -> Result<()> {
        let status = Command::new("steam")
            .arg(format!("steam://rungameid/{app_id}"))
            .status()
            .with_context(|| format!("Error starting application {app_id}"))?;

        if !status.success() {
            return Err(anyhow!(
                "Steam command for application {app_id} failed with status {status}"
            ));
        }

        let app_process = AppProcess::find(Duration::from_secs(30))?;

        let mut gilrs = gilrs::Gilrs::new().unwrap();
        let mut state = IndexMap::<GamepadId, (bool, bool, Option<Instant>)>::new();

        while app_process.is_alive() {
            std::thread::sleep(std::time::Duration::from_millis(100));
            while let Some(Event { id, event, time }) = gilrs.next_event() {
                fn create_instant(time: SystemTime) -> Instant {
                    let elapsed = time.elapsed().unwrap_or_default();
                    Instant::now() - elapsed
                }
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
                            entry.2 = Some(create_instant(time))
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
                            Some(create_instant(time))
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
                let hold_duration = std::time::Duration::from_secs(2);
                if matches!(instant, &Some(i) if i.elapsed() > hold_duration) {
                    println!("Received exit signal. Closing application...");

                    return app_process.kill();
                }
            }
        }

        Ok(())
    }
}

enum ActionType {
    Dependencies,
    Setup,
    Teardown,
}

impl PipelineDefinition {
    fn build_actions(&self) -> Vec<&PipelineAction> {
        fn build_recursive(selection: &Selection) -> Vec<&PipelineAction> {
            match selection {
                Selection::Action(action) => vec![action],
                Selection::OneOf { selection, actions } => {
                    build_recursive(&actions[selection].selection)
                }
                Selection::AllOf(definitions) => definitions
                    .iter()
                    .flat_map(|d| build_recursive(&d.selection))
                    .collect(),
            }
        }

        build_recursive(&self.action.selection)
    }
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
            ActionType::Teardown => self.teardown(ctx),
        }
    }
}
