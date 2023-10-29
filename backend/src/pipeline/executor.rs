use anyhow::{anyhow, Context, Result};
use gilrs::{Button, Event, EventType, Gamepad, GamepadId};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime};
use typemap::{Key, TypeMap};

use crate::settings::Profile;
use crate::sys::kwin::{KWin, KWinScriptConfig};
use crate::sys::process::AppProcess;
use crate::sys::x_display::XDisplay;

use super::action::display_teardown::{DisplayConfig, RelativeLocation, TeardownExternalSettings};
use super::action::virtual_screen::VirtualScreen;
use super::action::{ErasedPipelineAction, PipelineActionId};
use super::dependency::{Dependency, DependencyExecutor, DependencyId};

use super::{action::PipelineActionImpl, dependency::true_video_wall::TrueVideoWall};

pub struct PipelineExecutor {
    ctx: PipelineContext,
    actions: HashMap<PipelineActionId, Box<dyn ErasedPipelineAction>>,
}

pub struct PipelineContext {
    /// path to directory containing contents of decky "defaults" folder
    pub defaults_dir: PathBuf,
    /// path to directory containing user configuration files
    pub config_dir: PathBuf,
    /// known dependencies
    pub dependencies: HashMap<DependencyId, Dependency>,
    /// KWin script handler
    pub kwin: KWin,
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

impl PipelineContext {
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

impl PipelineExecutor {
    pub fn new(defaults_dir: PathBuf, config_dir: PathBuf) -> Result<Self> {
        let mut kwin = KWin::new(defaults_dir.join("kwin"));
        kwin.register(
            "TrueVideoWall".to_string(),
            KWinScriptConfig {
                enabled_key: "truevideowallEnabled".to_string(),
                bundle_name: Path::new("170914-truevideowall-1.0.kwinscript").to_path_buf(),
            },
        )
        .expect("TrueVideoWall script should exist");

        let mut s = Self {
            ctx: PipelineContext {
                defaults_dir,
                config_dir,
                dependencies: HashMap::from([(
                    TrueVideoWall::id(),
                    Dependency::TrueVideoWall(TrueVideoWall),
                )]),
                kwin: kwin,
                display: XDisplay::new()?,
                state: TypeMap::new(),
            },
            actions: HashMap::new(),
        }
        .with_action(DisplayConfig {
            teardown_external_settings: TeardownExternalSettings::Previous,
            teardown_deck_location: RelativeLocation::Below,
        })
        .with_action(VirtualScreen);

        Ok(s)
    }

    fn with_action<A: ErasedPipelineAction + 'static>(mut self, action: A) -> Self {
        self.actions.insert(action.id(), Box::new(action));
        self
    }

    // pub fn exec(&mut self, game_id: String, pipeline: &PipelineDefinition) -> Result<()> {
    //     let res = self.build(pipeline);

    //     match res {
    //         Ok(pipeline) => {
    //             // Install dependencies
    //             for action in pipeline.iter() {
    //                 if let Err(err) = action.exec(&mut self.ctx, ActionType::Dependencies) {
    //                     return Err(err).with_context(|| "Error installing dependencies");
    //                 }
    //             }

    //             // Set up pipeline
    //             let mut run = vec![];
    //             let mut errors = vec![];

    //             for action in pipeline {
    //                 run.push(action);
    //                 let res = run
    //                     .last()
    //                     .expect("action should exist")
    //                     .exec(&mut self.ctx, ActionType::Setup);

    //                 if let Err(err) = res {
    //                     errors.push(err);
    //                     break;
    //                 }
    //             }

    //             if errors.is_empty() {
    //                 // Run app
    //                 if let Err(err) = self.run_app(game_id) {
    //                     errors.push(err);
    //                 }
    //             }

    //             // Teardown pipeline
    //             for action in run.into_iter().rev() {
    //                 let ctx = &mut self.ctx;

    //                 let res = action.exec(ctx, ActionType::Teardown);
    //                 if let Err(err) = res {
    //                     errors.push(err);
    //                 }
    //             }

    //             if errors.is_empty() {
    //                 Ok(())
    //             } else {
    //                 Err(anyhow::anyhow!(
    //                     "Encountered errors executing pipeline: {:?}",
    //                     errors
    //                 ))
    //             }
    //         }
    //         Err(err) => Err(anyhow::anyhow!(
    //             "Encountered errors assembling pipeline: {:?}",
    //             vec![err]
    //         )),
    //     }
    // }

    fn run_app(&self, app_id: String) -> Result<()> {
        let status = Command::new("steam")
            .arg(format!("steam://rungameid/{app_id}"))
            .status()
            .with_context(|| format!("Error starting application {app_id}"))?;

        if !status.success() {
            return Err(anyhow!(
                "Steam command for application {app_id} failed with status {status}"
            ));
        }

        let app_process = AppProcess::find(Duration::from_secs(5))?;

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

    fn build(&self, profile: &Profile) -> Result<Vec<Box<dyn ErasedPipelineAction>>> {
        todo!()
        // pipeline
        //     .actions
        //     .iter()
        //     .map(|s| {
        //         if matches!(s.optional, Some(true) | None) {
        //             match &s.value {
        //                 SelectionType::Single(a) => Ok(vec![a.clone()]),
        //                 SelectionType::OneOf(values, key) => values
        //                     .get(key)
        //                     .ok_or(anyhow!("missing action {key}"))
        //                     .map(|a| vec![a.clone()]),
        //                 SelectionType::AnyOf(values, keys) => {
        //                     let mut ordered = keys
        //                         .iter()
        //                         .map(|k| {
        //                             values
        //                                 .get_index_of(k)
        //                                 .map(|i| (i, k))
        //                                 .ok_or_else(|| anyhow!("missing action {k}"))
        //                         })
        //                         .collect::<Result<Vec<_>, _>>()?;
        //                     ordered.sort_by_key(|v| v.0);

        //                     Ok(ordered
        //                         .into_iter()
        //                         .map(|(_, k)| values[k].clone())
        //                         .collect())
        //                 }
        //             }
        //         } else {
        //             Ok(vec![])
        //         }
        //     })
        //     .collect::<Result<Vec<_>, _>>()
        //     .map(|v| v.into_iter().flatten().map(|v| v.) .collect())
    }
}

enum ActionType {
    Dependencies,
    Setup,
    Teardown,
}

impl dyn ErasedPipelineAction {
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
