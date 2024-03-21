use eframe::egui::Ui;
use rfd::AsyncFileDialog;
use tokio::{
    runtime::Runtime,
    sync::oneshot::{error::TryRecvError, Receiver, Sender},
};

use crate::{celeste_save_dir, spawn};

#[derive(Default)]
pub struct MainMenu {
    #[allow(clippy::type_complexity)]
    file_listener: Option<Receiver<Option<(String, Vec<u8>)>>>,
}

impl MainMenu {
    pub fn display(&mut self, ui: &mut Ui, rt: &Runtime) -> Option<(String, Vec<u8>)> {
        if let Some(recv) = &mut self.file_listener {
            match recv.try_recv() {
                Ok(file) => {
                    self.file_listener = None;
                    return file;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Closed) => {
                    self.file_listener = None;
                    panic!("FileHandle sender dropped before it sent any signals!")
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
                    ui.label("You can find native downloads of the app ");
                    ui.hyperlink_to(
                        "on the github",
                        "https://github.com/maddymakesgames/celeste_rs/releases",
                    );
                })
            })
        }

        ui.set_enabled(self.file_listener.is_none());
        if ui.button("Open File").clicked() {
            let file_dialogue = AsyncFileDialog::new()
                .add_filter("Celeste Save File", &["celeste"])
                .set_directory(celeste_save_dir().unwrap_or_default());

            let (send, recv) = tokio::sync::oneshot::channel();

            spawn(rt, handle_file_picker(file_dialogue, send));

            self.file_listener = Some(recv);
        }

        None
    }
}

async fn handle_file_picker(
    file_dialogue: AsyncFileDialog,
    send: Sender<Option<(String, Vec<u8>)>>,
) {
    let file = file_dialogue.pick_file().await;
    if let Some(file) = file {
        let name = file.file_name();
        let contents = file.read().await;
        drop(file);

        // Expect is fine. Theres not much we can do if this fails
        // And theres no real way to display an error anyway
        send.send(Some((name, contents)))
            .expect("Error sending file handle back to ui task");
    } else {
        // Expect is fine. Theres not much we can do if this fails
        // And theres no real way to display an error anyway
        send.send(None)
            .expect("Error sending file handle back to ui task");
    }
}
