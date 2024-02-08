use eframe::egui::Ui;
use rfd::{AsyncFileDialog, FileHandle};
use tokio::{
    runtime::Runtime,
    sync::mpsc::{error::TryRecvError, UnboundedReceiver},
};

#[derive(Default)]
pub struct MainMenu {
    file_listener: Option<UnboundedReceiver<Option<FileHandle>>>,
}

impl MainMenu {
    pub fn display(&mut self, ui: &mut Ui, rt: &Runtime) -> Option<Vec<u8>> {
        if let Some(recv) = &mut self.file_listener {
            match recv.try_recv() {
                Ok(file) => {
                    if let Some(file) = file {
                        println!("{}", file.file_name());
                        return Some(rt.block_on(async { file.read().await }));
                    } else {
                        println!("File picker closed :(");
                    }

                    self.file_listener = None;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    self.file_listener = None;
                    panic!("FileHandle sender dropped before it sent any signals!")
                }
            }
        }

        ui.set_enabled(self.file_listener.is_none());
        if ui.button("Open File").clicked() {
            let file_dialogue = AsyncFileDialog::new()
                .add_filter("Celeste Save File", &["celeste"])
                .pick_file();

            let (send, recv) = tokio::sync::mpsc::unbounded_channel();

            rt.spawn(async move {
                let file = file_dialogue.await;
                send.send(file)
                    .expect("Error sending file handle back to ui task");
            });

            self.file_listener = Some(recv);
        }

        None
    }
}
