use anyhow::{anyhow, Context, Result};
use egui::ahash::HashSet;
use either::Either;
use gilrs::{Button, Event, EventType, Gamepad, GamepadId};
use indexmap::IndexMap;
use nix::unistd::Pid;
use type_reg::untagged::{TypeMap as SerdeMap, TypeReg};

use std::marker::PhantomData;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use typemap_ors::{Key, TypeMap};

use crate::decky_env::DeckyEnv;
use crate::pipeline::action::cemu_audio::CemuAudio;
use crate::pipeline::action::cemu_layout::CemuLayout;
use crate::pipeline::action::citra_layout::CitraLayout;
use crate::pipeline::action::desktop_controller_layout_hack::DesktopControllerLayoutHack;
use crate::pipeline::action::display_config::DisplayConfig;
use crate::pipeline::action::lime_3ds_layout::Lime3dsLayout;
use crate::pipeline::action::melonds_layout::MelonDSLayout;
use crate::pipeline::action::multi_window::main_app_automatic_windowing::MainAppAutomaticWindowing;
use crate::pipeline::action::multi_window::primary_windowing::MultiWindow;
use crate::pipeline::action::multi_window::secondary_app::{
    LaunchSecondaryAppPreset, LaunchSecondaryFlatpakApp,
};
use crate::pipeline::action::session_handler::DesktopSessionHandler;
use crate::pipeline::action::source_file::SourceFile;
use crate::pipeline::action::virtual_screen::VirtualScreen;
use crate::pipeline::action::{ActionImpl, ActionType};
use crate::pipeline::data::RuntimeSelection;
use crate::secondary_app::SecondaryAppManager;
use crate::settings::{AppId, GameId, GlobalConfig, SteamLaunchInfo};
use crate::sys::app_process::AppProcess;
use crate::sys::kwin::KWin;
use crate::sys::x_display::XDisplay;

use super::action::session_handler::UiEvent;
use super::action::{Action, ErasedPipelineAction};
use super::data::{ExitHooks, GamepadButton, Pipeline, PipelineTarget};

pub struct PipelineExecutor {
    game_id: Either<AppId, GameId>,
    pipeline: Option<Pipeline>,
    target: PipelineTarget,
    ctx: PipelineContext,
}

type OnLaunchCallback = Box<dyn FnOnce(Pid, &mut PipelineContext) -> Result<()>>;

pub struct PipelineContext {
    /// Decky environment variables for the session
    pub decky_env: Arc<DeckyEnv>,
    /// KWin script handler
    pub kwin: KWin,
    /// Display handler,
    pub display: Option<XDisplay>,
    pub exit_hooks: Option<ExitHooks>,
    pub secondary_app: SecondaryAppManager,
    pub launch_info: Option<SteamLaunchInfo>,
    pub global_config: GlobalConfig,
    /// actions that have run
    have_run: Vec<Action>,
    /// pipeline state
    state: TypeMap,
    on_launch_callbacks: Vec<OnLaunchCallback>,
}

// state impl

struct StateKey<S: ActionImpl>(PhantomData<S>);

impl<S> Key for StateKey<S>
where
    S: ActionImpl + 'static,
{
    type Value = Vec<Option<<S as ActionImpl>::State>>;
}

impl PipelineContext {
    pub fn new(
        launch_info: Option<SteamLaunchInfo>,
        global_config: GlobalConfig,
        decky_env: Arc<DeckyEnv>,
    ) -> Self {
        PipelineContext {
            kwin: KWin::new(decky_env.asset_manager()),
            display: XDisplay::new().ok(),
            state: TypeMap::new(),
            have_run: vec![],
            secondary_app: SecondaryAppManager::new(decky_env.asset_manager()),
            exit_hooks: Some(ExitHooks::default()),
            on_launch_callbacks: vec![],
            launch_info,
            decky_env,
            global_config,
        }
    }

    pub fn register_on_launch_callback(&mut self, callback: OnLaunchCallback) {
        self.on_launch_callbacks.push(callback);
    }

