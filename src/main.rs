use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use dark_light::Mode;
use eframe::egui::{CentralPanel, Context, ScrollArea, TextEdit, Vec2, Visuals};
use eframe::{App, Frame, NativeOptions};

use crate::document::Document;

mod document;

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

struct SkrivTillsammansApp {
    doc1: Rc<RefCell<Document>>,
    doc2: Rc<RefCell<Document>>,
}

impl SkrivTillsammansApp {
    fn new(ctx: &eframe::CreationContext<'_>) -> Self {
        match dark_light::detect() {
            Mode::Dark => ctx.egui_ctx.set_visuals(Visuals::dark()),
            Mode::Light => ctx.egui_ctx.set_visuals(Visuals::light()),
            Mode::Default => ctx.egui_ctx.set_visuals(Visuals::default()),
        }

        let doc1 = Rc::new(RefCell::new(Document::new("", 1)));
        let doc2 = Rc::new(RefCell::new(doc1.borrow().fork(2)));

        let doc2_i = doc2.clone();
        doc1.borrow_mut().insert_listener(move |i| {
            doc2_i.borrow_mut().integrate_insertion(i);
            println!("{}", serde_json::to_string_pretty(i).unwrap());
        });
        let doc2_d = doc2.clone();
        doc1.borrow_mut()
            .delete_listener(move |d| doc2_d.borrow_mut().integrate_deletion(d));

        let doc1_i = doc1.clone();
        doc2.borrow_mut()
            .insert_listener(move |i| doc1_i.borrow_mut().integrate_insertion(i));
        let doc1_d = doc1.clone();
        doc2.borrow_mut()
            .delete_listener(move |d| doc1_d.borrow_mut().integrate_deletion(d));

        SkrivTillsammansApp { doc1, doc2 }
    }
}

impl App for SkrivTillsammansApp {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |ui| {
                ScrollArea::both()
                    .id_source("scroll1")
                    .show(&mut ui[0], |ui| {
                        ui.add(
                            TextEdit::multiline(self.doc1.borrow_mut().deref_mut()).min_size(
                                Vec2 {
                                    x: 0.,
                                    y: ui.available_height(),
                                },
                            ),
                        );
                    });
                ScrollArea::both()
                    .id_source("scroll2")
                    .show(&mut ui[1], |ui| {
                        ui.add(
                            TextEdit::multiline(self.doc2.borrow_mut().deref_mut()).min_size(
                                Vec2 {
                                    x: 0.,
                                    y: ui.available_height(),
                                },
                            ),
                        );
                    });
            });
        });
    }
}
