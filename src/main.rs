use dark_light::Mode;
use eframe::egui::{CentralPanel, Context, ScrollArea, TextEdit, Vec2, Visuals};
use eframe::{App, Frame, NativeOptions};

fn main() -> eframe::Result<()> {
    let native_options = NativeOptions {
        resizable: true,
        initial_window_size: Some(Vec2 {
            x: 1400.0,
            y: 800.0,
        }),
        min_window_size: Some(Vec2 { x: 700., y: 400. }),
        app_id: Some("skriv-tillsammans".to_string()),
        ..Default::default()
    };

    eframe::run_native(
        "Skriv Tillsammans",
        native_options,
        Box::new(|cc| Box::new(SkrivTillsammansApp::new(cc))),
    )
}

#[derive(Default)]
struct SkrivTillsammansApp {
    text1: String,
    text2: String,
}

impl SkrivTillsammansApp {
    fn new(ctx: &eframe::CreationContext<'_>) -> Self {
        match dark_light::detect() {
            Mode::Dark => ctx.egui_ctx.set_visuals(Visuals::dark()),
            Mode::Light => ctx.egui_ctx.set_visuals(Visuals::light()),
            Mode::Default => ctx.egui_ctx.set_visuals(Visuals::default()),
        }
        SkrivTillsammansApp {
            text1: "abc".to_string(),
            text2: "123".to_string(),
        }
    }
}

impl App for SkrivTillsammansApp {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |ui| {
                ScrollArea::both()
                    .id_source("scroll1")
                    .show(&mut ui[0], |ui| {
                        ui.add(TextEdit::multiline(&mut self.text1).min_size(Vec2 {
                            x: 0.,
                            y: ui.available_height(),
                        }));
                    });
                ScrollArea::both()
                    .id_source("scroll2")
                    .show(&mut ui[1], |ui| {
                        ui.add(TextEdit::multiline(&mut self.text2).min_size(Vec2 {
                            x: 0.,
                            y: ui.available_height(),
                        }));
                    });
            });
        });
    }
}
