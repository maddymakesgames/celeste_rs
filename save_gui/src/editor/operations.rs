use std::{fs::OpenOptions, io::Write, sync::Arc};

use celeste_rs::saves::{everest::LevelSetStats, SaveData};
use eframe::egui::{RichText, Ui};
use tokio::{
    runtime::Runtime,
    sync::{oneshot::error::TryRecvError, Mutex},
};

use crate::{
    celeste_save_dir,
    editor::{CelesteEditorRichTextExt, CelesteEditorUiExt, EditorScreen},
    spawn,
    ErrorSeverity,
    PopupWindow,
};


impl EditorScreen {
    pub fn show_operations(
        &mut self,
        ui: &mut Ui,
        rt: &Runtime,
        popups: &Arc<Mutex<Vec<PopupWindow>>>,
    ) {
        ui.vertical(|ui| {
            // TODO: remove most expects from this impl
            ui.horizontal(|ui| {
                if ui.button(RichText::new("Save File").info()).clicked() {
                    self.save_file(rt, popups);
                }

                if ui.button(RichText::new("Merge in file").info()).clicked() {
                    self.merge_file(rt, popups);
                }

                ui.info_hover(
                    "Merges in any applicable data from a different save file into this \
                     one.\nThis might not merge all the data you would want it to and there may \
                     still be bugs, so it is highly recommended you keep backups of your saves \
                     before using this.",
                )
            });

            ui.horizontal(|ui| {
                ui.label("Disable Safety Checks:");
                ui.checkbox(&mut self.safety_off, "");
                ui.info_hover(
                    "Check this to enable editing every field.\nThis is off by default as some \
                     values should not be independently edited.\nMake sure you know what you're \
                     doing when you check this.\nYou can hover on a disable item to see why it \
                     might be unsafe.\n(as of alpha version not all tooltips implemented and not \
                     all auto-editing implemented)",
                )
            });
        });
    }

    fn save_file(&self, rt: &Runtime, popups: &Arc<Mutex<Vec<PopupWindow>>>) {
        let file_dialogue = rfd::AsyncFileDialog::new().set_file_name(&self.file_name);
        let serialized = match self.save.to_string() {
            Ok(s) => s,
            Err(e) => {
                let mut popup_guard = popups.blocking_lock();
                popup_guard.push(PopupWindow::new(
                    ErrorSeverity::Error,
                    format!(
                        "Error serializing save file: {e:?}.\nThis is likely a bug. Please report \
                         it on github."
                    ),
                ));
                return;
            }
        };
        let popups = popups.clone();
        spawn(rt, async move {
            if let Some(file) = file_dialogue.save_file().await {
                #[cfg(not(target_family = "wasm"))]
                {
                    let mut file = match OpenOptions::new()
                        .create(true)
                        .write(true)
                        .open(file.path())
                    {
                        Ok(f) => f,
                        Err(e) => {
                            let mut popup_guard = popups.lock().await;
                            popup_guard.push(PopupWindow::new(
                                ErrorSeverity::Error,
                                format!(
                                    "Error opening file: {e:?}.\nPlease make sure you are \
                                     selecting a valid location on disk.\nThis could be a bug. \
                                     Please report it to github if it continues to happen."
                                ),
                            ));
                            return;
                        }
                    };

                    if let Err(e) = file.write_all(serialized.as_bytes()) {
                        let mut popup_guard = popups.lock().await;
                        popup_guard.push(PopupWindow::new(
                            ErrorSeverity::Error,
                            format!(
                                "Error writing to file: {e:?}.\nPlease make sure you have space \
                                 on disk and can write to the selected location.\nThis could be a \
                                 bug. Please report it on github if it continues to happen"
                            ),
                        ));
                    }
                }
                #[cfg(target_family = "wasm")]
                {
                    if let Err(e) = file.write(serialized.as_bytes()).await {
                        let mut popup_guard = popups.lock().await;
                        popup_guard.push(PopupWindow::new(
                            ErrorSeverity::Error,
                            format!(
                                "Error writing to file: {e:?}.\nPlease make sure you have space \
                                 on disk and can write to the selected location.\nThis could be a \
                                 bug. Please report it on github if it continues to happen"
                            ),
                        ));
                    }
                }
            }
        });
    }

    fn merge_file(&mut self, rt: &Runtime, popups: &Arc<Mutex<Vec<PopupWindow>>>) {
        let file_dialogue = rfd::AsyncFileDialog::new()
            .add_filter("Celeste Save File", &["celeste"])
            .set_directory(celeste_save_dir().unwrap_or_default());

        let (send, recv) = tokio::sync::oneshot::channel();
        self.merge_file_listener = Some(recv);
        let popups = popups.clone();
        spawn(rt, async move {
            if let Some(file) = file_dialogue.pick_file().await {
                let contents = file.read().await;
                if send.send(Some(contents)).is_err() {
                    let mut popup_guard = popups.lock().await;
                    popup_guard.push(PopupWindow::new(
                        ErrorSeverity::Warning,
                        "Could not send read file back to main thread.\nThis is likely a bug. \
                         Please report this on github.",
                    ))
                }
            } else if send.send(None).is_err() {
                let mut popup_guard = popups.lock().await;
                popup_guard.push(PopupWindow::new(
                    ErrorSeverity::Warning,
                    "Could not send None back to main thread.\nThis is likely a bug. Please \
                     report this on github.",
                ))
            }
        });
    }

    pub fn update_listeners(&mut self, popups: &Arc<Mutex<Vec<PopupWindow>>>) {
        if let Some(recv) = &mut self.merge_file_listener {
            match recv.try_recv() {
                Ok(contents) => {
                    if let Some(contents) = contents {
                        let save = match SaveData::from_reader(contents.as_slice()) {
                            Ok(s) => s,
                            Err(e) => {
                                let mut popup_guard = popups.blocking_lock();
                                popup_guard.push(PopupWindow::new(
                                    ErrorSeverity::Error,
                                    format!(
                                        "Error reading save file for merge: {e:?}.\nMake sure you \
                                         actually selected a celeste save file.\nIf this \
                                         continues to occur please report it on github."
                                    ),
                                ));
                                return;
                            }
                        };
                        self.save.merge_data(&save);

                        self.level_sets_panel.vanilla_level_set = LevelSetStats {
                            name: "Celeste".to_owned(),
                            areas: self.save.areas.clone(),
                            poem: self.save.poem.clone(),
                            unlocked_areas: self.save.unlocked_areas,
                            total_strawberries: self.save.total_strawberries,
                        };
                    }
                    self.merge_file_listener = None;
                }
                Err(TryRecvError::Closed) => {
                    eprintln!("Sender closed before we got merge contents");
                    self.merge_file_listener = None;
                }
                Err(TryRecvError::Empty) => {}
            }
        }
    }
}
