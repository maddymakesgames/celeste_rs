#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod editor;
mod main_menu;
use editor::EditorScreen;
use eframe::{
    egui::{CentralPanel, ScrollArea, Ui},
    App, CreationContext,
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

    eframe::run_web(
        "main_canvas",
        WebOptions::default(),
        Box::new(|_cc| Box::new(SaveEditor {})),
    )
    .unwrap()
}

// Global state struct for the editor
struct SaveEditor {
    screen: ScreenState,
    runtime: Runtime,
}

impl SaveEditor {
    fn new(_cc: &CreationContext) -> SaveEditor {
        #[cfg(not(target_family = "wasm"))]
        let runtime = tokio::runtime::Runtime::new().expect("Error creating tokio runtime");
        #[cfg(target_family = "wasm")]
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Error creating tokio runtime");

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

        #[cfg(target_family = "wasm")]
        self.runtime.block_on(async {
            tokio::task::yield_now().await;
            ctx.request_repaint();
        });
    }
}

enum ScreenState {
    Startup,
    Menu(MainMenu),
    Editor(EditorScreen),
}

impl ScreenState {
    fn update(&mut self, ui: &mut Ui, rt: &Runtime) {
        match self {
            ScreenState::Startup => *self = ScreenState::Menu(MainMenu::default()),
            ScreenState::Menu(m) => {
                if let Some(file) = m.display(ui, rt) {
                    match EditorScreen::new(file) {
                        Ok(e) => *self = ScreenState::Editor(e),
                        Err(e) => eprintln!("{e}"),
                    }
                }
            }
            ScreenState::Editor(e) => {
                e.display(ui, rt);
            }
        }
    }
}
