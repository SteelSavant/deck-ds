use anyhow::{anyhow, Context, Result};
use gilrs::{Button, Event, EventType, Gamepad, GamepadId};
use indexmap::IndexMap;
use type_reg::untagged::TypeMap as SerdeMap;

use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime};
use typemap::{Key, TypeMap};

use crate::asset::AssetManager;
use crate::pipeline::action::cemu_layout::CemuLayout;
use crate::pipeline::action::citra_layout::CitraLayout;
use crate::pipeline::action::melonds_layout::MelonDSLayout;
use crate::pipeline::action::multi_window::MultiWindow;
use crate::pipeline::action::source_file::SourceFile;
use crate::pipeline::action::ui_management::{DisplayRestoration, SerialiableDisplayState};
use crate::pipeline::action::virtual_screen::VirtualScreen;
use crate::pipeline::data::{PipelineAction, Selection};
use crate::settings::AppId;
use crate::sys::app_process::AppProcess;
use crate::sys::kwin::KWin;
use crate::sys::x_display::XDisplay;

use super::action::ui_management::UiEvent;
use super::action::{Action, ErasedPipelineAction};
use super::data::{Pipeline, PipelineTarget};

use super::action::ActionImpl;

const PIPELINE_CONTEXT_STATE_PATH: &str = "/tmp/deckds-context.tmp";

pub struct PipelineExecutor<'a> {
    app_id: AppId,
    pipeline: Option<Pipeline>,
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
    pub display: Option<XDisplay>,
    /// actions that have run
    have_run: Vec<Action>,
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
    pub fn new(assets_manager: AssetManager<'a>, home_dir: PathBuf, config_dir: PathBuf) -> Self {
        PipelineContext {
            home_dir,
            config_dir,
            kwin: KWin::new(assets_manager),
            display: XDisplay::new().ok(),
            state: TypeMap::new(),
            have_run: vec![],
        }
    }

    pub fn get_state<P: ActionImpl + 'static>(&self) -> Option<&P::State> {
        self.state.get::<StateKey<P, P::State>>()
    }

    pub fn get_state_mut<P: ActionImpl + 'static>(&mut self) -> Option<&mut P::State> {
        self.state.get_mut::<StateKey<P, P::State>>()
    }

    pub fn set_state<P: ActionImpl + 'static>(&mut self, state: P::State) -> Option<P::State> {
        self.state.insert::<StateKey<P, P::State>>(state)
    }

    pub fn send_ui_event(&self, event: UiEvent) {
        let ui_state = self.get_state::<DisplayRestoration>();
        if let Some(ui_state) = ui_state {
            ui_state.send_ui_event(event);
        }
    }

    fn setup_action(&mut self, action: Action) -> Result<()> {
        let res = action
            .exec(self, ActionType::Setup)
            .with_context(|| format!("failed to execute setup for {}", action.name()));
        self.have_run.push(action);
        self.save_state().and(res)
    }

    fn teardown_action(&mut self, action: Action) -> Result<()> {
        let res = action
            .exec(self, ActionType::Teardown)
            .with_context(|| format!("failed to execute teardown for {}", action.name()));
        self.save_state().and(res)
    }

    fn save_state(&self) -> Result<()> {
        let mut map = SerdeMap::new();

        for action in self.have_run.iter() {
            match action {
                Action::DisplayRestoration(_) => map.insert(
                    action.name(),
                    self.get_state::<DisplayRestoration>()
                        .map(SerialiableDisplayState::from),
                ),
                Action::VirtualScreen(_) => {
                    map.insert(action.name(), self.get_state::<VirtualScreen>().cloned())
                }
                Action::MultiWindow(_) => {
                    map.insert(action.name(), self.get_state::<MultiWindow>().cloned())
                }
                Action::CitraLayout(_) => {
                    map.insert(action.name(), self.get_state::<CitraLayout>().cloned())
                }
                Action::CemuLayout(_) => {
                    map.insert(action.name(), self.get_state::<CemuLayout>().cloned())
                }
                Action::MelonDSLayout(_) => {
                    map.insert(action.name(), self.get_state::<MelonDSLayout>().cloned())
                }
                Action::SourceFile(_) => {
                    map.insert(action.name(), self.get_state::<SourceFile>().cloned())
                }
            };
        }

        let actions = self.have_run.iter().map(|a| a.name()).collect::<Vec<_>>();
        map.insert("__actions__", actions);

        let serialized = serde_json::to_string_pretty(&map)?;
        Ok(std::fs::write(
            Path::new(PIPELINE_CONTEXT_STATE_PATH),
            serialized,
        )?)
    }
}

