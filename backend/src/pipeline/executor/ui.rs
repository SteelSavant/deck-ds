#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::sync::mpsc::{Receiver, Sender};

use eframe::egui;
use egui::{Color32, Frame, Label, Pos2, RichText, Style, Ui, ViewportBuilder};
use winit::platform::x11::EventLoopBuilderExtX11;

pub enum UiEvent {
    UpdateStatusMsg(String),
    Close,
}

pub struct DeckDsUi {
    // primary_size: [f32; 2],
    // secondary_size: [f32; 2],
    primary_position: Pos2,
    secondary_position: Pos2,
    secondary_text: String,
    custom_frame: Frame,
    rx: Receiver<UiEvent>,
    tx: Sender<egui::Context>,
}

impl DeckDsUi {
    pub fn new(
        // primary_size: [f32; 2],
        // secondary_size: [f32; 2],
        primary_position: Pos2,
        secondary_position: Pos2,
        rx: Receiver<UiEvent>,
        tx: Sender<egui::Context>,
    ) -> Self {
        let custom_frame = egui::containers::Frame {
            inner_margin: egui::style::Margin {
                left: 10.,
                right: 10.,
                top: 10.,
                bottom: 10.,
            },
            outer_margin: egui::style::Margin {
                left: 10.,
                right: 10.,
                top: 10.,
                bottom: 10.,
            },
            rounding: egui::Rounding {
                nw: 1.0,
                ne: 1.0,
                sw: 1.0,
                se: 1.0,
            },
            shadow: eframe::epaint::Shadow {
                extrusion: 1.0,
                color: Color32::BLACK,
            },
            fill: Color32::BLACK,
            stroke: egui::Stroke::new(2.0, Color32::BLACK),
        };

        Self {
            // primary_size,
            // secondary_size,
            primary_position,
            secondary_position,
            secondary_text: "starting up...".to_string(),
            custom_frame,
            rx,
            tx,
        }
    }

    pub fn run(self) -> Result<(), eframe::Error> {
        let options = eframe::NativeOptions {
            viewport: build_viewport(self.primary_position),
            event_loop_builder: Some(Box::new(|builder| {
                builder.with_any_thread(true);
            })),
            ..Default::default()
        };

        eframe::run_native(
            "DeckDS",
            options,
            Box::new(|cc| {
                self.tx
                    .send(cc.egui_ctx.clone())
                    .expect("send egui context should succeed");
                Box::new(self)
            }),
        )
    }
}

impl eframe::App for DeckDsUi {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        println!("repaint");

        match self.rx.try_recv() {
            Ok(event) => match event {
                UiEvent::UpdateStatusMsg(msg) => self.secondary_text = msg,
                UiEvent::Close => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            },
            Err(_) => (),
        }
        egui::CentralPanel::default()
            .frame(self.custom_frame)
            .show(ctx, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.visuals_mut().override_text_color = Some(Color32::WHITE);

                    create_deckds_label(
                        ui,
                        RichText::new("hold (select) + (start) to exit\ngame after launch"),
                        ui.available_height(),
                    );
                })
            });

        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("DeckDS Secondary"),
            build_viewport(self.secondary_position),
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

fn build_viewport(position: Pos2) -> ViewportBuilder {
    egui::ViewportBuilder::default()
        // .with_inner_size(size)
        .with_decorations(false)
        .with_active(false)
        .with_position(position)
        .with_resizable(false)
        .with_maximized(true)
        .with_window_level(egui::WindowLevel::AlwaysOnBottom)
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
