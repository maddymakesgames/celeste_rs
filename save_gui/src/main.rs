#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod editor;
mod main_menu;
mod tabbed;
use std::{future::Future, path::PathBuf, sync::Arc};

use editor::EditorScreen;
use eframe::{
    egui::{
        Align2,
        CentralPanel,
        Color32,
        Context,
        FontFamily,
        FontId,
        RichText,
        ScrollArea,
        Ui,
        ViewportCommand,
        WidgetText,
        Window,
    },
    App,
    CreationContext,
};
use tokio::{runtime::Runtime, sync::Mutex};

use crate::main_menu::MainMenu;

#[cfg(not(target_family = "wasm"))]
fn main() {
    use eframe::{egui::ViewportBuilder, NativeOptions};

    tracing_subscriber::fmt::init();

    // Expect is fine since its on startup
    eframe::run_native(
        "Celeste Save Editor",
        NativeOptions {
            viewport: ViewportBuilder::default().with_drag_and_drop(true),
            ..Default::default()
        },
        Box::new(|cc| Box::new(SaveEditor::new(cc))),
    )
    .expect("Error starting eframe app")
}

#[cfg(target_family = "wasm")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    // Expect is fine since its on startup
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
    popups: Arc<Mutex<Vec<PopupWindow>>>,
}

impl SaveEditor {
    fn new(cc: &CreationContext) -> SaveEditor {
        // expects are fine since if this fails theres nothing we can do
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
            popups: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl App for SaveEditor {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical()
                .auto_shrink(false)
                .show(ui, |ui| self.screen.update(ui, &self.runtime, &self.popups));
        });

        let mut popup_guard = self.popups.blocking_lock();
        let mut to_remove = None;
        for (i, popup) in popup_guard.iter().enumerate() {
            match popup.show(ctx) {
                PopupResult::ClosePopup => to_remove = Some(i),
                PopupResult::Nothing => {}
                PopupResult::CloseApp => {
                    // unwraps are safe cause window will always exist and I don't think reload can fail
                    #[cfg(target_family = "wasm")]
                    web_sys::window().unwrap().location().reload().unwrap();
                    #[cfg(not(target_family = "wasm"))]
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }
            }
        }

        if let Some(to_remove) = to_remove {
            popup_guard.remove(to_remove);
        }
    }
}

#[allow(clippy::large_enum_variant)]
enum ScreenState {
    Startup,
    Menu(MainMenu),
    Editor(EditorScreen),
}

impl ScreenState {
    fn update(&mut self, ui: &mut Ui, rt: &Runtime, popups: &Arc<Mutex<Vec<PopupWindow>>>) {
        match self {
            ScreenState::Startup => *self = ScreenState::Menu(MainMenu::default()),
            ScreenState::Menu(m) =>
                if let Some((file_name, contents)) = m.display(ui, rt) {
                    match EditorScreen::new(file_name, contents) {
                        Ok(e) => *self = ScreenState::Editor(e),
                        Err(e) => {
                            let mut popups = popups.blocking_lock();
                            popups.push(PopupWindow::new(
                                ErrorSeverity::Error,
                                format!(
                                    "Error reading savefile: {e:?}.\nMake sure this actually is a \
                                     save file."
                                ),
                            ))
                        }
                    }
                },
            ScreenState::Editor(e) => {
                e.display(ui, rt, popups);
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

struct PopupWindow {
    severity: ErrorSeverity,
    /// The text displayed on the popup
    text: String,
    /// The text on the close button
    button_text: &'static str,
    /// Whether to close the app after the button is clicked
    close: bool,
}

impl PopupWindow {
    pub fn new(severity: ErrorSeverity, text: impl ToString) -> Self {
        Self {
            severity,
            text: text.to_string(),
            button_text: if ErrorSeverity::Severe == severity {
                "Close App"
            } else {
                "Close"
            },
            close: ErrorSeverity::Severe == severity,
        }
    }

    pub fn show(&self, ctx: &Context) -> PopupResult {
        Window::new(self.severity)
            .collapsible(false)
            .auto_sized()
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(&self.text);
                    if ui.button(self.button_text).clicked() {
                        if self.close {
                            PopupResult::CloseApp
                        } else {
                            PopupResult::ClosePopup
                        }
                    } else {
                        PopupResult::Nothing
                    }
                })
                .inner
            })
            // Window cannot be collapssed so unwraps are safe
            .unwrap()
            .inner
            .unwrap()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Severe,
}

impl ErrorSeverity {
    pub fn text(&self) -> &'static str {
        match self {
            ErrorSeverity::Info => "Info",
            ErrorSeverity::Warning => "Warning",
            ErrorSeverity::Error => "Error",
            ErrorSeverity::Severe => "Critical Error!",
        }
    }

    pub fn color(&self) -> Color32 {
        match self {
            ErrorSeverity::Info => Color32::PLACEHOLDER,
            ErrorSeverity::Warning => Color32::from_rgb(242, 226, 12),
            ErrorSeverity::Error | ErrorSeverity::Severe => Color32::from_rgb(244, 31, 31),
        }
    }
}

impl From<ErrorSeverity> for WidgetText {
    fn from(val: ErrorSeverity) -> Self {
        WidgetText::RichText(RichText::new(val.text()).color(val.color()))
    }
}

enum PopupResult {
    ClosePopup,
    Nothing,
    CloseApp,
}
