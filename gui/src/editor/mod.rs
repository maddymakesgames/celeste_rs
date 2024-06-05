use std::sync::Arc;

use celeste_rs::saves::{
    util::{EntityID, FileTime},
    StrawberryCount,
};
use eframe::egui::{
    scroll_area::ScrollBarVisibility,
    DragValue,
    InnerResponse,
    Response,
    RichText,
    ScrollArea,
    Sense,
    TextStyle,
    Ui,
    Vec2,
    Widget,
    WidgetText,
};
use egui_extras::{Column, TableBuilder};
use tokio::{runtime::Runtime, sync::Mutex};

pub mod level_sets;
pub mod metadata;
pub mod operations;
pub mod sessions;
pub mod stats;


use crate::{
    editor::{
        level_sets::{LevelSetsData, LevelSetsTab},
        metadata::{AssistsTab, MetadataTab},
        operations::{OperationsData, OperationsTab},
        sessions::{SessionsData, SessionsTab},
        stats::StatsTab,
    },
    main_menu::LoadableFiles,
    tabbed::TabbedContentWidget,
    PopupWindow,
};

pub struct EditorScreen {
    file_name: String,
    pub files: Vec<LoadableFiles>,
    global_data: GlobalEditorData,
    tab_data: Vec<EditorTabData>,
    selected_panel: usize,
}

impl EditorScreen {
    pub fn name(&self) -> &str {
        &self.file_name
    }

    pub fn new(file_name: String, base_file: LoadableFiles) -> EditorScreen {
        EditorScreen {
            file_name: file_name
                .split('.')
                .next()
                .unwrap_or("Unnamed File")
                .to_owned(),
            files: vec![base_file],
            tab_data: EditorTabData::data_vec(),
            global_data: GlobalEditorData { safety_off: false },
            selected_panel: 0,
        }
    }

