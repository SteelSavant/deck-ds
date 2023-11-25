use anyhow::{anyhow, Context, Result};
use gilrs::{Button, Event, EventType, Gamepad, GamepadId};
use indexmap::IndexMap;
use std::iter;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant, SystemTime};
use typemap::{Key, TypeMap};

use crate::asset::AssetManager;
use crate::pipeline::data::{PipelineAction, Selection, WrappedPipelineAction};
use crate::settings::AppId;
use crate::sys::kwin::KWin;
use crate::sys::process::AppProcess;
use crate::sys::x_display::XDisplay;

use super::action::{Action, ErasedPipelineAction};
use super::data::{ActionPipeline, PipelineTarget};

use super::action::ActionImpl;

pub struct PipelineExecutor<'a> {
    app_id: AppId,
    pipeline: ActionPipeline,
    target: PipelineTarget,
    ctx: PipelineContext<'a>,
}

pub struct PipelineContext<'a> {
    /// path to directory containing the user's home directory
    pub home_dir: PathBuf,
    /// path to directory containing user configuration files
    pub config_dir: PathBuf,
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
    pub fn get_state<P: ActionImpl + 'static>(&self) -> Option<&P::State> {
        self.state.get::<StateKey<P, P::State>>()
    }

    pub fn get_state_mut<P: ActionImpl + 'static>(&mut self) -> Option<&mut P::State> {
        self.state.get_mut::<StateKey<P, P::State>>()
    }

    pub fn set_state<P: ActionImpl + 'static>(&mut self, state: P::State) -> Option<P::State> {
        self.state.insert::<StateKey<P, P::State>>(state)
    }
}

impl<'a> PipelineExecutor<'a> {
    pub fn new(
        app_id: AppId,
        pipeline: ActionPipeline,
        target: PipelineTarget,
        assets_manager: AssetManager<'a>,
        home_dir: PathBuf,
        config_dir: PathBuf,
    ) -> Result<Self> {
        let s = Self {
            app_id,
            pipeline: pipeline,
            target,
            ctx: PipelineContext {
                home_dir,
                config_dir,

                kwin: KWin::new(assets_manager),
                display: XDisplay::new()?,
                state: TypeMap::new(),
            },
        };

        Ok(s)
    }

    pub fn exec(&mut self) -> Result<()> {
        // Set up pipeline
        let mut run = vec![];
        let mut errors = vec![];

        let pipeline = self.pipeline.build_actions(self.target);

        // Install dependencies
        for action in pipeline.iter() {
            if let Err(err) = action.exec(&mut self.ctx, ActionType::Dependencies) {
                return Err(err).with_context(|| "Error installing dependencies");
            }
        }

        // Setup
        for action in pipeline {
            run.push(action);
            let res = run
                .last()
                .expect("action should exist")
                .exec(&mut self.ctx, ActionType::Setup);

            if let Err(err) = res {
                log::error!("{}", err);
                errors.push(err);
                break;
            }
        }

        if errors.is_empty() {
            // Run app
            if let Err(err) = self.run_app() {
                log::error!("{}", err);
                errors.push(err);
            }
        }

        // Teardown
        for action in run.into_iter().rev() {
            let ctx = &mut self.ctx;

            let res = action.exec(ctx, ActionType::Teardown);
            if let Err(err) = res {
                log::error!("{}", err);
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

    fn run_app(&self) -> Result<()> {
        let app_id = self.app_id.raw();
        let launch_type = match self.target {
            PipelineTarget::Desktop => "rungameid",
            PipelineTarget::Gamemode => "launch",
        };

        let status = Command::new("steam")
            .arg(format!("steam://{launch_type}/{app_id}"))
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

        const BTN0: gilrs::Button = gilrs::Button::Start;
        const BTN1: gilrs::Button = gilrs::Button::Select;

        while app_process.is_alive() {
            std::thread::sleep(std::time::Duration::from_millis(100));
            while let Some(Event { id, event, time }) = gilrs.next_event() {
                fn create_instant(time: SystemTime) -> Instant {
                    let elapsed = time.elapsed().unwrap_or_default();
                    Instant::now() - elapsed
                }
                log::trace!("Event: {:?}", event);
                match event {
                    EventType::ButtonPressed(btn @ (BTN0 | BTN1), _) => {
                        let entry = state.entry(id).or_default();
                        if btn == BTN0 {
                            entry.0 = true;
                        } else {
                            entry.1 = true;
                        }

                        if let &mut (true, true, None) = entry {
                            entry.2 = Some(create_instant(time))
                        }
                    }
                    EventType::ButtonReleased(btn @ (BTN0 | BTN1), _) => {
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

                        let btn0_pressed = check_pressed(gamepad, BTN0);
                        let btn1_pressed = check_pressed(gamepad, BTN1);
                        let instant = if btn0_pressed && btn1_pressed {
                            Some(create_instant(time))
                        } else {
                            None
                        };

                        state.insert(id, (btn0_pressed, btn1_pressed, instant));
                    }
                    EventType::Disconnected => {
                        state.remove(&id);
                    }
                    _ => (),
                }
            }

            log::trace!("Gamepad State: {state:?}");

            for (_, _, instant) in state.values() {
                let hold_duration = std::time::Duration::from_secs(2);
                if matches!(instant, &Some(i) if i.elapsed() > hold_duration) {
                    log::info!("Received exit signal. Closing application...");

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

impl ActionPipeline {
    fn build_actions(&self, target: PipelineTarget) -> Vec<&Action> {
        fn build_recursive(selection: &Selection<WrappedPipelineAction>) -> Vec<&Action> {
            match selection {
                Selection::Action(action) => vec![action],
                Selection::OneOf { selection, actions } => {
                    let action = actions
                        .iter()
                        .find(|a| a.0.id == *selection)
                        .unwrap_or_else(|| panic!("Selection {selection:?} should exist"));

                    build_recursive(&action.0.selection)
                }
                Selection::AllOf(actions) => actions
                    .iter()
                    .filter_map(|a| match a.enabled {
                        None | Some(true) => Some(&a.selection),
                        Some(false) => None,
                    })
                    .flat_map(|a| build_recursive(&a.0.selection))
                    .collect(),
            }
        }

        self.targets
            .get(&target)
            .into_iter()
            .flat_map(move |action| build_recursive(action))
            .collect()
    }
}

impl Action {
    fn exec(&self, ctx: &mut PipelineContext, action: ActionType) -> Result<()> {
        match action {
            ActionType::Dependencies => {
                let deps = self.get_dependencies();

                for d in deps {
                    d.verify_or_install(ctx)?;
                }

                Ok(())
            }
            ActionType::Setup => self.setup(ctx),
            ActionType::Teardown => self.teardown(ctx),
        }
    }
}
