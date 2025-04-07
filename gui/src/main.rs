#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod editor;
mod main_menu;
mod tabbed;
use std::{future::Future, path::PathBuf, sync::Arc};

use editor::EditorScreen;
use eframe::{
    App,
    CreationContext,
    egui::{
        Align2,
        CentralPanel,
        Color32,
        Context,
        FontFamily::{self, Proportional},
        FontId,
        RichText,
        TextStyle::{self, *},
        Ui,
        WidgetText,
        Window,
        scroll_area::ScrollBarVisibility,
    },
};
use indexmap::IndexMap;
use tokio::{runtime::Runtime, sync::Mutex};

use crate::{
    main_menu::{LoadableFiles, MainMenu},
    tabbed::TabbedContentWidget,
};

#[cfg(not(target_family = "wasm"))]
fn main() {
    use eframe::{NativeOptions, egui::ViewportBuilder};

    tracing_subscriber::fmt::init();

    // Expect is fine since its on startup
    eframe::run_native(
        "celeste.rs",
        NativeOptions {
            viewport: ViewportBuilder::default().with_drag_and_drop(true),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(SaveEditor::new(cc)))),
    )
    .expect("Error starting eframe app")
}

#[cfg(target_family = "wasm")]
fn main() {
    use eframe::wasm_bindgen::JsCast;
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    // Expect is fine since its on startup
    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Canvas doesn't exist")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("Canvas element isn't a canvas");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| Ok(Box::new(SaveEditor::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}

// Global state struct for the editor
struct SaveEditor {
    selected_screen: usize,
    screens: IndexMap<String, ScreenState>,
    runtime: Runtime,
    popups: Arc<Mutex<Vec<PopupWindow>>>,
}

impl SaveEditor {
    fn new(cc: &CreationContext) -> SaveEditor {
        // expects are fine since if this fails theres nothing we can do
        // We create a multithreading runtime on native, and a single-threaded runtime on wasm.
        // We don't *really* use the runtime on wasm but it'd be too annoying for it to not exist
        // We also can rewrite the code easily to actually use the runtime so /shrug
        #[cfg(not(target_family = "wasm"))]
        let runtime = tokio::runtime::Runtime::new().expect("Error creating tokio runtime");
        #[cfg(target_family = "wasm")]
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Error creating tokio runtime");


        let mut style = (*cc.egui_ctx.style()).clone();

        // Modify the text_styles to include our own font sizes
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

        let mut screens = IndexMap::new();
        screens.insert("".to_owned(), ScreenState::Menu(MainMenu::default()));

        SaveEditor {
            selected_screen: 0,
            screens,
            runtime,
            popups: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl App for SaveEditor {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        // Show the main window contents
        CentralPanel::default().show(ctx, |ui| {
            let mut add_empty_tab = false;
            TabbedContentWidget::show(
                ui,
                &mut self.selected_screen,
                self.screens
                    .iter()
                    .map(|(_number, state)| state.name())
                    .map(str::to_owned)
                    .collect::<Vec<_>>(),
                ScrollBarVisibility::VisibleWhenNeeded,
                TextStyle::Name("header2".into()),
                |selected, ui| {
                    if let Some(iter) =
                        self.screens[selected].update(ui, &self.runtime, &self.popups)
                    {
                        let mut vec = iter.collect::<Vec<_>>();
                        vec.sort_by_key(|(_, f)| f.file_name().to_owned());
                        for (file_num, file) in vec.into_iter() {
                            if self.screens.contains_key(&file_num) {
                                let screen = self.screens.get_mut(&file_num).unwrap();
                                screen.add_file(file)
                            } else {
                                self.screens.insert(
                                    file_num.clone(),
                                    ScreenState::Editor(EditorScreen::new(
                                        format!("{file_num}.celeste"),
                                        file,
                                    )),
                                );
                            }
                        }
                    }
                },
                Some(&mut add_empty_tab),
            );
            if add_empty_tab {
                let mut max_file = 0;
                for (name, _) in &self.screens {
                    if let Ok(num) = name.parse::<i32>() {
                        if num >= max_file {
                            max_file = num + 1;
                        }
                    }
                }

                let file_name = format!("{max_file}.celeste");
                self.screens.insert(
                    max_file.to_string(),
                    ScreenState::Editor(EditorScreen::new(
                        file_name.clone(),
                        LoadableFiles::SaveData(file_name, Box::default()),
                    )),
                );
            }
        });

        let mut popup_guard = self.popups.blocking_lock();
        let mut to_remove = None;
        // Loop over any open popups and display them
        for (i, popup) in popup_guard.iter().enumerate() {
            match popup.show(ctx) {
                PopupResult::ClosePopup => to_remove = Some(i),
                PopupResult::Nothing => {}
                PopupResult::CloseApp => {
                    #[cfg(target_family = "wasm")]
                    // unwraps are safe cause window will always exist and I don't think reload can fail
                    web_sys::window().unwrap().location().reload().unwrap();
                    #[cfg(not(target_family = "wasm"))]
                    {
                        use eframe::egui::ViewportCommand;
                        ctx.send_viewport_cmd(ViewportCommand::Close);
                    }
                }
            }
        }

        // If the user closed a popup remove it from the list
        if let Some(to_remove) = to_remove {
            popup_guard.remove(to_remove);
        }
    }
}

#[allow(clippy::large_enum_variant)]
enum ScreenState {
    Menu(MainMenu),
    Editor(EditorScreen),
}

impl ScreenState {
    fn update(
        &mut self,
        ui: &mut Ui,
        rt: &Runtime,
        popups: &Arc<Mutex<Vec<PopupWindow>>>,
    ) -> Option<impl Iterator<Item = (String, LoadableFiles)> + use<>> {
        match self {
            ScreenState::Menu(m) =>
            // The main menu displays until a file has been opened
            // In which case we transition to the editor
                if let Some(saves) = m.display(ui, rt, popups) {
                    let saves_iter = saves.into_iter();
                    return Some(saves_iter);
                },
            // The editor (current) doesn't ever transition out
            ScreenState::Editor(e) => {
                e.display(ui, rt, popups);
            }
        }
        None
    }

    fn name(&self) -> &str {
        match self {
            ScreenState::Menu(_) => "new tab",
            ScreenState::Editor(e) => e.name(),
        }
    }

    fn add_file(&mut self, file: LoadableFiles) {
        match self {
            ScreenState::Menu(_) => {}
            ScreenState::Editor(e) => {
                let name = file.file_name();
                if !e.files.iter().map(|f| f.file_name()).any(|f| f == name) {
                    e.files.push(file)
                }
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

/// An error popup window
struct PopupWindow {
    /// The severity of the message
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