    pub fn load(global_config: GlobalConfig, decky_env: Arc<DeckyEnv>) -> Result<Option<Self>> {
        let mut default: PipelineContext = PipelineContext::new(None, global_config, decky_env);

        let persisted = std::fs::read_to_string(default.get_state_path()).ok();
        let persisted = match persisted {
            Some(p) => p,
            None => {
                return Ok(None);
            }
        };

        log::info!("Pipeline context exists; loading");

        let mut type_reg = TypeReg::new();
        fn register_type<T>(type_reg: &mut TypeReg<String>)
        where
            T: ActionImpl + Clone + Send + Sync + 'static,
            <T as ActionImpl>::State: Clone + Send + Sync,
        {
            type_reg.register::<(T, Vec<Option<<T as ActionImpl>::State>>)>(T::TYPE.to_string());
        }

        register_type::<DesktopSessionHandler>(&mut type_reg);
        register_type::<VirtualScreen>(&mut type_reg);
        register_type::<MultiWindow>(&mut type_reg);
        register_type::<SourceFile>(&mut type_reg);
        register_type::<CemuLayout>(&mut type_reg);
        register_type::<CitraLayout>(&mut type_reg);
        register_type::<MelonDSLayout>(&mut type_reg);
        register_type::<DisplayConfig>(&mut type_reg);
        register_type::<LaunchSecondaryAppPreset>(&mut type_reg);
        register_type::<LaunchSecondaryFlatpakApp>(&mut type_reg);
        register_type::<MainAppAutomaticWindowing>(&mut type_reg);
        register_type::<Lime3dsLayout>(&mut type_reg);
        register_type::<CemuAudio>(&mut type_reg);

        type_reg.register::<Vec<String>>("__actions__".to_string());
        type_reg.register::<DeckyEnv>("__env__".to_string());
        type_reg.register::<Option<SteamLaunchInfo>>("__steam_launch_info__".to_string());

        let mut deserializer = serde_json::Deserializer::from_str(&persisted);
        let type_map: SerdeMap<String> = type_reg
            .deserialize_map(&mut deserializer)
            .with_context(|| "failed to deserialize persisted context state")?;

        let actions = type_map
            .get::<Vec<String>, _>("__actions__")
            .with_context(|| "could not find key '__actions__' while loading context state")?
            .iter()
            .map(|v| v.as_str());

        let env = type_map
            .get::<DeckyEnv, _>("__env__")
            .with_context(|| "could not find key '__env__' while loading context state")?;

        let launch_info = type_map
            .get::<Option<SteamLaunchInfo>, _>("__steam_launch_info__")
            .with_context(|| {
                "could not find key '__steam_launch_info__' while loading context state"
            })?;

        for action in actions {
            match ActionType::from_str(action) {
                Ok(action) => match action {
                    ActionType::DesktopSessionHandler => {
                        load_state::<DesktopSessionHandler>(&mut default, &type_map)
                    }
                    ActionType::VirtualScreen => {
                        load_state::<VirtualScreen>(&mut default, &type_map)
                    }
                    ActionType::MultiWindow => load_state::<MultiWindow>(&mut default, &type_map),
                    ActionType::SourceFile => load_state::<SourceFile>(&mut default, &type_map),
                    ActionType::CemuLayout => load_state::<CemuLayout>(&mut default, &type_map),
                    ActionType::CitraLayout => load_state::<CitraLayout>(&mut default, &type_map),
                    ActionType::MelonDSLayout => {
                        load_state::<MelonDSLayout>(&mut default, &type_map)
                    }
                    ActionType::DisplayConfig => {
                        load_state::<DisplayConfig>(&mut default, &type_map)
                    }
                    ActionType::LaunchSecondaryFlatpakApp => {
                        load_state::<LaunchSecondaryFlatpakApp>(&mut default, &type_map)
                    }
                    ActionType::LaunchSecondaryAppPreset => {
                        load_state::<LaunchSecondaryAppPreset>(&mut default, &type_map)
                    }
                    ActionType::MainAppAutomaticWindowing => {
                        load_state::<MainAppAutomaticWindowing>(&mut default, &type_map)
                    }
                    ActionType::Lime3dsLayout => {
                        load_state::<Lime3dsLayout>(&mut default, &type_map)
                    }
                    ActionType::CemuAudio => load_state::<CemuAudio>(&mut default, &type_map),
                    ActionType::DesktopControllerLayoutHack => {
                        load_state::<DesktopControllerLayoutHack>(&mut default, &type_map)
                    }
                },
                Err(err) => {
                    log::warn!("failed to parse action {action} from type reg: {err:#?}")
                }
            }
        }

        fn load_state<T>(ctx: &mut PipelineContext, serde_map: &SerdeMap<String>)
        where
            T: ActionImpl + Clone + Send + Sync + 'static,
            <T as ActionImpl>::State: Clone + Send + Sync,
            Action: From<T>,
        {
            if let Some(value) =
                serde_map.get::<(T, Vec<Option<<T as ActionImpl>::State>>), _>(&T::TYPE.to_string())
            {
                ctx.have_run.push(value.0.clone().into());
                ctx.state.insert::<StateKey<T>>(value.1.clone());
            }
        }

        default.kwin = KWin::new(env.asset_manager());
        default.secondary_app = SecondaryAppManager::new(env.asset_manager());
        default.decky_env = Arc::new(env.clone());
        default.launch_info = launch_info.clone();

        Ok(Some(default))
    }

