use celeste_rs::saves::{
    DeathCount,
    Poem,
    StrawberryCount,
    everest::LevelSets,
    util::FileTime,
    vanilla::{AreaMode, Areas},
};
use eframe::egui::{
    CentralPanel,
    Color32,
    DragValue,
    FontId,
    RichText,
    ScrollArea,
    Sense,
    SidePanel,
    TextStyle,
    TopBottomPanel,
    Ui,
    Vec2,
};

use crate::{
    editor::{
        CelesteEditorUiExt,
        EditorTab,
        entity_id_list_widget,
        file_time_widget,
        metadata::show_poem,
    },
    main_menu::LoadableFiles,
};

pub struct LevelSetsTab<'a> {
    vanilla_areas: Option<&'a mut Areas>,
    modded_sets: &'a mut LevelSets,
    modded_sets_recycle_bin: &'a mut LevelSets,
    has_modded_save_data: bool,
    safety_off: bool,
    poem: Option<&'a mut Poem>,
    total_strawberries: Option<&'a mut StrawberryCount>,
    total_deaths: Option<&'a mut DeathCount>,
    time: Option<&'a mut FileTime>,
}

impl<'a> EditorTab<'a> for LevelSetsTab<'a> {
    type EditorData = LevelSetsData;

    fn from_files(
        files: &'a mut [crate::main_menu::LoadableFiles],
        global_data: &'a mut super::GlobalEditorData,
    ) -> Option<Self>
    where
        Self: Sized,
    {
        let mut tab = None;

        for file in files {
            if let LoadableFiles::SaveData(_, save) = file {
                return Some(LevelSetsTab {
                    vanilla_areas: Some(&mut save.areas),
                    modded_sets: &mut save.level_sets,
                    modded_sets_recycle_bin: &mut save.level_set_recycle_bin,
                    has_modded_save_data: save.has_modded_save_data,
                    safety_off: global_data.safety_off,
                    poem: Some(&mut save.poem),
                    total_strawberries: Some(&mut save.total_strawberries),
                    total_deaths: Some(&mut save.total_deaths),
                    time: Some(&mut save.time),
                });
            }

            if let LoadableFiles::ModSaveData(_, mod_data) = file {
                tab = Some(LevelSetsTab {
                    vanilla_areas: None,
                    modded_sets: &mut mod_data.level_sets,
                    modded_sets_recycle_bin: &mut mod_data.level_set_recycle_bin,
                    has_modded_save_data: true,
                    safety_off: global_data.safety_off,
                    poem: None,
                    total_strawberries: None,
                    total_deaths: None,
                    time: None,
                });
            }
        }

        tab
    }

    fn display(
        self,
        ui: &mut Ui,
        data: &mut Self::EditorData,
        _: &tokio::runtime::Runtime,
        _: &std::sync::Arc<tokio::sync::Mutex<Vec<crate::PopupWindow>>>,
    ) -> eframe::egui::Response {
        TopBottomPanel::top("level_sets_info_panel").show_inside(ui, |ui| {
            ui.label(
                RichText::new(
                    "Each level in a level set has an A, B, and C-side in the save file.\nThis \
                     does not mean that the level actually includes 3 different sides. Most \
                     modded maps will only have an A-Side.",
                )
                .weak(),
            );
        });

        if self.has_modded_save_data {
            self.show_modded(ui, data)
        } else {
            self.show_vanilla(ui, data)
        }

        ui.allocate_response(Vec2::new(0.0, 0.0), Sense::focusable_noninteractive())
    }
}