impl<'a> PipelineExecutor<'a> {
    pub fn new(
        app_id: AppId,
        pipeline: Pipeline,
        target: PipelineTarget,
        assets_manager: AssetManager<'a>,
        home_dir: PathBuf,
        config_dir: PathBuf,
    ) -> Result<Self> {
        let s = Self {
            app_id,
            pipeline: Some(pipeline),
            target,
            ctx: PipelineContext::new(assets_manager, home_dir, config_dir),
        };

        Ok(s)
    }

    pub fn exec(mut self) -> Result<()> {
        // Set up pipeline
        let mut errors = vec![];

        let pipeline = self
            .pipeline
            .take()
            .with_context(|| "cannot execute pipeline; pipeline has already been executed")?
            .build_actions(self.target);

        // Install dependencies
        for action in pipeline.iter() {
            self.ctx.send_ui_event(UiEvent::UpdateStatusMsg(format!(
                "checking dependencies for {}...",
                action.name()
            )));

            if let Err(err) = action.exec(&mut self.ctx, ActionType::Dependencies) {
                return Err(err).with_context(|| "Error installing dependencies");
            }
        }

        // Setup
        for action in pipeline {
            self.ctx.send_ui_event(UiEvent::UpdateStatusMsg(format!(
                "setting up {}...",
                action.name()
            )));

            if let Err(err) = self.ctx.setup_action(action) {
                log::error!("{}", err);
                errors.push(err);
                break;
            }
        }

        if errors.is_empty() {
            self.ctx.send_ui_event(UiEvent::UpdateStatusMsg(
                "waiting for game launch...".to_string(),
            ));

            // Run app
            if let Err(err) = self.run_app() {
                log::error!("{}", err);
                errors.push(err);
            }
        }

        // Teardown
        while let Some(action) = self.ctx.have_run.pop() {
            let ctx: &mut PipelineContext<'_> = &mut self.ctx;

            ctx.send_ui_event(UiEvent::UpdateStatusMsg(format!(
                "tearing down {}...",
                action.name()
            )));

            let res = ctx.teardown_action(action);

            if let Err(err) = res {
                log::error!("{}", err);
                errors.push(err);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            let err = anyhow::anyhow!("Encountered errors executing pipeline: {:?}", errors);

            log::error!("{err}");
            Err(err)
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

        self.ctx.send_ui_event(UiEvent::ClearStatus);
        self.ctx.send_ui_event(UiEvent::UpdateWindowLevel(
            egui::WindowLevel::AlwaysOnBottom,
        ));

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

        self.ctx.send_ui_event(UiEvent::UpdateStatusMsg(
            "returning to game mode...".to_string(),
        ));

        self.ctx
            .send_ui_event(UiEvent::UpdateWindowLevel(egui::WindowLevel::AlwaysOnTop));

        Ok(())
    }
}

enum ActionType {
    Dependencies,
    Setup,
    Teardown,
}

impl Pipeline {
    fn build_actions(mut self, target: PipelineTarget) -> Vec<Action> {
        fn build_recursive(selection: Selection<PipelineAction>) -> Vec<Action> {
            match selection {
                Selection::Action(action) => vec![action],
                Selection::OneOf { selection, actions } => {
                    let action = actions
                        .into_iter()
                        .find(|a| a.id == selection)
                        .unwrap_or_else(|| panic!("Selection {selection:?} should exist"));

                    build_recursive(action.selection)
                }
                Selection::AllOf(actions) => actions
                    .into_iter()
                    .filter_map(|a| match a.enabled {
                        None | Some(true) => Some(a.selection),
                        Some(false) => None,
                    })
                    .flat_map(|a| build_recursive(a))
                    .collect(),
            }
        }

        self.targets
            .remove(&target)
            .into_iter()
            .flat_map(move |action| build_recursive(action))
            .collect()
    }
}

impl Action {
    fn exec(&self, ctx: &mut PipelineContext, action: ActionType) -> Result<()> {
        match action {
            ActionType::Dependencies => {
                let deps = self.get_dependencies(ctx);

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
