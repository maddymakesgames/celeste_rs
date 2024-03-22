use std::{io::Cursor, sync::Arc};

use celeste_rs::saves::SaveData;
use eframe::egui::Ui;
use rfd::AsyncFileDialog;
use tokio::{
    runtime::Runtime,
    sync::{
        oneshot::{error::TryRecvError, Receiver, Sender},
        Mutex,
    },
};

use crate::{celeste_save_dir, spawn, ErrorSeverity, PopupWindow};

#[derive(Default)]
pub struct MainMenu {
    #[allow(clippy::type_complexity)]
    file_listener: Option<Receiver<Option<(String, SaveData)>>>,
}

impl MainMenu {
    pub fn display(
        &mut self,
        ui: &mut Ui,
        rt: &Runtime,
        popups: &Arc<Mutex<Vec<PopupWindow>>>,
    ) -> Option<(String, SaveData)> {
        if let Some(recv) = &mut self.file_listener {
            match recv.try_recv() {
                Ok(file) => {
                    self.file_listener = None;
                    return file;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Closed) => {
                    self.file_listener = None;
                    popups.blocking_lock().push(PopupWindow::new(
                        ErrorSeverity::Severe,
                        "file_listner dropped before it sent any signals.\nThis is a bug and a \
                         critical issue. Please make a bug report on github.\nThe program will \
                         now close.",
                    ));
                }
            }

            ui.spinner();
        }

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

        ui.set_enabled(self.file_listener.is_none());
        if ui.button("Open File").clicked() {
            let file_dialogue = AsyncFileDialog::new()
                .add_filter("Celeste Save File", &["celeste"])
                .set_directory(celeste_save_dir().unwrap_or_default());

            let (send, recv) = tokio::sync::oneshot::channel();

            spawn(rt, handle_file_picker(file_dialogue, send, popups.clone()));

            self.file_listener = Some(recv);
        }

        None
    }
}

async fn handle_file_picker(
    file_dialogue: AsyncFileDialog,
    send: Sender<Option<(String, SaveData)>>,
    popups: Arc<Mutex<Vec<PopupWindow>>>,
) {
    let file = file_dialogue.pick_file().await;
    if let Some(file) = file {
        let name = file.file_name();
        let contents = file.read().await;
        drop(file);
        match SaveData::from_reader(Cursor::new(contents)) {
            Ok(save) =>
                if send.send(Some((name, save))).is_err() {
                    popups.lock().await.push(PopupWindow::new(
                        ErrorSeverity::Error,
                        "Error sending data back to main thread.\nThis is a bug, please make a \
                         bug report on github.",
                    ))
                },
            Err(e) => {
                popups.lock().await.push(PopupWindow::new(
                    ErrorSeverity::Error,
                    format!(
                        "Errors found when parsing save file: {e}.\nMake sure the file you \
                         selected is actually a save file.\nIf this continues please report it as \
                         a bug on github."
                    ),
                ));

                if send.send(None).is_err() {
                    popups.lock().await.push(PopupWindow::new(
                        ErrorSeverity::Error,
                        "Error sending data back to main thread.\nThis is a bug, please make a \
                         bug report on github.",
                    ));
                }
            }
        }
    } else if send.send(None).is_err() {
        popups.lock().await.push(PopupWindow::new(
            ErrorSeverity::Error,
            "Error sending data back to main thread.\nThis is a bug, please make a bug report on \
             github.",
        ));
    }
}