    fn persist(&self) -> Result<()> {
        let mut map = SerdeMap::new();

        fn insert_action<T>(ctx: &PipelineContext, map: &mut SerdeMap<String>, action: &T)
        where
            T: ActionImpl + Clone + Send + Sync + 'static,
            <T as ActionImpl>::State: Clone + Send + Sync,
        {
            // TODO::this technically saves too much if multiple actions of the same type exist
            // in the pipeline, but... eh. Its not common, and it doesn't affect functionality.
            map.insert(
                T::TYPE.to_string(),
                (action.clone(), ctx.state.get::<StateKey<T>>().cloned()),
            );
        }

        // TODO::clone less things
        for action in self.have_run.iter() {
            match action {
                Action::DesktopSessionHandler(a) => insert_action(self, &mut map, a),
                Action::DisplayConfig(a) => insert_action(self, &mut map, a),
                Action::VirtualScreen(a) => insert_action(self, &mut map, a),
                Action::MultiWindow(a) => insert_action(self, &mut map, a),
                Action::CitraLayout(a) => insert_action(self, &mut map, a),
                Action::CemuLayout(a) => insert_action(self, &mut map, a),
                Action::CemuAudio(a) => insert_action(self, &mut map, a),
                Action::MelonDSLayout(a) => insert_action(self, &mut map, a),
                Action::SourceFile(a) => insert_action(self, &mut map, a),
                Action::LaunchSecondaryFlatpakApp(a) => insert_action(self, &mut map, a),
                Action::LaunchSecondaryAppPreset(a) => insert_action(self, &mut map, a),
                Action::MainAppAutomaticWindowing(a) => insert_action(self, &mut map, a),
                Action::Lime3dsLayout(a) => insert_action(self, &mut map, a),
                Action::DesktopControllerLayoutHack(a) => insert_action(self, &mut map, a),
            };
        }

        let actions = self
            .have_run
            .iter()
            .map(|a| a.get_type())
            .collect::<Vec<_>>();
        map.insert("__actions__".to_string(), actions);
        map.insert("__env__".to_string(), (*self.decky_env).clone());
        map.insert(
            "__steam_launch_info__".to_string(),
            self.launch_info.clone(),
        );

        let serialized = serde_json::to_string_pretty(&map)?;
        let path = self.get_state_path();

        Ok(std::fs::write(path, serialized)?)
    }

    pub fn get_state_index<P: ActionImpl + 'static>(&self) -> Option<usize> {
        if !self.state.contains::<StateKey<P>>() {
            return None;
        }

