use std::sync::Arc;

use celeste_rs::saves::{
    mods::{ParsedModSave, ParsedModSession, ParsedModSetting},
    ModSaveData,
    SaveData,
};
use eframe::egui::{RichText, Ui};
use tokio::{
    runtime::Runtime,
    sync::{
        oneshot::{error::TryRecvError, Receiver},
        Mutex,
    },
};

use crate::{
    celeste_save_dir,
    editor::{CelesteEditorRichTextExt, CelesteEditorUiExt, EditorTab, GlobalEditorData},
    main_menu::LoadableFiles,
    spawn,
    ErrorSeverity,
    PopupWindow,
};


pub struct OperationsTab<'a> {
    files: &'a mut [LoadableFiles],
    global_data: &'a mut GlobalEditorData,
    loaded_save_data: bool,
}

pub struct OperationsData {
    merge_file_listener: Option<Receiver<Option<Vec<u8>>>>,
}

impl OperationsData {
    pub fn new() -> OperationsData {
        OperationsData {
            merge_file_listener: None,
        }
    }
}

impl<'a> EditorTab<'a> for OperationsTab<'a> {
    type EditorData = OperationsData;

    fn from_files(
        files: &'a mut [crate::main_menu::LoadableFiles],
        global_data: &'a mut GlobalEditorData,
    ) -> Option<Self>
    where
        Self: Sized,
    {
        // We loop to make sure we have a SaveData
        // because we need that for some operations
        let mut loaded_save = None;
        for file in &mut *files {
            if let LoadableFiles::SaveData(_, save) = file {
                loaded_save = Some(save);
            }
        }

        Some(OperationsTab {
            loaded_save_data: loaded_save.is_some(),
            files,
            global_data,
        })
    }

    fn display(
        mut self,
        ui: &mut Ui,
        data: &mut Self::EditorData,
        rt: &Runtime,
        popups: &Arc<Mutex<Vec<PopupWindow>>>,
    ) -> eframe::egui::Response {
        self.update_listeners(data, popups);

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                if ui.button(RichText::new("Save File").info()).clicked() {
                    self.save_files(rt, popups);
                }

                if self.loaded_save_data {
                    if ui.button(RichText::new("Merge in file").info()).clicked() {
                        self.merge_file(data, rt, popups);
                    }

                    ui.info_hover(
                        "Merges in any applicable data from a different save file into this \
                         one.\nWhile this has been tested, it might not merge all the data you \
                         would want it to and there may still be bugs.\nIt is highly recommended \
                         you keep backups of your saves before using this.\n\nHuge note: this \
                         DOES NOT really merge in golden strawberry data. With only the save file \
                         there is no way to tell if a strawberry is a golden or not, and so we \
                         cannot properly adjust the counts.\nIf you have a save with a lot of \
                         goldens and want the merged count to be accurate, you'll need to \
                         manually adjust the golden count.",
                    );
                }
            });

            ui.horizontal(|ui| {
                ui.label("Disable Safety Checks:");
                ui.checkbox(&mut self.global_data.safety_off, "");
                ui.info_hover(
                    "Check this to enable editing every field.\nThis is off by default as some \
                     values should not be independently edited.\nMake sure you know what you're \
                     doing when you check this.\nYou can hover on a disable item to see why it \
                     might be unsafe.\n(as of alpha version not all tooltips implemented and not \
                     all auto-editing implemented)",
                )
            });
        })
        .response
    }
}

impl<'a> OperationsTab<'a> {
    fn save_files(&self, rt: &Runtime, popups: &Arc<Mutex<Vec<PopupWindow>>>) {
        for file in self.files.iter() {
            match file {
                LoadableFiles::SaveData(file_name, save_data) =>
                    OperationsTab::save_save_data(save_data, file_name, rt, popups),
                LoadableFiles::ModSaveData(file_name, mod_save_data) =>
                    OperationsTab::save_mod_save_data(mod_save_data, file_name, rt, popups),
                LoadableFiles::ModSave(file_name, mod_save) => OperationsTab::save_yaml_file(
                    mod_save,
                    ParsedModSave::to_writer,
                    file_name,
                    rt,
                    popups,
                ),
                LoadableFiles::ModSession(file_name, mod_session) => OperationsTab::save_yaml_file(
                    mod_session,
                    ParsedModSession::to_writer,
                    file_name,
                    rt,
                    popups,
                ),
                LoadableFiles::ModSetting(file_name, mod_setting) => OperationsTab::save_yaml_file(
                    mod_setting,
                    ParsedModSetting::to_writer,
                    file_name,
                    rt,
                    popups,
                ),
            }
        }
    }

