use std::{io::Cursor, sync::Arc};

use celeste_rs::saves::{
    everest::LevelSetStats,
    ops::DeError,
    util::{EntityID, FileTime},
    SaveData,
};
use eframe::egui::{
    DragValue,
    InnerResponse,
    Response,
    RichText,
    ScrollArea,
    TextStyle,
    Ui,
    Widget,
    WidgetText,
};
use egui_extras::{Column, TableBuilder};
use tokio::{
    runtime::Runtime,
    sync::{oneshot::Receiver, Mutex},
};

pub mod level_sets;
pub mod metadata;
pub mod operations;
pub mod sessions;
pub mod stats;


use crate::{tabbed::TabbedContentWidget, PopupWindow};

pub struct EditorScreen {
    file_name: String,
    save: SaveData,
    safety_off: bool,
    selected_panel: usize,
    level_sets_search: String,
    vanilla_level_set: LevelSetStats,
    merge_file_listener: Option<Receiver<Option<Vec<u8>>>>,
    selected_session_panel: usize,
}

impl EditorScreen {
    pub fn new(file_name: String, bytes: Vec<u8>) -> Result<EditorScreen, DeError> {
        let save = SaveData::from_reader(Cursor::new(bytes))?;
        let vanilla_level_set = LevelSetStats {
            name: "Celeste".to_owned(),
            areas: save.areas.clone(),
            poem: save.poem.clone(),
            unlocked_areas: save.unlocked_areas,
            total_strawberries: save.total_strawberries,
        };

        Ok(EditorScreen {
            file_name,
            save,
            safety_off: false,
            level_sets_search: String::new(),
            vanilla_level_set,
            selected_panel: 0,
            selected_session_panel: 0,
            merge_file_listener: None,
        })
    }

    pub fn display(&mut self, ui: &mut Ui, rt: &Runtime, popups: &Arc<Mutex<Vec<PopupWindow>>>) {
        self.update_listeners(popups);

        let mut selected_panel = self.selected_panel;
        TabbedContentWidget::show(
            ui,
            &mut selected_panel,
            [
                "Metadata",
                "Stats",
                "Assists",
                "Level Sets",
                "Session",
                "Operations",
            ],
            |idx, ui| {
                ScrollArea::vertical().show(ui, |ui| match idx {
                    0 => self.show_metadata(ui),
                    1 => self.show_stats(ui),
                    2 => self.show_assists(ui),
                    3 => self.show_level_sets(ui),
                    4 => self.show_session(ui),
                    5 => self.show_operations(ui, rt, popups),
                    _ => {
                        ui.label("Trying to show an unknown panel. Whoops!");
                    }
                })
            },
        );
        self.selected_panel = selected_panel;
    }
}

fn file_time_widget(filetime: &mut FileTime, ui: &mut Ui) -> InnerResponse<bool> {
    ui.horizontal(|ui| {
        let mut changed = false;
        let (mut hours, mut mins, mut secs, mut millis) = filetime.as_parts();
        changed |= ui.add(DragValue::new(&mut hours)).changed();
        ui.label("hours");
        changed |= ui
            .add(DragValue::new(&mut mins).clamp_range(0 ..= 59))
            .changed();
        ui.label("minutes");
        changed |= ui
            .add(DragValue::new(&mut secs).clamp_range(0 ..= 59))
            .changed();
        ui.label("seconds");
        changed |= ui
            .add(DragValue::new(&mut millis).clamp_range(0 ..= 999))
            .changed();
        ui.label("milliseconds");

        *filetime = FileTime::from_parts(hours, mins, secs, millis);
        changed
    })
}


fn entity_id_list_widget(
    ui: &mut Ui,
    id: &str,
    entity_title: &str,
    entities: &mut Vec<EntityID>,
    safety_off: bool,
    total_entity_count: &mut u8,
    add_entity_buff: &mut String,
) {
    ui.push_id(id, |ui| {
        let mut to_remove = None;
        TableBuilder::new(ui)
            .column(Column::auto())
            .column(Column::remainder())
            .header(18.0, |mut header| {
                header.col(|ui| {
                    ui.label(RichText::new(entity_title).strong());
                });
                header.col(|_ui| {});
            })
            .body(|body| {
                body.rows(18.0, entities.len(), |mut row| {
                    let idx = row.index();
                    row.col(|ui| {
                        ui.label(&entities[idx].key);
                    });
                    row.col(|ui| {
                        if ui.button("remove").clicked() {
                            to_remove = Some(idx);
                        }
                    });
                })
            });

        if let Some(idx) = to_remove {
            entities.remove(idx);
            *total_entity_count -= 1;
        }
    });
    ui.horizontal(|ui| {
        ui.label("Add new entity: ");
        ui.add_enabled_ui(safety_off, |ui| {
            ui.text_edit_singleline(add_entity_buff);
            if ui.button("Add").clicked() {
                entities.push(EntityID {
                    key: std::mem::take(add_entity_buff),
                });
            }
        });
    });
}

trait CelesteEditorRichTextExt {
    fn info(self) -> RichText;
    fn heading2(self) -> RichText;
}

impl CelesteEditorRichTextExt for RichText {
    fn info(self) -> RichText {
        self.text_style(TextStyle::Name("info".into()))
    }

    fn heading2(self) -> RichText {
        self.text_style(TextStyle::Name("header2".into()))
    }
}

trait CelesteEditorUiExt {
    fn info(&mut self, text: impl Into<String>) -> Response;
    fn info_hover(&mut self, text: impl Into<WidgetText>) -> Response;
    fn heading2(&mut self, text: impl Into<String>) -> Response;
    fn drag_value(
        &mut self,
        label: impl Into<WidgetText>,
        value: &mut impl eframe::emath::Numeric,
    ) -> Response;
    fn labeled(&mut self, label: impl Into<WidgetText>, widget: impl Widget) -> Response;
}

impl CelesteEditorUiExt for Ui {
    fn info(&mut self, text: impl Into<String>) -> Response {
        self.label(RichText::new(text).text_style(TextStyle::Name("info".into())))
    }

    fn info_hover(&mut self, text: impl Into<WidgetText>) -> Response {
        self.small("ℹ").on_hover_text(text)
    }

    fn heading2(&mut self, text: impl Into<String>) -> Response {
        self.label(RichText::new(text).text_style(TextStyle::Name("header2".into())))
    }

    fn drag_value(
        &mut self,
        label: impl Into<WidgetText>,
        value: &mut impl eframe::emath::Numeric,
    ) -> Response {
        self.horizontal(|ui| {
            ui.label(label);
            ui.add(DragValue::new(value))
        })
        .response
    }

    fn labeled(&mut self, label: impl Into<WidgetText>, widget: impl Widget) -> Response {
        self.horizontal(|ui| {
            ui.label(label);
            ui.add(widget)
        })
        .response
    }
}