        self.state
            .get::<StateKey<P>>()
            .expect("state slot should exist")
            .iter()
            .enumerate()
            .last()
            .map(|v| v.0)
    }

    pub fn get_state<P: ActionImpl + 'static>(&self) -> Option<&P::State> {
        if !self.state.contains::<StateKey<P>>() {
            return None;
        }

        self.state
            .get::<StateKey<P>>()
            .expect("state slot should exist")
            .iter()
            .last()
            .and_then(|v| v.as_ref())
    }

    pub fn get_state_mut<P: ActionImpl + 'static>(&mut self) -> Option<&mut P::State> {
        if !self.state.contains::<StateKey<P>>() {
            return None;
        }

        self.state
            .get_mut::<StateKey<P>>()
            .expect("state slot should exist")
            .iter_mut()
            .last()
            .and_then(|v| v.as_mut())
    }

    pub fn set_state<P: ActionImpl + 'static>(&mut self, state: P::State) -> Option<P::State> {
        let entry = self.state.entry::<StateKey<P>>().or_insert(vec![]);
        entry
            .last_mut()
            .expect("state slot should exist")
            .replace(state)
    }

    pub fn send_ui_event(&mut self, event: UiEvent) {
        let ui_state = self.get_state_mut::<DesktopSessionHandler>();
        if let Some(ui_state) = ui_state {
            ui_state.send_ui_event(event);
        }
    }

    pub fn teardown(mut self, errors: &mut Vec<anyhow::Error>) {
        while let Some(action) = self.have_run.pop() {
            let ctx: &mut PipelineContext = &mut self;

            let msg = format!("tearing down {}...", action.get_type());

            log::info!("{msg}");

            ctx.send_ui_event(UiEvent::UpdateStatusMsg(msg));

            let res = ctx.teardown_action(action);

            if let Err(err) = res {
                log::error!("{}", err);
                errors.push(err);
            }
        }

        let _ = self.kwin.reconfigure(); // restore kwin; since the result is ignored, the target doesn't matter
        let _ = std::fs::remove_file(self.get_state_path());
    }

    fn get_state_path(&self) -> PathBuf {
        self.decky_env.decky_plugin_runtime_dir.join("state.json")
    }

    pub fn handle_state_slot(&mut self, action: &ActionType, is_push: bool) {
        fn handle<T: ActionImpl + 'static>(this: &mut PipelineContext, is_push: bool) {
            let v = this.state.entry::<StateKey<T>>().or_insert(vec![]);
            if is_push {
                v.push(None)
            } else {
                v.pop();
            }
        }

        match action {
            ActionType::CemuLayout => handle::<CemuLayout>(self, is_push),
            ActionType::CemuAudio => handle::<CemuAudio>(self, is_push),
            ActionType::CitraLayout => handle::<CitraLayout>(self, is_push),
            ActionType::DesktopSessionHandler => handle::<DesktopSessionHandler>(self, is_push),
            ActionType::DisplayConfig => handle::<DisplayConfig>(self, is_push),
            ActionType::MultiWindow => handle::<MultiWindow>(self, is_push),
            ActionType::MainAppAutomaticWindowing => {
                handle::<MainAppAutomaticWindowing>(self, is_push)
            }
            ActionType::MelonDSLayout => handle::<MelonDSLayout>(self, is_push),
            ActionType::SourceFile => handle::<SourceFile>(self, is_push),
            ActionType::VirtualScreen => handle::<VirtualScreen>(self, is_push),
            ActionType::LaunchSecondaryFlatpakApp => {
                handle::<LaunchSecondaryFlatpakApp>(self, is_push)
            }
            ActionType::LaunchSecondaryAppPreset => {
                handle::<LaunchSecondaryAppPreset>(self, is_push)
            }
            ActionType::Lime3dsLayout => handle::<Lime3dsLayout>(self, is_push),
            ActionType::DesktopControllerLayoutHack => {
                handle::<DesktopControllerLayoutHack>(self, is_push)
            }
        }
    }

    fn setup_action(&mut self, action: Action) -> Result<()> {
        let res = action
            .exec(self, ExecActionType::Setup)
            .with_context(|| format!("failed to execute setup for {}", action.get_type()));
        self.have_run.push(action);
        self.persist().and(res)
    }

    fn teardown_action(&mut self, action: Action) -> Result<()> {
        let res = action
            .exec(self, ExecActionType::Teardown)
            .with_context(|| format!("failed to execute teardown for {}", action.get_type()));
        self.persist().and(res)
    }
}

impl PipelineExecutor {
    pub fn new(
        game_id: Either<AppId, GameId>,
        pipeline: Pipeline,
        target: PipelineTarget,
        decky_env: Arc<DeckyEnv>,
        launch_info: SteamLaunchInfo,
        global_config: GlobalConfig,
    ) -> Result<Self> {
        let s = Self {
            game_id,
            pipeline: Some(pipeline),
            target,
            ctx: PipelineContext::new(Some(launch_info), global_config, decky_env),
        };

        Ok(s)
    }

