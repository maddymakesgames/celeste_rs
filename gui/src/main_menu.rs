use std::{fmt::Display, io::Cursor, sync::Arc};

use celeste_rs::saves::{
    mods::{ParsedModSave, ParsedModSession, ParsedModSetting},
    ModSaveData,
    SaveData,
};
use eframe::egui::{TopBottomPanel, Ui};
use rfd::AsyncFileDialog;
use tokio::{
    runtime::Runtime,
    sync::{
        mpsc::{channel, error::TryRecvError, Receiver, Sender},
        Mutex,
    },
};

use crate::{spawn, ErrorSeverity, PopupWindow};

#[allow(dead_code)]
pub enum LoadableFiles {
    SaveData(String, Box<SaveData>),
    ModSaveData(String, ModSaveData),
    ModSave(String, ParsedModSave),
    ModSession(String, ParsedModSession),
    ModSetting(String, ParsedModSetting),
}

impl LoadableFiles {
    pub fn file_name(&self) -> &str {
        match self {
            LoadableFiles::SaveData(a, _)
            | LoadableFiles::ModSaveData(a, _)
            | LoadableFiles::ModSave(a, _)
            | LoadableFiles::ModSession(a, _)
            | LoadableFiles::ModSetting(a, _) => a,
        }
    }
}

#[derive(Default)]
pub struct MainMenu {
    #[allow(clippy::type_complexity)]
    file_listener: Option<Receiver<Option<(String, LoadableFiles)>>>,
    output: Vec<(String, LoadableFiles)>,
}

impl MainMenu {
    pub fn display(
        &mut self,
        ui: &mut Ui,
        rt: &Runtime,
        popups: &Arc<Mutex<Vec<PopupWindow>>>,
    ) -> Option<Vec<(String, LoadableFiles)>> {
        // Update the listener and return if we've recieved some data
        if let Some(inner) = self.update_listener(ui) {
            return Some(inner);
        }

        // On wasm make a note of the native version being the preferred way of using the app
        #[cfg(target_family = "wasm")]
        {
            ui.vertical(|ui| {
                ui.label(
                    "While the web version of this app should be perfectly functional, it is \
                     primarily developed for native and thus there could be bugs / performance \
                     issues.",
                );
                ui.horizontal(|ui| {
                    ui.label("You can find native downloads of the app");
                    ui.hyperlink_to(
                        "on the github",
                        "https://github.com/maddymakesgames/celeste_rs/releases",
                    );
                })
            });
        }

        // disable the ui when we're already trying to read a file
        ui.set_enabled(self.file_listener.is_none());

        if ui.button("Open Files").clicked() {
            // Create a file dialogue filtered for .celeste files
            let file_dialogue = AsyncFileDialog::new()
                .set_title("Open Celeste Save File")
                .add_filter("Celeste Save File", &["celeste"]);

            // Create a channel to send the parsed file back through
            let (send, recv) = channel(5);

            // Spawn a task to read and parse the file
            spawn(rt, handle_file_picker(file_dialogue, send, popups.clone()));

            // Keep the recieving end of the channel so we can listen for the parsed file
            self.file_listener = Some(recv);
        }

        #[cfg(not(target_family = "wasm"))]
        if ui.button("Load Celeste Save Folder").clicked() {
            use std::{ffi::OsString, io::BufReader};

            use std::fs::{read_dir, OpenOptions};
            let file_dialogue = AsyncFileDialog::new()
                .set_title("Celeste Save Folder")
                .pick_folder();

            let (send, recv) = channel(5);
            let popups = popups.clone();
            spawn(rt, async move {
                if let Some(dir) = file_dialogue.await {
                    match read_dir(dir.path()) {
                        Ok(iter) =>
                            for entry in iter.flatten() {
                                // If its not a .celeste file, just skip over it
                                if entry.path().extension() != Some(&OsString::from("celeste")) {
                                    continue;
                                }

                                let file_name = entry.file_name().to_string_lossy().to_string();

                                // File names are in the format (File number)-[modded file type]-[related mod].celeste
                                // So we can determine how to treat the file by splitting on '-' and '.'
                                let mut file_name_parts = file_name.split(['-', '.']);

                                let Some(file_number) =
                                    file_name_parts.next().map(ToOwned::to_owned)
                                else {
                                    continue;
                                };

                                // ignore settings.celeste and debug-* files
                                if file_number == "settings" || file_number == "debug" {
                                    continue;
                                }

                                // At this point we're likely to be able to handle it so its useful to open the file at this point

                                if let Ok(file) = OpenOptions::new().read(true).open(entry.path()) {
                                    let reader = BufReader::new(file);
                                    let file_type = file_name_parts.next();

                                    match file_type {
                                        Some("modsavedata") =>
                                            match ModSaveData::from_reader(reader) {
                                                Ok(modsavedata) => {
                                                    if send
                                                        .send(Some((
                                                            file_number,
                                                            LoadableFiles::ModSaveData(
                                                                file_name,
                                                                modsavedata,
                                                            ),
                                                        )))
                                                        .await
                                                        .is_err()
                                                    {
                                                        send_error(&popups).await;
                                                    }
                                                }
                                                Err(e) => parse_error(&popups, &file_name, e).await,
                                            },
                                        Some("modsave") =>
                                            match ParsedModSave::parse_from_path(entry.path()) {
                                                Ok((_, save)) =>
                                                    if send
                                                        .send(Some((
                                                            file_number,
                                                            LoadableFiles::ModSave(file_name, save),
                                                        )))
                                                        .await
                                                        .is_err()
                                                    {
                                                        send_error(&popups).await
                                                    },
                                                Err(e) => parse_error(&popups, &file_name, e).await,
                                            },
                                        Some("modsession") =>
                                            match ParsedModSession::parse_from_path(entry.path()) {
                                                Ok((_, session)) =>
                                                    if send
                                                        .send(Some((
                                                            file_number,
                                                            LoadableFiles::ModSession(
                                                                file_name, session,
                                                            ),
                                                        )))
                                                        .await
                                                        .is_err()
                                                    {
                                                        send_error(&popups).await
                                                    },
                                                Err(e) => parse_error(&popups, &file_name, e).await,
                                            },
                                        // No second part to the file name means its just a root save file
                                        Some("celeste") => match SaveData::from_reader(reader) {
                                            Ok(save) =>
                                                if send
                                                    .send(Some((
                                                        file_number,
                                                        LoadableFiles::SaveData(
                                                            file_name,
                                                            Box::new(save),
                                                        ),
                                                    )))
                                                    .await
                                                    .is_err()
                                                {
                                                    send_error(&popups).await;
                                                },
                                            Err(e) => parse_error(&popups, &file_name, e).await,
                                        },
                                        // Unsupported filetypes
                                        Some(_) | None => {}
                                    }
                                }
                            },
                        Err(e) => popups.lock().await.push(PopupWindow::new(
                            ErrorSeverity::Error,
                            format!("Error opening directory: {e}"),
                        )),
                    }
                }
            });

            self.file_listener = Some(recv);
        }

        TopBottomPanel::bottom("version_number_panel")
            .show_separator_line(false)
            .resizable(false)
            .show_inside(ui, |ui| ui.small(format!("v{}", env!("CARGO_PKG_VERSION"))));

        None
    }

