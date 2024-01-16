use std::{
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use anyhow::{Context, Ok, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use xrandr::{Relation, XId};

use crate::{pipeline::executor::PipelineContext, sys::x_display::ModePreference};

use self::ui::DeckDsUi;

use super::{ActionId, ActionImpl};

mod ui;

pub use ui::Pos;
pub use ui::Size;
pub use ui::UiEvent;

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DisplayRestoration {
    pub id: ActionId,

    pub teardown_external_settings: TeardownExternalSettings,
    pub teardown_deck_location: RelativeLocation,
}
impl DisplayRestoration {
    pub(crate) fn desktop_only(&self, ctx: &mut PipelineContext<'_>) -> Result<()> {
        let mut display = ctx
            .display
            .take()
            .with_context(|| "DisplayRestoration requires x11 to be running")?;
        if let Some(current_output) = display.get_preferred_external_output()? {
            match self.teardown_external_settings {
                TeardownExternalSettings::Previous => Ok(()),
                TeardownExternalSettings::Native => {
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
                TeardownExternalSettings::Preference(preference) => {
                    display.set_or_create_preferred_mode(&current_output, &preference)
                }
            }?;

            let deck = display.get_embedded_output()?.unwrap();
            display.set_output_position(&deck, &self.teardown_deck_location.into(), &current_output)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub struct DisplayState {
    previous_output_id: XId,
    previous_output_mode: Option<XId>,
    runtime_state: Option<RuntimeDisplayState>,
}

impl From<&DisplayState> for SerialiableDisplayState {
    fn from(value: &DisplayState) -> Self {
        Self {
            previous_output_id: value.previous_output_id,
            previous_output_mode: value.previous_output_mode,
        }
    }
}

#[derive(Debug)]
struct RuntimeDisplayState {
    ui_tx: Sender<UiEvent>,
    ui_ctx: egui::Context,
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
            previous_output_id: self.previous_output_id,
            previous_output_mode: self.previous_output_mode,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DisplayState {
    fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        SerialiableDisplayState::deserialize(deserializer).map(|de| DisplayState {
            previous_output_id: de.previous_output_id,
            previous_output_mode: de.previous_output_mode,
            runtime_state: None,
        })
    }
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, JsonSchema)]
pub enum RelativeLocation {
    Above,
    #[default]
    Below,
    LeftOf,
    RightOf,
    SameAs,
}

impl From<RelativeLocation> for Relation {
    fn from(value: RelativeLocation) -> Self {
        match value {
            RelativeLocation::Above => Relation::Above,
            RelativeLocation::Below => Relation::Below,
            RelativeLocation::LeftOf => Relation::LeftOf,
            RelativeLocation::RightOf => Relation::RightOf,
            RelativeLocation::SameAs => Relation::SameAs,
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", content = "value")]
pub enum TeardownExternalSettings {
    /// Previous resolution, before setup
    #[default]
    Previous,
    /// Native resolution
    Native,
    /// Resolution based on specific settings
    Preference(ModePreference),
}

impl DisplayState {
    pub fn send_ui_event(&self, event: UiEvent) {
        if let Some(state) = self.runtime_state.as_ref() {
            let _ = state.ui_tx.send(event);
            state.ui_ctx.request_repaint();
        }
    }
}

impl ActionImpl for DisplayRestoration {
    type State = DisplayState;

    fn setup(&self, ctx: &mut PipelineContext) -> Result<()> {
        let display = ctx
            .display
            .as_mut()
            .with_context(|| "DisplayRestoration requires x11 to be running")?;

        let preferred = display.get_preferred_external_output()?;

        match preferred {
            Some(primary) => {
                let (ui_tx, ui_rx): (Sender<UiEvent>, Receiver<UiEvent>) = mpsc::channel();
                let (main_tx, main_rx): (Sender<egui::Context>, Receiver<egui::Context>) =
                    mpsc::channel();

                let main_tx = main_tx.clone();

                std::thread::spawn(move || {
                    // TODO::caluculate current sizes + positions; mostly don't care as it will be immediately reset
                    DeckDsUi::new(
                        Size(1920, 1080),
                        Size(1280, 800),
                        Pos(0, 0),
                        Pos(0, 1920),
                        ui_rx,
                        main_tx,
                    )
                    .run()
                    .map_err(|err| format!("{err:?}"))
                });

                let ui_ctx = main_rx.recv().expect("UI thread should send ctx");

                ctx.set_state::<Self>(DisplayState {
                    previous_output_id: primary.xid,
                    previous_output_mode: primary.current_mode,
                    runtime_state: Some(RuntimeDisplayState { ui_ctx, ui_tx }),
                });
                Ok(())
            }
            None => Err(anyhow::anyhow!(
                "Unable to find external display for dual screen"
            )),
        }
    }

    fn teardown(&self, ctx: &mut PipelineContext) -> Result<()> {
        let mut display = ctx
            .display
            .take()
            .with_context(|| "DisplayRestoration requires x11 to be running")?;
        let current_output = display.get_preferred_external_output()?;

        let res = match ctx.get_state::<Self>() {
            Some(state) => {
                if let Some(runtime) = state.runtime_state.as_ref() {
                    runtime.ui_ctx.request_repaint_after(Duration::from_secs(1))
                }

                // let _ = state.ui_tx.send(UiEvent::Close);

                let output = state.previous_output_id;

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
                    TeardownExternalSettings::Previous => match state.previous_output_mode {
                        Some(mode) => {
                            let mode = display.get_mode(mode)?;
                            display.set_output_mode(&current_output, &mode)
                        }
                        None => DisplayRestoration {
                            teardown_external_settings: TeardownExternalSettings::Native,
                            ..*self
                        }
                        .teardown(ctx),
                    },
                    TeardownExternalSettings::Native => {
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
                    TeardownExternalSettings::Preference(preference) => {
                        display.set_or_create_preferred_mode(&current_output, &preference)
                    }
                }?;

                let deck = display.get_embedded_output()?.unwrap();
                display.set_output_position(
                    &deck,
                    &self.teardown_deck_location.into(),
                    &current_output,
                )
            }

            // No state, nothing to tear down
            None => Ok(()),
        };

        ctx.display = Some(display);

        res
    }

    fn get_id(&self) -> ActionId {
        self.id
    }
}

#[cfg(test)]
mod tests {}