    pub fn exec(mut self, global_config: &GlobalConfig) -> Result<()> {
        // Set up pipeline

        let pipeline = {
            let p = self
                .pipeline
                .take()
                .with_context(|| "cannot execute pipeline; pipeline has already been executed")?;

            self.ctx.exit_hooks =
                if self.target == PipelineTarget::Desktop && p.should_register_exit_hooks {
                    Some(
                        p.exit_hooks_override
                            .clone()
                            .unwrap_or(global_config.exit_hooks.clone()),
                    )
                } else {
                    None
                };

            p.build_actions(self.target)
        };

        let mut errors = vec![];

        // Install dependencies
        for action in pipeline.iter() {
            self.ctx.send_ui_event(UiEvent::UpdateStatusMsg(format!(
                "checking dependencies for {}...",
                action.get_type()
            )));

            if let Err(err) = action.exec(&mut self.ctx, ExecActionType::Dependencies) {
                return Err(err).with_context(|| "Error installing dependencies");
            }
        }

        // Setup
        for action in pipeline {
            self.ctx.send_ui_event(UiEvent::UpdateStatusMsg(format!(
                "setting up {}...",
                action.get_type()
            )));

            if let Err(err) = self.ctx.setup_action(action) {
                log::error!("{:#?}", err);
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
                log::error!("{:#?}", err);
                errors.push(err);
            }
        }

        // Teardown
        self.ctx.teardown(&mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            let err = anyhow::anyhow!("Encountered errors executing pipeline: {:?}", errors);

            log::error!("{err:#?}");
            Err(err)
        }
    }