    pub fn display(&mut self, ui: &mut Ui, rt: &Runtime, popups: &Arc<Mutex<Vec<PopupWindow>>>) {
        ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Loaded Files:");
                for file in &self.files {
                    ui.label(file.file_name());
                }
            });
        });

        let mut selected_panel = self.selected_panel;

        let mut tabs = EditorTabContainer::tab_array()
            .into_iter()
            .filter(|t| t.is_constructable(&mut self.files, &mut self.global_data))
            .collect::<Vec<_>>();

        let tab_names = tabs
            .iter()
            .map(EditorTabContainer::name)
            .collect::<Vec<_>>();


        TabbedContentWidget::show(
            ui,
            &mut selected_panel,
            tab_names,
            ScrollBarVisibility::AlwaysHidden,
            TextStyle::Name("header2".into()),
            |idx, ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    let mut tab = tabs.remove(idx);
                    tab.load_from_files(&mut self.files, &mut self.global_data);
                    let data = EditorTabData::get_data(&mut self.tab_data, &tab);

                    tab.display(ui, data, rt, popups)
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
    total_entity_count: Option<&mut StrawberryCount>,
    add_entity_buff: &mut String,
) {
    ui.push_id(id, |ui| {
        let mut to_remove = None;
        let text_size = ui.text_style_height(&TextStyle::Body);
        TableBuilder::new(ui)
            .column(Column::auto().resizable(true))
            .column(Column::remainder())
            .header(text_size, |mut header| {
                header.col(|ui| {
                    ui.label(RichText::new(format!("{entity_title} ID")).strong());
                });
                header.col(|_ui| {});
            })
            .body(|body| {
                body.rows(text_size, entities.len(), |mut row| {
                    let idx = row.index();
                    row.col(|ui| {
                        ui.style_mut().wrap = Some(false);
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
            if let Some(counter) = total_entity_count {
                entities.remove(idx);
                *counter -= 1;
            }
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
        ui.info_hover("The Entity ID needs to be for an entity that actually exists in the level.");
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

pub struct GlobalEditorData {
    safety_off: bool,
}

enum EditorTabContainer<'a> {
    Metadata(Option<MetadataTab<'a>>),
    Stats(Option<StatsTab<'a>>),
    Assists(Option<AssistsTab<'a>>),
    LevelSets(Option<LevelSetsTab<'a>>),
    Sessions(Option<SessionsTab<'a>>),
    Operations(Option<OperationsTab<'a>>),
}

enum EditorTabData {
    Metadata(()),
    Stats(()),
    Assists(()),
    LevelSets(LevelSetsData),
    Sessions(SessionsData),
    Operations(OperationsData),
}

impl EditorTabData {
    pub fn data_vec() -> Vec<EditorTabData> {
        vec![
            EditorTabData::Metadata(()),
            EditorTabData::Stats(()),
            EditorTabData::Assists(()),
            EditorTabData::LevelSets(LevelSetsData::new()),
            EditorTabData::Sessions(SessionsData::new()),
            EditorTabData::Operations(OperationsData::new()),
        ]
    }

    pub fn get_data<'a>(
        data: &'a mut Vec<EditorTabData>,
        tab: &EditorTabContainer,
    ) -> &'a mut EditorTabData {
        for data in data {
            match (tab, &data) {
                (EditorTabContainer::Metadata(_), EditorTabData::Metadata(_))
                | (EditorTabContainer::Stats(_), EditorTabData::Stats(_))
                | (EditorTabContainer::Assists(_), EditorTabData::Assists(_))
                | (EditorTabContainer::LevelSets(_), EditorTabData::LevelSets(_))
                | (EditorTabContainer::Sessions(_), EditorTabData::Sessions(_))
                | (EditorTabContainer::Operations(_), EditorTabData::Operations(_)) => return data,
                _ => {}
            }
        }
        unreachable!("EditorTabData vec doesn't have an entry for one of the tabs")
    }
}


impl<'a> EditorTabContainer<'a> {
    pub fn tab_array() -> [Self; 6] {
        [
            EditorTabContainer::Metadata(None),
            EditorTabContainer::Stats(None),
            EditorTabContainer::Assists(None),
            EditorTabContainer::LevelSets(None),
            EditorTabContainer::Sessions(None),
            EditorTabContainer::Operations(None),
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            EditorTabContainer::Metadata(_) => "Metadata",
            EditorTabContainer::Stats(_) => "Stats",
            EditorTabContainer::Assists(_) => "Assists",
            EditorTabContainer::LevelSets(_) => "Level Sets",
            EditorTabContainer::Sessions(_) => "Sessions",
            EditorTabContainer::Operations(_) => "Operations",
        }
    }

    pub fn load_from_files(
        &mut self,
        files: &'a mut [LoadableFiles],
        global_data: &'a mut GlobalEditorData,
    ) {
        match self {
            EditorTabContainer::Metadata(m) => *m = MetadataTab::from_files(files, global_data),
            EditorTabContainer::Stats(s) => *s = StatsTab::from_files(files, global_data),
            EditorTabContainer::Assists(a) => *a = AssistsTab::from_files(files, global_data),
            EditorTabContainer::LevelSets(l) => *l = LevelSetsTab::from_files(files, global_data),
            EditorTabContainer::Sessions(s) => *s = SessionsTab::from_files(files, global_data),
            EditorTabContainer::Operations(o) => *o = OperationsTab::from_files(files, global_data),
        }
    }

    fn display(
        self,
        ui: &mut Ui,
        data: &mut EditorTabData,
        rt: &Runtime,
        popups: &Arc<Mutex<Vec<PopupWindow>>>,
    ) -> Response {
        match self {
            EditorTabContainer::Metadata(m) =>
                if let (Some(m), EditorTabData::Metadata(d)) = (m, data) {
                    return m.display(ui, d, rt, popups);
                },
            EditorTabContainer::Stats(s) =>
                if let (Some(s), EditorTabData::Stats(d)) = (s, data) {
                    return s.display(ui, d, rt, popups);
                },
            EditorTabContainer::Assists(a) =>
                if let (Some(a), EditorTabData::Assists(d)) = (a, data) {
                    return a.display(ui, d, rt, popups);
                },
            EditorTabContainer::LevelSets(l) =>
                if let (Some(l), EditorTabData::LevelSets(d)) = (l, data) {
                    return l.display(ui, d, rt, popups);
                },
            EditorTabContainer::Sessions(s) =>
                if let (Some(s), EditorTabData::Sessions(d)) = (s, data) {
                    return s.display(ui, d, rt, popups);
                },
            EditorTabContainer::Operations(o) =>
                if let (Some(o), EditorTabData::Operations(d)) = (o, data) {
                    return o.display(ui, d, rt, popups);
                },
        }

        ui.allocate_response(Vec2::new(0.0, 0.0), Sense::focusable_noninteractive())
    }

    fn is_constructable(
        &self,
        files: &'a mut [LoadableFiles],
        global_data: &'a mut GlobalEditorData,
    ) -> bool {
        match self {
            EditorTabContainer::Metadata(_) =>
                MetadataTab::from_files(files, global_data).is_some(),
            EditorTabContainer::Stats(_) => StatsTab::from_files(files, global_data).is_some(),
            EditorTabContainer::Assists(_) => AssistsTab::from_files(files, global_data).is_some(),
            EditorTabContainer::LevelSets(_) =>
                LevelSetsTab::from_files(files, global_data).is_some(),
            EditorTabContainer::Sessions(_) =>
                SessionsTab::from_files(files, global_data).is_some(),
            EditorTabContainer::Operations(_) =>
                OperationsTab::from_files(files, global_data).is_some(),
        }
    }
}

pub trait EditorTab<'a> {
    type EditorData;

    fn from_files(
        files: &'a mut [LoadableFiles],
        global_data: &'a mut GlobalEditorData,
    ) -> Option<Self>
    where
        Self: Sized;
    fn display(
        self,
        ui: &mut Ui,
        data: &mut Self::EditorData,
        rt: &Runtime,
        popups: &Arc<Mutex<Vec<PopupWindow>>>,
    ) -> Response;
}

#[allow(dead_code)]
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
