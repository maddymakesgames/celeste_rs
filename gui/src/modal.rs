use eframe::egui::{Context, Id, Modal as RawModal, ModalResponse, Ui, WidgetText};

use std::hash::Hash;

pub struct Modal<R> {
    title: WidgetText,
    description: WidgetText,
    id: Id,
    display_callback: Box<dyn Fn(&mut Ui) -> R>,
}

impl<R> Modal<R> {
    pub fn new(
        id_salt: impl Hash,
        title: impl Into<WidgetText>,
        description: impl Into<WidgetText>,
        callback: impl Fn(&mut Ui) -> R + 'static,
    ) -> Modal<R> {
        Modal::<R> {
            title: title.into(),
            description: description.into(),
            id: Id::new(id_salt),
            display_callback: Box::new(callback),
        }
    }

    pub fn show(&self, ctx: &Context) -> ModalResponse<R> {
        RawModal::new(self.id).show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label(self.title.clone());
                ui.separator();
                ui.label(self.description.clone());
                (self.display_callback)(ui)
            })
            .inner
        })
    }
}
