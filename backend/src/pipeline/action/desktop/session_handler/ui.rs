// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::sync::mpsc::{Receiver, Sender};

use eframe::egui;
use egui::{
    epaint, Color32, Frame, Label, Pos2, RichText, Style, Ui, Vec2, ViewportBuilder,
    ViewportCommand, WindowLevel,
};
use serde::Deserialize;
use winit::platform::x11::EventLoopBuilderExtX11;

use crate::sys::kwin::screen_tracking::KWinScreenTrackingScope;

pub enum UiEvent {
    UpdateViewports {
        primary_size: Size,
        secondary_size: Option<Size>,
        primary_position: Pos,
        secondary_position: Option<Pos>,
    },
    UpdateWindowLevel(WindowLevel),
    UpdateStatusMsg(String),
    ClearStatus,
    Close,
}

pub struct DeckDsUi {
    primary_size: Size,
    secondary_size: Option<Size>,
    primary_position: Pos,
    secondary_position: Option<Pos>,
    primary_text: String,
    secondary_text: String,
    custom_frame: Frame,
    ui_tx: Sender<UiEvent>,
    ui_rx: Receiver<UiEvent>,
    tx: Sender<egui::Context>,
    window_level: WindowLevel,
    screen_state_ctx: Option<KWinScreenTrackingScope>,
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub struct Pos {
    pub x: u32,
    pub y: u32,
}

impl Pos {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

impl From<Pos> for Pos2 {
    fn from(value: Pos) -> Self {
        Pos2::new(value.x as f32, value.y as f32)
    }
}

impl From<Size> for Vec2 {
    fn from(value: Size) -> Self {
        [value.w as f32, value.h as f32].into()
    }
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub struct Size {
    pub w: u32,
    pub h: u32,
}

impl Size {
    pub fn new(w: u32, h: u32) -> Self {
        Self { w, h }
    }
    pub fn normalized(self) -> Self {
        if self.w < self.h {
            Size::new(self.h, self.w)
        } else {
            self
        }
    }
}

impl DeckDsUi {
    pub fn new(
        primary_size: Size,
        secondary_size: Option<Size>,
        primary_position: Pos,
        secondary_position: Option<Pos>,
        secondary_text: String,
        ui_tx: Sender<UiEvent>,
        ui_rx: Receiver<UiEvent>,
        tx: Sender<egui::Context>,
    ) -> Self {
        let custom_frame = egui::containers::Frame {
            inner_margin: epaint::Margin {
                left: 10.,
                right: 10.,
                top: 10.,
                bottom: 10.,
            },
            outer_margin: epaint::Margin::ZERO,
            rounding: egui::Rounding {
                nw: 1.0,
                ne: 1.0,
                sw: 1.0,
                se: 1.0,
            },
            shadow: eframe::epaint::Shadow {
                offset: Vec2 { x: 1.0, y: 1.0 },
                blur: 0.5,
                spread: 0.5,
                color: Color32::BLACK,
            },
            fill: Color32::BLACK,
            stroke: egui::Stroke::new(2.0, Color32::BLACK),
        };

        log::debug!(
            "initial viewport pos: {:?}, size: {:?}",
            primary_position,
            primary_size
        );

        Self {
            primary_size,
            secondary_size,
            primary_position,
            secondary_position,
            primary_text: "starting up...".to_string(),
            secondary_text,
            custom_frame,
            ui_tx,
            ui_rx,
            tx,
            window_level: WindowLevel::AlwaysOnTop,
            screen_state_ctx: None,
        }
    }