impl LevelSetsTab<'_> {
    pub fn show_modded(self, ui: &mut Ui, data: &mut LevelSetsData) {
        let row_height = ui.text_style_height(&TextStyle::Body);

        let mut total_deaths_buf = 0;
        let mut total_time_buf = FileTime::from_millis(0);

        SidePanel::left("level_sets_list_panel").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Search for a levelset: ");
                ui.text_edit_singleline(&mut data.level_sets_search);
            });

            let search_text = data.level_sets_search.to_ascii_lowercase();

            let row_count = self
                .modded_sets
                .iter()
                .chain(self.modded_sets_recycle_bin.iter())
                .filter(|l| l.name.to_ascii_lowercase().contains(&search_text))
                .count();

            ScrollArea::both().auto_shrink(false).show_rows(
                ui,
                row_height,
                row_count,
                |ui, row_range| {
                    for (idx, (set_idx, level_set)) in self
                        .modded_sets
                        .iter()
                        .chain(self.modded_sets_recycle_bin.iter())
                        .map(|l| &l.name)
                        .enumerate()
                        .filter(|(_, l)| l.to_ascii_lowercase().contains(&search_text))
                        .skip(row_range.start + if self.vanilla_areas.is_none() { 1 } else { 0 })
                        .enumerate()
                    {
                        let adjusted_idx = idx + row_range.start;
                        if adjusted_idx > row_range.end {
                            break;
                        }

                        if ui
                            .selectable_label(data.selected_level_set == Some(set_idx), level_set)
                            .clicked()
                        {
                            data.selected_level_set = Some(set_idx);
                            data.selected_area = None;
                            data.selected_mode = None;
                        }
                    }
                },
            );
        });

        if let Some(set_idx) = data.selected_level_set {
            let (areas, poem) = if set_idx == 0 {
                if let (Some(areas), Some(poem)) = (self.vanilla_areas.as_deref(), self.poem) {
                    (areas, poem)
                } else {
                    let set = &mut self.modded_sets[0];
                    (&set.areas, &mut set.poem)
                }
            } else {
                // Unwrap safe cause selected_leveL_set is always in-bounds
                self.modded_sets
                    .iter_mut()
                    .chain(self.modded_sets_recycle_bin.iter_mut())
                    .nth(set_idx)
                    .map(|s| (&s.areas, &mut s.poem))
                    .unwrap()
            };

            SidePanel::left("area_list_panel").show_inside(ui, |ui| {
                ScrollArea::both().show_rows(ui, row_height, areas.len(), |ui, row_range| {
                    for (idx, area) in areas.iter().enumerate().skip(row_range.start) {
                        if idx > row_range.end {
                            break;
                        }

                        if ui
                            .selectable_label(data.selected_area == Some(idx), area.def.sid())
                            .clicked()
                        {
                            data.selected_area = Some(idx);
                            data.selected_mode = None;
                        }
                    }
                    ui.separator();

                    ScrollArea::horizontal().show(ui, |ui| {
                        show_poem(poem, ui);
                    });
                })
            });


            if let Some(area_idx) = data.selected_area {
                let width = ui
                    .painter()
                    .layout_no_wrap(
                        "A-Side".to_owned(),
                        FontId::proportional(18.0),
                        Color32::BLACK,
                    )
                    .rect
                    .expand(5.0)
                    .width();
                SidePanel::left("mode_list_panel")
                    .max_width(width)
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        for (idx, side) in ["A-Side", "B-Side", "C-Side"].iter().enumerate() {
                            if ui
                                .selectable_label(data.selected_mode == Some(idx), *side)
                                .clicked()
                            {
                                data.selected_mode = Some(idx);
                            }
                        }
                    });

                if let Some(mode_idx) = data.selected_mode {
                    // Unwrap safe cause selected_leveL_set is always in-bounds
                    let set = self
                        .modded_sets
                        .iter_mut()
                        .chain(self.modded_sets_recycle_bin.iter_mut())
                        .nth(set_idx)
                        .unwrap();

                    CentralPanel::default().show_inside(ui, |ui| {
                        // Unwraps safe cause indicies are always kept in-bounds
                        let (area, strawberry_count) = if set_idx == 0 {
                            if let (Some(areas), Some(strawberries)) =
                                (self.vanilla_areas, self.total_strawberries)
                            {
                                (areas.get_mut(area_idx).unwrap(), strawberries)
                            } else {
                                (
                                    set.areas.get_mut(area_idx).unwrap(),
                                    &mut set.total_strawberries,
                                )
                            }
                        } else {
                            (
                                set.areas.get_mut(area_idx).unwrap(),
                                &mut set.total_strawberries,
                            )
                        };

                        let mode = area.modes.get_mut(mode_idx).unwrap();
                        let strawb_count = mode.strawberries.len();

                        let changed = ScrollArea::both()
                            .auto_shrink(false)
                            .show(ui, |ui| {
                                mode_widget(
                                    ui,
                                    area.def.sid(),
                                    self.safety_off,
                                    self.total_deaths.unwrap_or(&mut total_deaths_buf),
                                    self.time.unwrap_or(&mut total_time_buf),
                                    mode,
                                    &mut data.add_strawberry_buff,
                                )
                            })
                            .inner;

                        if changed {
                            match mode.strawberries.len().cmp(&strawb_count) {
                                std::cmp::Ordering::Less => *strawberry_count -= 1,
                                std::cmp::Ordering::Greater => *strawberry_count += 1,
                                std::cmp::Ordering::Equal => {}
                            }
                        }
                    });
                }
            }
        }
    }

    pub fn show_vanilla(self, ui: &mut Ui, data: &mut LevelSetsData) {
        // has_moddded_data is only true when using a SaveData
        // and SaveData *always* has vanilla area data
        let areas = self.vanilla_areas.unwrap();
        SidePanel::left("vanilla_area_panel").show_inside(ui, |ui| {
            ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                for (idx, area_sid) in areas.iter().map(|a| a.def.sid()).enumerate() {
                    if ui
                        .selectable_label(data.selected_area == Some(idx), area_sid)
                        .clicked()
                    {
                        data.selected_area = Some(idx);
                        data.selected_mode = None;
                    }
                }
            });
        });

        if let Some(area_idx) = data.selected_area {
            let width = ui
                .painter()
                .layout_no_wrap(
                    "A-Side".to_owned(),
                    FontId::proportional(18.0),
                    Color32::BLACK,
                )
                .rect
                .expand(5.0)
                .width();
            SidePanel::left("mode_list_panel")
                .max_width(width)
                .resizable(false)
                .show_inside(ui, |ui| {
                    for (idx, side) in ["A-Side", "B-Side", "C-Side"].iter().enumerate() {
                        if ui
                            .selectable_label(data.selected_mode == Some(idx), *side)
                            .clicked()
                        {
                            data.selected_mode = Some(idx);
                        }
                    }
                });

            if let Some(mode_idx) = data.selected_mode {
                CentralPanel::default().show_inside(ui, |ui| {
                    let area = &mut areas[area_idx];

                    mode_widget(
                        ui,
                        area.def.sid(),
                        self.safety_off,
                        // Unwraps safe because we can only call show_vanilla
                        // if we have a full SaveData
                        self.total_deaths.unwrap(),
                        self.time.unwrap(),
                        &mut area.modes[mode_idx],
                        &mut data.add_strawberry_buff,
                    );
                });
            }
        }
    }
}

