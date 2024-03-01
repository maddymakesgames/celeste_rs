use eframe::egui::Ui;
use rfd::{AsyncFileDialog, FileHandle};
use tokio::{
    runtime::Runtime,
    sync::oneshot::{error::TryRecvError, Receiver, Sender},
};

#[derive(Default)]
pub struct MainMenu {
    file_listener: Option<Receiver<Option<(String, Vec<u8>)>>>,
}

impl MainMenu {
    pub fn display(&mut self, ui: &mut Ui, rt: &Runtime) -> Option<Vec<u8>> {
        if let Some(recv) = &mut self.file_listener {
            match recv.try_recv() {
                Ok(file) => {
                    if let Some((file_name, contents)) = file {
                        println!("{}", file_name);
                        return Some(contents);
                    } else {
                        println!("File picker closed :(");
                    }

                    self.file_listener = None;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Closed) => {
                    self.file_listener = None;
                    panic!("FileHandle sender dropped before it sent any signals!")
                }
            }
        }

        ui.set_enabled(self.file_listener.is_none());
        if ui.button("Open File").clicked() {
            let file_dialogue =
                AsyncFileDialog::new().add_filter("Celeste Save File", &["celeste"]);

            let (send, recv) = tokio::sync::oneshot::channel();

            #[cfg(not(target_family = "wasm"))]
            rt.spawn(handle_file_picker(file_dialogue, send));
            #[cfg(target_family = "wasm")]
            wasm_bindgen_futures::spawn_local(handle_file_picker(file_dialogue, send));

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

        send.send(Some((name, contents)))
            .expect("Error sending file handle back to ui task");
    } else {
        send.send(None)
            .expect("Error sending file handle back to ui task");
    }
}