    fn run_app(&mut self) -> Result<()> {
        let (app_id, launch_type) = match (self.game_id.as_ref(), self.target) {
            (Either::Right(id), PipelineTarget::Desktop) => (id.raw(), "rungameid"),
            (Either::Right(id), _) => (id.raw(), "launch"),
            (Either::Left(id), _) => (id.raw(), "launch"),
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

        let app_process = AppProcess::find(Duration::from_secs(60))?;

        log::debug!("Got app process {:?}...", app_process.get_pid());

        let mut tmp = vec![];
        std::mem::swap(&mut tmp, &mut self.ctx.on_launch_callbacks);

        for callback in tmp.into_iter() {
            callback(app_process.get_pid(), &mut self.ctx)?;
        }

        std::mem::swap(&mut vec![], &mut self.ctx.on_launch_callbacks);

        if self.target == PipelineTarget::Desktop {
            // reconfigure kwin after actions + callbacks have executed
            self.ctx.kwin.reconfigure()?;
        }

        let mut gilrs = gilrs::Gilrs::new().unwrap();
        let mut state = IndexMap::<GamepadId, (HashSet<Button>, Option<Instant>)>::new();

        self.ctx.send_ui_event(UiEvent::ClearStatus);
        self.ctx.send_ui_event(UiEvent::UpdateWindowLevel(
            egui::WindowLevel::AlwaysOnBottom,
        ));

        log::debug!("Waiting for app process to close...");

        while app_process.is_alive() {
            std::thread::sleep(std::time::Duration::from_millis(100));

            if let Some(hooks) = self.ctx.exit_hooks.clone() {
                let btns: Vec<Button> = hooks.iter().map(|v| (*v).into()).collect();

                while let Some(Event { id, event, time }) = gilrs.next_event() {
                    fn create_instant(time: SystemTime) -> Instant {
                        let elapsed = time.elapsed().unwrap_or_default();
                        Instant::now() - elapsed
                    }
                    log::trace!("Event: {:?}", event);
                    match event {
                        EventType::ButtonPressed(btn, _) => {
                            let entry = state.entry(id).or_default();
                            if btns.contains(&btn) {
                                entry.0.insert(btn);
                            }

                            if entry.0.len() == btns.len() && entry.1.is_none() {
                                entry.1 = Some(create_instant(time))
                            }
                        }
                        EventType::ButtonReleased(btn, _) => {
                            let entry = state.entry(id).or_default();
                            if btns.contains(&btn) {
                                entry.0.remove(&btn);
                                entry.1 = None;
                            }
                        }
                        EventType::Connected => {
                            let gamepad = gilrs.gamepad(id);

                            fn check_pressed(gamepad: Gamepad, btn: Button) -> bool {
                                gamepad
                                    .button_data(btn)
                                    .map(|data| data.is_pressed())
                                    .unwrap_or_default()
                            }

                            for btn in btns.iter() {
                                let pressed = check_pressed(gamepad, *btn);
                                if pressed {
                                    let entry = state.entry(id).or_default();
                                    entry.0.insert(*btn);

                                    if entry.0.len() == btns.len() && entry.1.is_none() {
                                        entry.1 = Some(create_instant(time))
                                    }
                                }
                            }
                        }
                        EventType::Disconnected => {
                            state.swap_remove(&id);
                        }
                        _ => (),
                    }
                }

                log::trace!("Gamepad State: {state:?}");

                for (_, instant) in state.values() {
                    let hold_duration = std::time::Duration::from_secs(2);
                    if matches!(instant, &Some(i) if i.elapsed() > hold_duration) {
                        log::info!("Received exit signal. Closing application...");

                        return app_process.kill();
                    }
                }
            }
        }

        log::debug!("App process closed.");

        self.ctx.send_ui_event(UiEvent::UpdateStatusMsg(
            "returning to game mode...".to_string(),
        ));

        self.ctx
            .send_ui_event(UiEvent::UpdateWindowLevel(egui::WindowLevel::AlwaysOnTop));

        Ok(())
    }
}

enum ExecActionType {
    Dependencies,
    Setup,
    Teardown,
}

impl Pipeline {
    fn build_actions(mut self, target: PipelineTarget) -> Vec<Action> {
        fn build_recursive(selection: RuntimeSelection) -> Vec<Action> {
            match selection {
                RuntimeSelection::Action(action) => vec![action],
                RuntimeSelection::OneOf { selection, actions } => {
                    let action = actions
                        .into_iter()
                        .find(|a| a.id == selection)
                        .unwrap_or_else(|| panic!("Selection {selection:?} should exist"));

                    build_recursive(action.selection)
                }
                RuntimeSelection::AllOf(actions) => actions
                    .into_iter()
                    .filter_map(|a| match a.enabled {
                        None | Some(true) => Some(a.selection),
                        Some(false) => None,
                    })
                    .flat_map(build_recursive)
                    .collect(),
            }
        }

        self.targets
            .remove(&target)
            .into_iter()
            .flat_map(build_recursive)
            .collect()
    }
}

impl Action {
    fn exec(&self, ctx: &mut PipelineContext, action: ExecActionType) -> Result<()> {
        match action {
            ExecActionType::Dependencies => {
                let deps = self.get_dependencies(ctx);

                for d in deps {
                    d.verify_or_install(ctx)?;
                }

                Ok(())
            }
            ExecActionType::Setup => self.setup(ctx),
            ExecActionType::Teardown => self.teardown(ctx),
        }
    }
}

impl From<GamepadButton> for Button {
    fn from(value: GamepadButton) -> Self {
        match value {
            GamepadButton::Start => Button::Start,
            GamepadButton::Select => Button::Select,
            GamepadButton::North => Button::North,
            GamepadButton::East => Button::East,
            GamepadButton::South => Button::South,
            GamepadButton::West => Button::West,
            GamepadButton::RightThumb => Button::RightThumb,
            GamepadButton::LeftThumb => Button::LeftThumb,
            GamepadButton::DPadUp => Button::DPadUp,
            GamepadButton::DPadLeft => Button::DPadLeft,
            GamepadButton::DPadRight => Button::DPadRight,
            GamepadButton::DPadDown => Button::DPadDown,
            GamepadButton::L1 => Button::LeftTrigger,
            GamepadButton::L2 => Button::LeftTrigger2,
            GamepadButton::R1 => Button::RightTrigger,
            GamepadButton::R2 => Button::RightTrigger2,
            // If it can be determined what "C" and "Z" map to, they exist as well.
            // "Mode" is not used as SteamOS overrides it.
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::pipeline::action::{
        cemu_layout::CemuLayoutState,
        citra_layout::{CitraLayoutOption, CitraLayoutState, CitraState},
        melonds_layout::{MelonDSLayoutOption, MelonDSLayoutState, MelonDSSizingOption},
        multi_window::primary_windowing::{
            CemuWindowOptions, CitraWindowOptions, CustomWindowOptions, DolphinWindowOptions,
            GeneralOptions, LimitedMultiWindowLayout, MultiWindowLayout, MultiWindowOptions,
        },
        session_handler::{DisplayState, ExternalDisplaySettings, RelativeLocation},
        source_file::{FileSource, FlatpakSource},
        ActionId,
    };

    use super::*;

    #[test]
    fn test_ctx_serde() -> anyhow::Result<()> {
        // TODO::test all action types

        let decky_env = Arc::new(DeckyEnv::new_test("ctx_serde"));

        let mut ctx = PipelineContext::new(None, Default::default(), decky_env.clone());

        let actions: Vec<Action> = vec![
            DesktopSessionHandler {
                id: ActionId::nil(),
                teardown_external_settings: ExternalDisplaySettings::Native,
                teardown_deck_location: Some(RelativeLocation::Below),
                deck_is_primary_display: true,
            }
            .clone()
            .into(),
            VirtualScreen {
                id: ActionId::nil(),
            }
            .clone()
            .into(),
            MultiWindow {
                id: ActionId::nil(),
                general: GeneralOptions::default(),
                cemu: None,
                citra: None,
                dolphin: None,
                custom: None,
            }
            .into(),
            SourceFile {
                id: ActionId::nil(),
                source: FileSource::Flatpak(FlatpakSource::Cemu),
            }
            .into(),
            CemuLayout {
                id: ActionId::nil(),
                layout: CemuLayoutState {
                    separate_gamepad_view: true,
                    fullscreen: true,
                },
            }
            .into(),
            CitraLayout {
                id: ActionId::nil(),
                layout: CitraLayoutState {
                    layout_option: CitraLayoutOption::Default,
                    swap_screens: false,
                    fullscreen: true,
                    rotate_upright: false,
                },
            }
            .into(),
            MelonDSLayout {
                id: ActionId::nil(),
                layout_option: MelonDSLayoutOption::Vertical,
                sizing_option: MelonDSSizingOption::Even,
                book_mode: false,
                swap_screens: false,
            }
            .into(),
        ];

        // assert_eq!(
        //     actions
        //         .iter()
        //         .map(|v| v.get_type())
        //         .collect::<HashSet<_>>()
        //         .len(),
        //     ActionType::iter().count(),
        //     "not all actions tested"
        // );

        for a in actions.iter() {
            ctx.handle_state_slot(&a.get_type(), true);
        }

        ctx.have_run = actions;

        ctx.set_state::<DesktopSessionHandler>(DisplayState::default());
        ctx.set_state::<VirtualScreen>(false);
        ctx.set_state::<MultiWindow>(MultiWindowOptions {
            enabled: true,
            general: GeneralOptions::default(),
            cemu: CemuWindowOptions {
                single_screen_layout: LimitedMultiWindowLayout::ColumnLeft,
                multi_screen_layout: MultiWindowLayout::Separate,
            },
            citra: CitraWindowOptions {
                single_screen_layout: LimitedMultiWindowLayout::ColumnRight,
                multi_screen_layout: MultiWindowLayout::Separate,
            },
            dolphin: DolphinWindowOptions {
                single_screen_layout: LimitedMultiWindowLayout::SquareLeft,
                multi_screen_single_secondary_layout: MultiWindowLayout::SquareRight,
                multi_screen_multi_secondary_layout: MultiWindowLayout::Separate,
                gba_blacklist: vec![1, 2, 3, 4],
            },
            custom: CustomWindowOptions::default(),
        });
        ctx.set_state::<SourceFile>("some_random_path".into());
        ctx.set_state::<CemuLayout>(CemuLayoutState {
            separate_gamepad_view: true,
            fullscreen: false,
        });
        ctx.set_state::<CitraLayout>(CitraState::default());
        ctx.set_state::<MelonDSLayout>(MelonDSLayoutState::default());

        ctx.persist()?;

        let loaded = PipelineContext::load(Default::default(), decky_env.clone())
            .with_context(|| "Persisted context should load")?
            .with_context(|| "Persisted context should exist")?;

        for (expected_action, actual_action) in ctx.have_run.iter().zip(loaded.have_run.iter()) {
            assert_eq!(expected_action, actual_action);
        }

        fn check_state<T>(ctx: &PipelineContext, loaded: &PipelineContext)
        where
            T: ActionImpl + 'static,
            <T as ActionImpl>::State: PartialEq,
        {
            let expected = ctx.get_state::<T>();
            let actual = loaded.get_state::<T>();

            assert_eq!(expected, actual, "{} failed to match", T::TYPE);
        }

        check_state::<DesktopSessionHandler>(&ctx, &loaded);
        // check_state::<DisplayConfig>(&ctx, &loaded);
        check_state::<VirtualScreen>(&ctx, &loaded);
        check_state::<MultiWindow>(&ctx, &loaded);
        check_state::<SourceFile>(&ctx, &loaded);
        check_state::<CemuLayout>(&ctx, &loaded);
        check_state::<CitraLayout>(&ctx, &loaded);
        check_state::<MelonDSLayout>(&ctx, &loaded);

        Ok(())
    }
}
