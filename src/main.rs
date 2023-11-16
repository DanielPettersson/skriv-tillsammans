use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use cola::Deletion;
use dark_light::Mode;
use eframe::egui::{CentralPanel, Context, ScrollArea, TextEdit, Vec2, Visuals};
use eframe::{App, Frame, NativeOptions};
use tokio::sync::mpsc::unbounded_channel;
use uuid::Uuid;

use crate::document::{Document, Insertion};

mod document;
mod peers;

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
    doc: Arc<Mutex<Document>>,
}

impl SkrivTillsammansApp {
    fn new(ctx: &eframe::CreationContext<'_>) -> Self {
        match dark_light::detect() {
            Mode::Dark => ctx.egui_ctx.set_visuals(Visuals::dark()),
            Mode::Light => ctx.egui_ctx.set_visuals(Visuals::light()),
            Mode::Default => ctx.egui_ctx.set_visuals(Visuals::default()),
        }

        let uuid = Uuid::new_v4();
        let replica_id = uuid.as_u64_pair().1;
        let doc = Arc::new(Mutex::new(Document::new("", replica_id)));

        let (local_delete_sender, local_delete_receiver) = unbounded_channel();
        let (local_insert_sender, local_insert_receiver) = unbounded_channel();
        let (remote_delete_sender, remote_delete_receiver): (Sender<String>, Receiver<String>) =
            channel();
        let (remote_insert_sender, remote_insert_receiver): (Sender<String>, Receiver<String>) =
            channel();

        let doc_clone = doc.clone();
        let ctx_clone = ctx.egui_ctx.clone();
        thread::spawn(move || {
            for data in remote_delete_receiver {
                let d: Deletion = serde_json::from_str(&data).unwrap();
                doc_clone.lock().unwrap().integrate_deletion(&d);
                ctx_clone.request_repaint()
            }
        });
        let doc_clone = doc.clone();
        let ctx_clone = ctx.egui_ctx.clone();
        thread::spawn(move || {
            for data in remote_insert_receiver {
                let i: Insertion = serde_json::from_str(&data).unwrap();
                doc_clone.lock().unwrap().integrate_insertion(&i);
                ctx_clone.request_repaint();
            }
        });

        thread::spawn(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    peers::peers(
                        local_delete_receiver,
                        remote_delete_sender,
                        local_insert_receiver,
                        remote_insert_sender,
                    )
                    .await
                })
                .unwrap();
        });

        {
            let mut d = doc.lock().unwrap();
            d.insert_listener(move |i| {
                let json = serde_json::to_string(i).unwrap();
                local_insert_sender.send(json).unwrap()
            });
            d.delete_listener(move |d| {
                let json = serde_json::to_string(d).unwrap();
                local_delete_sender.send(json).unwrap()
            });
        }

        SkrivTillsammansApp { doc }
    }
}

impl App for SkrivTillsammansApp {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::both().show(ui, |ui| {
                let mut doc = self.doc.lock().unwrap();
                ui.add(TextEdit::multiline(&mut *doc).min_size(ui.available_size()));
            });
        });
    }
}