pub struct LevelSetsData {
    level_sets_search: String,
    selected_level_set: Option<usize>,
    selected_area: Option<usize>,
    selected_mode: Option<usize>,
    add_strawberry_buff: String,
}

impl LevelSetsData {
    pub fn new() -> Self {
        LevelSetsData {
            level_sets_search: String::new(),
            selected_mode: None,
            selected_level_set: None,
            selected_area: None,
            add_strawberry_buff: String::new(),
        }
    }
}

pub fn mode_widget(
    ui: &mut Ui,
    sid: &str,
    safety_off: bool,
    total_deaths: &mut u64,
    total_time: &mut FileTime,
    mode: &mut AreaMode,
    add_strawberry_buff: &mut String,
) -> bool {
    let mut changed = false;
    let stats = &mut mode.stats;
    ui.horizontal(|ui| {
        ui.label("Play Time:");
        let time = stats.time_played;
        changed |= file_time_widget(&mut stats.time_played, ui)
            .response
            .changed();
        if time != stats.time_played {
            if time > stats.time_played {
                total_time.0 -= (time - stats.time_played).0
            } else {
                total_time.0 += (stats.time_played - time).0
            }
        }
    });

    ui.checkbox(&mut stats.completed, "Completed");
    ui.checkbox(&mut stats.single_run_completed, "Completed in one run");
    ui.horizontal(|ui| {
        ui.label("Best Time:");
        changed |= file_time_widget(&mut stats.best_time, ui)
            .response
            .changed()
    });

    ui.checkbox(&mut stats.full_clear, "Full Cleared");
    ui.horizontal(|ui| {
        ui.label("Best Full Clear Time:");
        changed |= file_time_widget(&mut stats.best_full_clear_time, ui)
            .response
            .changed()
    });

    ui.horizontal(|ui| {
        ui.label("Total Strawberries:");
        changed |= ui
            .add_enabled(safety_off, DragValue::new(&mut stats.total_strawberries))
            .changed();
        ui.info_hover(
            "This is updated based off any updates to the strawberries list.\nThis should not be \
             manually edited to avoid desyncing between the count of strawberries and the actual \
             number of strawberries collected.",
        );
    });

    ui.collapsing("Strawberries", |ui| {
        entity_id_list_widget(
            ui,
            &format!("strawberry_{sid}"),
            "Strawberry",
            &mut mode.strawberries,
            safety_off,
            Some(&mut stats.total_strawberries),
            add_strawberry_buff,
        )
    });

    ui.horizontal(|ui| {
        ui.label("Deaths:");
        let deaths = stats.deaths;
        changed |= ui
            .add(DragValue::new(&mut stats.deaths).range(0 ..= i64::MAX))
            .changed();
        if deaths != stats.deaths {
            if deaths > stats.deaths {
                *total_deaths -= deaths - stats.deaths
            } else {
                *total_deaths += stats.deaths - deaths
            }
        }
    });

    ui.horizontal(|ui| {
        ui.label("Best Dashes:");
        changed |= ui.add(DragValue::new(&mut stats.best_dashes)).changed()
    });

    ui.horizontal(|ui| {
        ui.label("Best Deaths:");
        changed |= ui.add(DragValue::new(&mut stats.best_deaths)).changed()
    });

    ui.checkbox(&mut stats.heart_gem, "Heart Collected");

    changed
}
