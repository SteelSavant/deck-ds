use anyhow::{anyhow, Context, Result};
use gilrs::{Button, Event, EventType, Gamepad, GamepadId};
use indexmap::IndexMap;
use type_reg::untagged::{TypeMap as SerdeMap, TypeReg};

use std::marker::PhantomData;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant, SystemTime};
use typemap::{Key, TypeMap};

use crate::asset::AssetManager;
use crate::pipeline::action::cemu_layout::CemuLayout;
use crate::pipeline::action::citra_layout::CitraLayout;
use crate::pipeline::action::melonds_layout::MelonDSLayout;
use crate::pipeline::action::multi_window::MultiWindow;
use crate::pipeline::action::source_file::SourceFile;
use crate::pipeline::action::ui_management::UIManagement;
use crate::pipeline::action::virtual_screen::VirtualScreen;
use crate::pipeline::data::{PipelineAction, Selection};
use crate::settings::GameId;
use crate::sys::app_process::AppProcess;
use crate::sys::kwin::KWin;
use crate::sys::x_display::XDisplay;

use super::action::ui_management::UiEvent;
use super::action::{Action, ErasedPipelineAction};
use super::data::{Pipeline, PipelineTarget};

use super::action::ActionImpl;

pub struct PipelineExecutor<'a> {
    game_id: GameId,
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

    pub fn load(
        assets_manager: AssetManager<'a>,
        home_dir: PathBuf,
        config_dir: PathBuf,
    ) -> Option<Self> {
        let mut default: PipelineContext<'_> =
            PipelineContext::new(assets_manager, home_dir, config_dir);

        let persisted = std::fs::read_to_string(default.get_state_path()).ok()?;

        log::info!("Pipeline context exists; loading");

        let mut type_reg = TypeReg::new();
        fn register_type<T>(type_reg: &mut TypeReg<String>)
        where
            T: ActionImpl + Clone + Send + Sync + 'static,
            <T as ActionImpl>::State: Clone + Send + Sync,
        {
            type_reg.register::<(T, Option<<T as ActionImpl>::State>)>(T::NAME.to_string());
        }

        type_reg.register::<Vec<String>>("__actions__".to_string());

        register_type::<UIManagement>(&mut type_reg);
        register_type::<VirtualScreen>(&mut type_reg);
        register_type::<MultiWindow>(&mut type_reg);
        register_type::<SourceFile>(&mut type_reg);
        register_type::<CemuLayout>(&mut type_reg);
        register_type::<CitraLayout>(&mut type_reg);
        register_type::<MelonDSLayout>(&mut type_reg);

        let mut deserializer = serde_json::Deserializer::from_str(&persisted);
        let type_map: SerdeMap<String> = type_reg.deserialize_map(&mut deserializer).unwrap();

        let actions = type_map
            .get::<Vec<String>, _>("__actions__")?
            .iter()
            .map(|v| v.as_str());

        for action in actions {
            match action {
                UIManagement::NAME => load_state::<UIManagement>(&mut default, &type_map),
                VirtualScreen::NAME => load_state::<VirtualScreen>(&mut default, &type_map),
                MultiWindow::NAME => load_state::<MultiWindow>(&mut default, &type_map),
                SourceFile::NAME => load_state::<SourceFile>(&mut default, &type_map),
                CemuLayout::NAME => load_state::<CemuLayout>(&mut default, &type_map),
                CitraLayout::NAME => load_state::<CitraLayout>(&mut default, &type_map),
                MelonDSLayout::NAME => load_state::<MelonDSLayout>(&mut default, &type_map),
                _ => {}
            }
        }

        fn load_state<T>(ctx: &mut PipelineContext, serde_map: &SerdeMap<String>)
        where
            T: ActionImpl + Clone + Send + Sync + 'static,
            <T as ActionImpl>::State: Clone + Send + Sync,
            Action: From<T>,
        {
            if let Some(value) = serde_map.get::<(T, Option<<T as ActionImpl>::State>), _>(T::NAME)
            {
                ctx.have_run.push(value.0.clone().into());
                if let Some(state) = value.1.as_ref() {
                    ctx.set_state::<T>(state.clone());
                }
            }
        }

        Some(default)
    }

    fn persist(&self) -> Result<()> {
        let mut map = SerdeMap::new();

        fn insert_action<T>(ctx: &PipelineContext, map: &mut SerdeMap<String>, action: &T)
        where
            T: ActionImpl + Clone + Send + Sync + 'static,
            <T as ActionImpl>::State: Clone + Send + Sync,
        {
            map.insert(
                T::NAME.to_string(),
                (action.clone(), ctx.get_state::<T>().cloned()),
            );
        }

        // TODO::clone less things
        for action in self.have_run.iter() {
            match action {
                Action::UIManagement(a) => insert_action(self, &mut map, a),
                Action::VirtualScreen(a) => insert_action(self, &mut map, a),
                Action::MultiWindow(a) => insert_action(self, &mut map, a),
                Action::CitraLayout(a) => insert_action(self, &mut map, a),
                Action::CemuLayout(a) => insert_action(self, &mut map, a),
                Action::MelonDSLayout(a) => insert_action(self, &mut map, a),
                Action::SourceFile(a) => insert_action(self, &mut map, a),
            };
        }

        let actions = self
            .have_run
            .iter()
            .map(|a| a.get_name())
            .collect::<Vec<_>>();
        map.insert("__actions__".to_string(), actions);

        let serialized = serde_json::to_string_pretty(&map)?;
        let path = self.get_state_path();

        Ok(std::fs::write(path, serialized)?)
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
        let ui_state = self.get_state::<UIManagement>();
        if let Some(ui_state) = ui_state {
            ui_state.send_ui_event(event);
        }
    }

    pub fn teardown(mut self, errors: &mut Vec<anyhow::Error>) {
        while let Some(action) = self.have_run.pop() {
            let ctx: &mut PipelineContext<'_> = &mut self;

            let msg = format!("tearing down {}...", action.get_name());

            log::info!("{msg}");

            ctx.send_ui_event(UiEvent::UpdateStatusMsg(msg));

            let res = ctx.teardown_action(action);

            if let Err(err) = res {
                log::error!("{}", err);
                errors.push(err);
            }
        }

        let _ = std::fs::remove_file(self.get_state_path());
    }

    fn get_state_path(&self) -> PathBuf {
        self.config_dir.join("state.json")
    }

    fn setup_action(&mut self, action: Action) -> Result<()> {
        let res = action
            .exec(self, ActionType::Setup)
            .with_context(|| format!("failed to execute setup for {}", action.get_name()));
        self.have_run.push(action);
        self.persist().and(res)
    }

    fn teardown_action(&mut self, action: Action) -> Result<()> {
        let res = action
            .exec(self, ActionType::Teardown)
            .with_context(|| format!("failed to execute teardown for {}", action.get_name()));
        self.persist().and(res)
    }
}