    pub fn run(mut self) -> Result<(), eframe::Error> {
        let options = eframe::NativeOptions {
            hardware_acceleration: eframe::HardwareAcceleration::Preferred,
            viewport: build_viewport(self.primary_position, self.primary_size, self.window_level),
            event_loop_builder: Some(Box::new(|builder| {
                builder.with_any_thread(true);
            })),
            ..Default::default()
        };

        eframe::run_native(
            "DeckDS",
            options,
            Box::new(|cc| {
                let egui_ctx = cc.egui_ctx.clone();
                let ui_tx = self.ui_tx.clone();
                let screen_state_ctx =
                    KWinScreenTrackingScope::new(Box::new(move |mut screens| {
                        if screens.is_empty() {
                            return;
                        }
                        screens.sort_by(|a, b| (a.size.w * a.size.h).cmp(&(b.size.w * b.size.h)));
                        let primary = screens.last().unwrap();
                        let secondary = if screens.len() > 1 {
                            screens.first()
                        } else {
                            None
                        };

                        log::debug!("Using screens {:?}:{:?}", primary, secondary);

                        let event = UiEvent::UpdateViewports {
                            primary_size: primary.size,
                            secondary_size: secondary.map(|v| v.size),
                            primary_position: primary.pos,
                            secondary_position: secondary.map(|v| v.pos),
                        };

                        let _ = ui_tx.send(event);
                        egui_ctx.request_repaint();
                    }))
                    .expect("kwin screen tracking scope should be constructible");

                self.screen_state_ctx = Some(screen_state_ctx);
                self.tx
                    .send(cc.egui_ctx.clone())
                    .expect("send egui context should succeed");
                Ok(Box::new(self))
            }),
        )
    }
}

impl eframe::App for DeckDsUi {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(event) = self.ui_rx.try_recv() {
            match event {
                UiEvent::UpdateViewports {
                    primary_size,
                    secondary_size,
                    primary_position,
                    secondary_position,
                } => {
                    self.primary_size = primary_size;
                    self.primary_position = primary_position;
                    self.secondary_position = secondary_position;
                    self.secondary_size = secondary_size;

                    log::debug!("setting viewports to Primary: {primary_size:?}, {primary_position:?} -- Secondary: {secondary_size:?}, {secondary_position:?}");

                    ctx.send_viewport_cmd(ViewportCommand::OuterPosition(primary_position.into()));
                    ctx.send_viewport_cmd(ViewportCommand::InnerSize(primary_size.into()))
                }
                UiEvent::UpdateStatusMsg(msg) => self.primary_text = msg,
                UiEvent::UpdateWindowLevel(window_level) => {
                    self.window_level = window_level;
                    ctx.send_viewport_cmd(ViewportCommand::WindowLevel(window_level))
                }
                UiEvent::ClearStatus => {
                    self.primary_text = "".to_string();
                    self.secondary_text = "".to_string();
                }
                UiEvent::Close => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            }
        }

        egui::CentralPanel::default()
            .frame(self.custom_frame)
            .show(ctx, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.visuals_mut().override_text_color = Some(Color32::WHITE);

                    create_deckds_label(
                        ui,
                        RichText::new(&self.primary_text),
                        ui.available_height(),
                    );
                })
            });
        if let (Some(pos), Some(size)) = (self.secondary_position, self.secondary_size) {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("DeckDS Secondary"),
                build_viewport(pos, size, self.window_level),
                |ctx, class| {
                    assert!(
                        class == egui::ViewportClass::Immediate,
                        "This egui backend doesn't support multiple viewports"
                    );

                    egui::CentralPanel::default()
                        .frame(self.custom_frame)
                        .show(ctx, |ui| {
                            ui.visuals_mut().override_text_color = Some(Color32::WHITE);

                            ui.centered_and_justified(|ui| {
                                create_deckds_label(
                                    ui,
                                    RichText::new(&self.secondary_text),
                                    ui.available_height(),
                                );
                            });
                        });
                },
            );
        }
    }
}

fn build_viewport(position: Pos, size: Size, window_level: WindowLevel) -> ViewportBuilder {
    // can't use fullscreen because
    // - the chosen monitor isn't selectable
    // - the virtual display for melonDS is only one screen,
    //   which prevents both from displaying properly
    egui::ViewportBuilder::default()
        .with_decorations(false)
        .with_active(false)
        .with_position(Pos2::from(position))
        .with_window_level(window_level)
        .with_inner_size(Vec2::from(size))
        .with_mouse_passthrough(true)

    // TODO::icon
}

fn create_deckds_label(ui: &mut Ui, subtext: RichText, viewport_height: f32) {
    use egui::{text::LayoutJob, Align, FontSelection};

    let style = Style::default();
    let mut layout_job = LayoutJob::default();

    let logo_size: f32 = viewport_height / 8.;
    let size: f32 = logo_size / 3.;
    RichText::new("Deck")
        .color(ui.visuals().text_color())
        .size(logo_size)
        .append_to(
            &mut layout_job,
            &style,
            FontSelection::Default,
            Align::Center,
        );
    RichText::new("DS")
        .color(ui.visuals().text_color())
        .size(logo_size)
        .strong()
        .append_to(
            &mut layout_job,
            &style,
            FontSelection::Default,
            Align::Center,
        );
    RichText::new("\n\n").append_to(
        &mut layout_job,
        &style,
        FontSelection::Default,
        Align::Center,
    );

    subtext.size(size).append_to(
        &mut layout_job,
        &style,
        FontSelection::Default,
        Align::Center,
    );

    ui.add(Label::new(layout_job));
}
