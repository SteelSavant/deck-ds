use std::{
    os::unix::thread,
    sync::mpsc::{self, Receiver, Sender},
    thread::sleep,
    time::Duration,
};

use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use xrandr::XId;

use crate::pipeline::{action::ActionType, dependency::Dependency, executor::PipelineContext};

use self::ui::DeckDsUi;

use super::super::{ActionId, ActionImpl};
use smart_default::SmartDefault;

pub use super::common::{ExternalDisplaySettings, RelativeLocation};

mod ui;

pub use ui::Pos;
pub use ui::Size;
pub use ui::UiEvent;

#[derive(Debug, Copy, Clone, SmartDefault, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DesktopSessionHandler {
    pub id: ActionId,
    #[default(true)]
    pub deck_is_primary_display: bool,
    pub teardown_external_settings: ExternalDisplaySettings,
    #[default(Some(Default::default()))]
    pub teardown_deck_location: Option<RelativeLocation>,
}

impl DesktopSessionHandler {
    pub(crate) fn desktop_only(&self, ctx: &mut PipelineContext<'_>) -> Result<()> {
        let mut display = ctx
            .display
            .take()
            .with_context(|| "DesktopSessionHandler requires x11 to be running")?;

        let mut deck = display
            .get_embedded_output()?
            .with_context(|| "unable to find embedded display")?;
        let current_output = display.get_preferred_external_output()?;

        if let Some(current_output) = current_output.as_ref() {
            match self.teardown_external_settings {
                ExternalDisplaySettings::Previous => Ok(()),
                ExternalDisplaySettings::Native => {
                    let native_mode = display.get_native_mode(current_output)?;
                    if let Some(mode) = native_mode {
                        display.set_output_mode(current_output, &mode)
                    } else {
                        Ok(())
                    }
                }
                ExternalDisplaySettings::Preference(preference) => {
                    display.set_or_create_preferred_mode(current_output, &preference)
                }
            }?;
        }

        if let Some(location) = self.teardown_deck_location {
            display.reconfigure_embedded(
                &mut deck,
                &location.into(),
                current_output.as_ref(),
                self.deck_is_primary_display,
            )?;
        } else {
            display.set_output_enabled(&mut deck, false)?;
        }

        Ok(())
    }
}

#[cfg_attr(test, derive(Default))]
#[derive(Debug, PartialEq)]
pub struct DisplayState {
    previous_external_output_id: XId,
    previous_external_output_mode: Option<XId>,
    runtime_state: Option<RuntimeDisplayState>,
}

impl Clone for DisplayState {
    /// Clone for DisplayState is only implemented for ease of the current serialization impl.
    /// The clone will not contain any of the UI runtime context information.

    fn clone(&self) -> Self {
        SerialiableDisplayState::from(self).into()
    }
}

impl From<&DisplayState> for SerialiableDisplayState {
    fn from(value: &DisplayState) -> Self {
        Self {
            previous_output_id: value.previous_external_output_id,
            previous_output_mode: value.previous_external_output_mode,
        }
    }
}

impl From<SerialiableDisplayState> for DisplayState {
    fn from(value: SerialiableDisplayState) -> Self {
        DisplayState {
            previous_external_output_id: value.previous_output_id,
            previous_external_output_mode: value.previous_output_mode,
            runtime_state: None,
        }
    }
}

#[derive(Debug)]
struct RuntimeDisplayState {
    ui_tx: Sender<UiEvent>,
    ui_ctx: egui::Context,
}

impl PartialEq for RuntimeDisplayState {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SerialiableDisplayState {
    previous_output_id: XId,
    previous_output_mode: Option<XId>,
}

impl Serialize for DisplayState {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerialiableDisplayState {
            previous_output_id: self.previous_external_output_id,
            previous_output_mode: self.previous_external_output_mode,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DisplayState {
    fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        SerialiableDisplayState::deserialize(deserializer).map(|de| de.into())
    }
}

impl DisplayState {
    pub fn send_ui_event(&mut self, event: UiEvent) {
        let is_close = matches!(event, UiEvent::Close);

        if let Some(state) = self.runtime_state.as_ref() {
            let _ = state.ui_tx.send(event);
            state.ui_ctx.request_repaint();
        }

        if is_close {
            self.runtime_state = None
        }
    }
}

impl ActionImpl for DesktopSessionHandler {
    type State = DisplayState;

    const TYPE: ActionType = ActionType::DesktopSessionHandler;

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        let display = ctx
            .display
            .as_mut()
            .with_context(|| "DesktopSessionHandler requires x11 to be running")?;

        let preferred = display.get_preferred_external_output()?;
        let embedded = display.get_embedded_output()?;

        log::debug!(
            "session handler found outputs: {:?}, {:?}",
            embedded,
            preferred
        );