    fn update_listener(&mut self, ui: &mut Ui) -> Option<Vec<(String, LoadableFiles)>> {
        if let Some(recv) = &mut self.file_listener {
            // Try to recieve file data from the channel
            // We use try_recv because it will give Err(Empty) if it can't immediately read data
            match recv.try_recv() {
                Ok(Some(file)) => {
                    self.output.push(file);
                }
                Err(TryRecvError::Disconnected) => {
                    self.file_listener = None;
                    if !self.output.is_empty() {
                        return Some(std::mem::take(&mut self.output));
                    }
                }
                _ => {}
            }

            // Display a little spinner to show we're working <3
            ui.spinner();
        }
        None
    }
}

async fn handle_file_picker(
    file_dialogue: AsyncFileDialog,
    send: Sender<Option<(String, LoadableFiles)>>,
    popups: Arc<Mutex<Vec<PopupWindow>>>,
) {
    // Wait for the user to pick a file
    let files = file_dialogue.pick_files().await;

    if let Some(files) = files {
        for file in files {
            // Read the contents of the file
            let name = file.file_name();
            let contents = file.read().await;
            drop(file);

            let Some(file_number) = name.split(['-', '.']).next() else {
                popups.lock().await.push(PopupWindow::new(
                    ErrorSeverity::Warning,
                    format!("File \"{name}\" is not a loadable save file."),
                ));
                return;
            };

            // Attempt to parse the save file showing an error popup if we fail
            match SaveData::from_reader(Cursor::new(contents)) {
                Ok(save) =>
                    if send
                        .send(Some((
                            file_number.to_owned(),
                            LoadableFiles::SaveData(name, Box::new(save)),
                        )))
                        .await
                        .is_err()
                    {
                        send_error(&popups).await;
                    },
                Err(e) => {
                    popups.lock().await.push(PopupWindow::new(
                        ErrorSeverity::Error,
                        format!(
                            "Errors found when parsing save file: {e}.\nMake sure the file you \
                             selected is actually a save file.\nIf this continues please report \
                             it as a bug on github."
                        ),
                    ));
                }
            }
        }
    }
}

async fn send_error(popups: &Arc<Mutex<Vec<PopupWindow>>>) {
    popups.lock().await.push(PopupWindow::new(
        ErrorSeverity::Error,
        "Error sending data back to main thread.\nThis is a bug, please make a bug report on \
         github.",
    ))
}

async fn parse_error<T: Display>(popups: &Arc<Mutex<Vec<PopupWindow>>>, file_name: &str, err: T) {
    popups.lock().await.push(PopupWindow::new(
        ErrorSeverity::Error,
        format!(
            "Errors found when parsing save file \"{file_name}\": {err}.\nMake sure the file you \
             selected is actually a save file.\nIf this continues please report it as a bug on \
             github."
        ),
    ))
}