impl<'a> PipelineExecutor<'a> {
    pub fn new(
        game_id: GameId,
        pipeline: Pipeline,
        target: PipelineTarget,
        assets_manager: AssetManager<'a>,
        home_dir: PathBuf,
        config_dir: PathBuf,
    ) -> Result<Self> {
        let s = Self {
            game_id,
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
                action.get_name()
            )));

            if let Err(err) = action.exec(&mut self.ctx, ActionType::Dependencies) {
                return Err(err).with_context(|| "Error installing dependencies");
            }
        }

        // Setup
        for action in pipeline {
            self.ctx.send_ui_event(UiEvent::UpdateStatusMsg(format!(
                "setting up {}...",
                action.get_name()
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
        self.ctx.teardown(&mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            let err = anyhow::anyhow!("Encountered errors executing pipeline: {:?}", errors);

            log::error!("{err}");
            Err(err)
        }
    }

    fn run_app(&self) -> Result<()> {
        let app_id = self.game_id.raw();
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

#[cfg(test)]
mod tests {

    use crate::{
        pipeline::action::{
            cemu_layout::CemuLayoutState,
            citra_layout::{CitraLayoutOption, CitraLayoutState, CitraState},
            melonds_layout::{MelonDSLayoutOption, MelonDSLayoutState, MelonDSSizingOption},
            source_file::{FileSource, FlatpakSource},
            ui_management::{DisplayState, RelativeLocation, TeardownExternalSettings},
            ActionId,
        },
        util::create_dir_all,
        ASSETS_DIR,
    };

    use super::*;

    #[test]
    fn test_ctx_serde() -> anyhow::Result<()> {
        let home_dir: PathBuf = "test/out/home/ctx-serde".into();
        let config_dir: PathBuf = "test/out/.config/deck-ds-ctx-serde".into();
        let external_asset_path: PathBuf = "test/out/assets/ctx-serde".into();

        if !config_dir.exists() {
            create_dir_all(&config_dir)?;
        }

        let mut ctx = PipelineContext::new(
            AssetManager::new(&ASSETS_DIR, external_asset_path.clone()),
            home_dir.clone(),
            config_dir.clone(),
        );

        ctx.have_run = vec![
            UIManagement {
                id: ActionId::nil(),
                teardown_external_settings: TeardownExternalSettings::Native,
                teardown_deck_location: RelativeLocation::Below,
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

        ctx.set_state::<UIManagement>(DisplayState::default());
        // ctx.set_state::<VirtualScreen>(());
        // ctx.set_state::<MultiWindow>(());
        ctx.set_state::<SourceFile>("some_random_path".into());
        ctx.set_state::<CemuLayout>(CemuLayoutState {
            separate_gamepad_view: true,
            fullscreen: false,
        });
        ctx.set_state::<CitraLayout>(CitraState::default());
        ctx.set_state::<MelonDSLayout>(MelonDSLayoutState::default());

        ctx.persist()?;

        let loaded = PipelineContext::load(
            AssetManager::new(&ASSETS_DIR, external_asset_path.clone()),
            home_dir,
            config_dir,
        )
        .with_context(|| "Persisted context should load")?;

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

            assert_eq!(expected, actual, "{} failed to match", T::NAME);
        }

        check_state::<UIManagement>(&ctx, &loaded);
        check_state::<VirtualScreen>(&ctx, &loaded);
        check_state::<MultiWindow>(&ctx, &loaded);
        check_state::<SourceFile>(&ctx, &loaded);
        check_state::<CemuLayout>(&ctx, &loaded);
        check_state::<CitraLayout>(&ctx, &loaded);
        check_state::<MelonDSLayout>(&ctx, &loaded);

        Ok(())
    }
}