        let (ui_tx, ui_rx): (Sender<UiEvent>, Receiver<UiEvent>) = mpsc::channel();
        let (main_tx, main_rx): (Sender<egui::Context>, Receiver<egui::Context>) = mpsc::channel();

        let should_register_exit_hooks = ctx.should_register_exit_hooks;
        let secondary_text = if should_register_exit_hooks {
                    "hold (select) + (start) to exit\ngame after launch"
                } else {
                    "exit hooks not registered;\nuse Steam Input mapping or press (Alt+F4) to exit\ngame after launch"
                }
                .to_string();

        let update = display.calc_ui_viewport_event(embedded.as_ref(), preferred.as_ref());

        if let UiEvent::UpdateViewports {
            primary_size,
            secondary_size,
            primary_position,
            secondary_position,
        } = update
        {
            log::debug!("session handler starting UI");
            std::thread::spawn(move || {
                DeckDsUi::new(
                    primary_size,
                    secondary_size,
                    primary_position,
                    secondary_position,
                    secondary_text,
                    ui_rx,
                    main_tx,
                )
                .run()
                .map_err(|err| log::error!("{err:?}"))
            });
        }

        log::debug!("session handler waiting for UI ctx");

        let ui_ctx = main_rx.recv().expect("UI thread should send ctx");

        if let Some(primary) = preferred.as_ref() {
            ctx.set_state::<Self>(DisplayState {
                previous_external_output_id: primary.xid,
                previous_external_output_mode: primary.current_mode,
                runtime_state: Some(RuntimeDisplayState { ui_ctx, ui_tx }),
            });
        }

        sleep(Duration::from_millis(500));

        log::debug!("session handler setup complete");

        Ok(())
    }

    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()> {
        let mut display = ctx
            .display
            .take()
            .with_context(|| "DesktopSessionHandler requires x11 to be running")?;
        let current_output = display.get_preferred_external_output()?;

        let res = match ctx.get_state::<Self>() {
            Some(state) => {
                if let Some(runtime) = state.runtime_state.as_ref() {
                    runtime.ui_ctx.request_repaint_after(Duration::from_secs(1))
                }

                let output = state.previous_external_output_id;

                // Gets the current output. If it matches the saved, return it,
                // otherwise exit teardown to avoid changing current monitor to
                // old monitor settings.
                let current_output = match current_output {
                    Some(current) => {
                        if current.xid == output {
                            current
                        } else {
                            return Ok(());
                        }
                    }
                    None => return Ok(()),
                };

                match self.teardown_external_settings {
                    ExternalDisplaySettings::Previous => {
                        match state.previous_external_output_mode {
                            Some(mode) => {
                                let mode = display.get_mode(mode)?;
                                display.set_output_mode(&current_output, &mode)
                            }
                            None => DesktopSessionHandler {
                                teardown_external_settings: ExternalDisplaySettings::Native,
                                ..*self
                            }
                            .teardown(ctx),
                        }
                    }
                    ExternalDisplaySettings::Native => {
                        let mode = current_output
                            .preferred_modes
                            .iter()
                            .map(|mode| display.get_mode(*mode))
                            .collect::<Result<Vec<_>, _>>()?;
                        let native_mode = mode.iter().reduce(|acc, e| {
                            match (acc.width * acc.height).cmp(&(e.width * e.height)) {
                                std::cmp::Ordering::Less => e,
                                std::cmp::Ordering::Greater => acc,
                                std::cmp::Ordering::Equal => {
                                    if acc.rate > e.rate {
                                        acc
                                    } else {
                                        e
                                    }
                                }
                            }
                        });
                        if let Some(mode) = native_mode {
                            display.set_output_mode(&current_output, mode)
                        } else {
                            Ok(())
                        }
                    }
                    ExternalDisplaySettings::Preference(preference) => {
                        display.set_or_create_preferred_mode(&current_output, &preference)
                    }
                }?;

                let mut deck = display.get_embedded_output()?.unwrap();

                if let Some(location) = self.teardown_deck_location {
                    display.reconfigure_embedded(
                        &mut deck,
                        &location.into(),
                        Some(&current_output),
                        self.deck_is_primary_display,
                    )?;
                } else {
                    display.set_output_enabled(&mut deck, false)?;
                }

                let update = display.calc_ui_viewport_event(Some(&deck), Some(&current_output));
                ctx.send_ui_event(update);

                Ok(())
            }

            // No state, nothing to tear down
            None => Ok(()),
        };

        ctx.display = Some(display);

        ctx.send_ui_event(UiEvent::Close);

        res
    }

    fn get_dependencies(
        &self,
        _ctx: &PipelineContext,
    ) -> Vec<crate::pipeline::dependency::Dependency> {
        vec![Dependency::Display]
    }

    #[inline]
    fn get_id(&self) -> ActionId {
        self.id
    }
}
