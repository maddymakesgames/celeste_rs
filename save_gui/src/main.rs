#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod editor;
mod main_menu;
use std::{future::Future, path::PathBuf};

use editor::EditorScreen;
use eframe::{
    egui::{CentralPanel, FontFamily, FontId, ScrollArea, Ui},
    App,
    CreationContext,
};
use tokio::runtime::Runtime;

use crate::main_menu::MainMenu;

#[cfg(not(target_family = "wasm"))]
fn main() {
    use eframe::{egui::ViewportBuilder, NativeOptions};

    tracing_subscriber::fmt::init();

    eframe::run_native(
        "Celeste Save Editor",
        NativeOptions {
            viewport: ViewportBuilder::default().with_drag_and_drop(true),
            ..Default::default()
        },
        Box::new(|cc| Box::new(SaveEditor::new(cc))),
    )
    .unwrap()
}

#[cfg(target_family = "wasm")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id",
                eframe::WebOptions::default(),
                Box::new(|cc| Box::new(SaveEditor::new(cc))),
            )
            .await
            .expect("Error starting eframe app")
    });
}

// Global state struct for the editor
struct SaveEditor {
    screen: ScreenState,
    runtime: Runtime,
}

impl SaveEditor {
    fn new(cc: &CreationContext) -> SaveEditor {
        #[cfg(not(target_family = "wasm"))]
        let runtime = tokio::runtime::Runtime::new().expect("Error creating tokio runtime");
        #[cfg(target_family = "wasm")]
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Error creating tokio runtime");

        let mut style = (*cc.egui_ctx.style()).clone();

        use eframe::egui::{FontFamily::Proportional, TextStyle::*};

        style.text_styles = [
            (Heading, FontId::new(32.0, Proportional)),
            (Name("header2".into()), FontId::new(26.0, Proportional)),
            (Body, FontId::new(18.0, Proportional)),
            (Name("info".into()), FontId::new(16.0, Proportional)),
            (Monospace, FontId::new(18.0, FontFamily::Monospace)),
            (Button, FontId::new(16.0, Proportional)),
            (Small, FontId::new(15.0, Proportional)),
        ]
        .into();

        cc.egui_ctx.set_style(style);

        SaveEditor {
            screen: ScreenState::Startup,
            runtime,
        }
    }
}

impl App for SaveEditor {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical()
                .auto_shrink(false)
                .show(ui, |ui| self.screen.update(ui, &self.runtime));
        });
    }
}

#[allow(clippy::large_enum_variant)]
enum ScreenState {
    Startup,
    Menu(MainMenu),
    Editor(EditorScreen),
}

impl ScreenState {
    fn update(&mut self, ui: &mut Ui, rt: &Runtime) {
        match self {
            ScreenState::Startup => *self = ScreenState::Menu(MainMenu::default()),
            ScreenState::Menu(m) =>
                if let Some((file_name, contents)) = m.display(ui, rt) {
                    match EditorScreen::new(file_name, contents) {
                        Ok(e) => *self = ScreenState::Editor(e),
                        Err(e) => eprintln!("{e}"),
                    }
                },
            ScreenState::Editor(e) => {
                e.display(ui, rt);
            }
        }
    }
}

// Provide a function for easily spawning futures on both native and web platforms
// While the native impl requires Send and wasm doesn't that shouldn't matter
// Since we develop for native first and that is the one with the stricter requirements
// We do need wasm to not require Send because rfd's FileHandle isn't Send on wasm
#[cfg(not(target_family = "wasm"))]
pub fn spawn<F>(rt: &Runtime, future: F)
where F: Future<Output = ()> + Send + 'static {
    rt.spawn(future);
}

#[cfg(target_family = "wasm")]
pub fn spawn<F>(_rt: &Runtime, future: F)
where F: Future<Output = ()> + 'static {
    wasm_bindgen_futures::spawn_local(future)
}

fn celeste_save_dir() -> Option<PathBuf> {
    // Celeste puts its save data in the 'local' folder for the os
    if cfg!(target_family = "unix") {
        Some(PathBuf::from(std::env::var("HOME").ok()?).join(".local/share/Celeste/Saves"))
    } else if cfg!(target_family = "windows") {
        Some(PathBuf::from(std::env::var("LOCALAPPDATA").ok()?).join("Celeste/Saves"))
    } else {
        None
    }
}