    fn save_file(
        data: String,
        file_name: &str,
        rt: &Runtime,
        popups: &Arc<Mutex<Vec<PopupWindow>>>,
    ) {
        let popups = popups.clone();
        let file_dialogue = rfd::AsyncFileDialog::new().set_file_name(file_name);
        spawn(rt, async move {
            if let Some(file) = file_dialogue.save_file().await {
                #[cfg(not(target_family = "wasm"))]
                {
                    use std::{fs::OpenOptions, io::Write};
                    let mut file = match OpenOptions::new()
                        .create(true)
                        .truncate(true)
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

                    if let Err(e) = file.write_all(data.as_bytes()) {
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

    fn save_save_data(
        save_data: &SaveData,
        file_name: &str,
        rt: &Runtime,
        popups: &Arc<Mutex<Vec<PopupWindow>>>,
    ) {
        let serialized = match save_data.to_string() {
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

        OperationsTab::save_file(serialized, file_name, rt, popups);
    }

    fn save_mod_save_data(
        mod_save_data: &ModSaveData,
        file_name: &str,
        rt: &Runtime,
        popups: &Arc<Mutex<Vec<PopupWindow>>>,
    ) {
        let serialized = match mod_save_data.to_string() {
            Ok(s) => s,
            Err(e) => {
                let mut popup_guard = popups.blocking_lock();
                popup_guard.push(PopupWindow::new(
                    ErrorSeverity::Error,
                    format!(
                        "Error serializing modsavedata file: {e:?}.\nThis is likely a bug. Please \
                         report it on github."
                    ),
                ));
                return;
            }
        };

        OperationsTab::save_file(serialized, file_name, rt, popups);
    }

    fn save_yaml_file<T, E: std::fmt::Debug>(
        file: &T,
        to_writer_func: impl Fn(&T, &mut String) -> Result<(), E>,
        file_name: &str,
        rt: &Runtime,
        popups: &Arc<Mutex<Vec<PopupWindow>>>,
    ) {
        let mut buf = String::new();
        if let Err(e) = to_writer_func(file, &mut buf) {
            let mut popup_guard = popups.blocking_lock();
            popup_guard.push(PopupWindow::new(
                ErrorSeverity::Error,
                format!(
                    "Error serializing save file: {e:?}.\nThis is likely a bug. Please report it \
                     on github."
                ),
            ));
        } else {
            OperationsTab::save_file(buf, file_name, rt, popups);
        }
    }

    fn merge_file(
        &mut self,
        data: &mut OperationsData,
        rt: &Runtime,
        popups: &Arc<Mutex<Vec<PopupWindow>>>,
    ) {
        let file_dialogue = rfd::AsyncFileDialog::new()
            .add_filter("Celeste Save File", &["celeste"])
            .set_directory(celeste_save_dir().unwrap_or_default());

        let (send, recv) = tokio::sync::oneshot::channel();
        data.merge_file_listener = Some(recv);
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

    pub fn update_listeners(
        &mut self,
        data: &mut OperationsData,
        popups: &Arc<Mutex<Vec<PopupWindow>>>,
    ) {
        if let Some(recv) = &mut data.merge_file_listener {
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

                        let mut self_save = None;

                        for file in &mut *self.files {
                            if let LoadableFiles::SaveData(_, save) = file {
                                self_save = Some(save);
                            }
                        }

                        if let Some(self_save) = self_save {
                            self_save.merge_data(&save);
                        }
                    }
                    data.merge_file_listener = None;
                }
                Err(TryRecvError::Closed) => {
                    eprintln!("Sender closed before we got merge contents");
                    data.merge_file_listener = None;
                }
                Err(TryRecvError::Empty) => {}
            }
        }
    }
}
