use std::{io::Cursor, sync::Arc};

use celeste_rs::saves::SaveData;
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

#[derive(Default)]
pub struct MainMenu {
    #[allow(clippy::type_complexity)]
    file_listener: Option<Receiver<Option<(String, SaveData)>>>,
    output: Vec<(String, SaveData)>,
}

impl MainMenu {
    pub fn display(
        &mut self,
        ui: &mut Ui,
        rt: &Runtime,
        popups: &Arc<Mutex<Vec<PopupWindow>>>,
    ) -> Option<Vec<(String, SaveData)>> {
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
                            'dir_iter: for entry in iter.flatten() {
                                let file_name = entry.file_name().to_string_lossy().to_string();

                                'char_loop: for char in file_name.chars() {
                                    if char == '.' {
                                        break 'char_loop;
                                    }

                                    if !char.is_ascii_digit() {
                                        continue 'dir_iter;
                                    }
                                }

                                if entry.path().extension() != Some(&OsString::from("celeste")) {
                                    continue;
                                }


                                if let Ok(file) = OpenOptions::new().read(true).open(entry.path()) {
                                    match SaveData::from_reader(BufReader::new(file)) {
                                        Ok(save) =>
                                            if send.send(Some((file_name, save))).await.is_err() {
                                                popups.lock().await.push(PopupWindow::new(
                                                    ErrorSeverity::Error,
                                                    "Error sending data back to main \
                                                     thread.\nThis is a bug, please make a bug \
                                                     report on github.",
                                                ))
                                            },
                                        Err(e) => {
                                            popups.lock().await.push(PopupWindow::new(
                                                ErrorSeverity::Error,
                                                format!(
                                                    "Errors found when parsing save file \
                                                     \"{file_name}\": {e}.\nMake sure the file \
                                                     you selected is actually a save file.\nIf \
                                                     this continues please report it as a bug on \
                                                     github."
                                                ),
                                            ));
                                        }
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

    fn update_listener(&mut self, ui: &mut Ui) -> Option<Vec<(String, SaveData)>> {
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
    send: Sender<Option<(String, SaveData)>>,
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

            // Attempt to parse the save file showing an error popup if we fail
            match SaveData::from_reader(Cursor::new(contents)) {
                Ok(save) =>
                    if send.send(Some((name, save))).await.is_err() {
                        popups.lock().await.push(PopupWindow::new(
                            ErrorSeverity::Error,
                            "Error sending data back to main thread.\nThis is a bug, please make \
                             a bug report on github.",
                        ))
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
