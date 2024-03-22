use celeste_rs::saves::{
    everest::LevelSetStats,
    util::FileTime,
    vanilla::{AreaMode, Modes},
    SaveData,
};
use eframe::egui::{
    CentralPanel,
    CollapsingHeader,
    CollapsingResponse,
    Color32,
    DragValue,
    FontId,
    RichText,
    ScrollArea,
    SidePanel,
    TextStyle,
    TopBottomPanel,
    Ui,
};

use crate::editor::{entity_id_list_widget, file_time_widget, CelesteEditorUiExt};

pub struct LevelSetsPanel {
    level_sets_search: String,
    pub vanilla_level_set: LevelSetStats,
    selected_level_set: Option<usize>,
    selected_area: Option<usize>,
    selected_mode: Option<usize>,
    add_strawberry_buff: String,
}

impl LevelSetsPanel {
    pub fn new(vanilla_level_set: LevelSetStats) -> Self {
        LevelSetsPanel {
            vanilla_level_set,
            level_sets_search: String::new(),
            selected_mode: None,
            selected_level_set: None,
            selected_area: None,
            add_strawberry_buff: String::new(),
        }
    }

    pub fn show(&mut self, ui: &mut Ui, save: &mut SaveData, safety_off: bool) {
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

        let row_height = ui.text_style_height(&TextStyle::Body);

        SidePanel::left("level_sets_list_panel").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Search for a levelset: ");
                ui.text_edit_singleline(&mut self.level_sets_search);
            });

            let search_text = self.level_sets_search.to_ascii_lowercase();

            let row_count = save
                .all_level_sets()
                .iter()
                .filter(|(l, _)| l.name.to_ascii_lowercase().contains(&search_text))
                .count();

            ScrollArea::both().auto_shrink(false).show_rows(
                ui,
                row_height,
                row_count,
                |ui, row_range| {
                    for (idx, (set_idx, level_set)) in save
                        .level_sets
                        .iter()
                        .chain(save.level_set_recycle_bin.iter())
                        .map(|l| &l.name)
                        .enumerate()
                        .filter(|(_, l)| l.to_ascii_lowercase().contains(&search_text))
                        .skip(row_range.start)
                        .enumerate()
                    {
                        let adjusted_idx = idx + row_range.start;
                        if adjusted_idx > row_range.end {
                            break;
                        }

                        if ui
                            .selectable_label(self.selected_level_set == Some(set_idx), level_set)
                            .clicked()
                        {
                            self.selected_level_set = Some(set_idx);
                            self.selected_area = None;
                            self.selected_mode = None;
                        }
                    }
                },
            );
        });

        if let Some(set_idx) = self.selected_level_set {
            let set = if set_idx == 0 {
                &mut self.vanilla_level_set
            } else {
                // Unwrap safe cause selected_leveL_set is always in-bounds
                save.level_sets
                    .iter_mut()
                    .chain(save.level_set_recycle_bin.iter_mut())
                    .nth(set_idx)
                    .unwrap()
            };

            SidePanel::left("area_list_panel").show_inside(ui, |ui| {
                ScrollArea::both().show_rows(ui, row_height, set.areas.len(), |ui, row_range| {
                    for (idx, area) in set.areas.iter().enumerate().skip(row_range.start) {
                        if idx > row_range.end {
                            break;
                        }

                        if ui
                            .selectable_label(self.selected_area == Some(idx), &area.def.sid)
                            .clicked()
                        {
                            self.selected_area = Some(idx);
                            self.selected_mode = None;
                        }
                    }
                })
            });


            if let Some(area_idx) = self.selected_area {
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
                                .selectable_label(self.selected_mode == Some(idx), *side)
                                .clicked()
                            {
                                self.selected_mode = Some(idx);
                            }
                        }
                    });

                if let Some(mode_idx) = self.selected_mode {
                    CentralPanel::default().show_inside(ui, |ui| {
                        // Unwraps safe cause indicies are always kept in-bounds
                        let area = set.areas.get_mut(area_idx).unwrap();
                        let mode = area.modes.get_mut(mode_idx).unwrap();

                        let changed = ScrollArea::both()
                            .auto_shrink(false)
                            .show(ui, |ui| {
                                mode_widget(
                                    ui,
                                    &area.def.sid,
                                    safety_off,
                                    &mut save.total_deaths,
                                    &mut save.time,
                                    mode,
                                    &mut self.add_strawberry_buff,
                                )
                            })
                            .inner;

                        if changed && set_idx == 0 {
                            save.areas = set.areas.clone();
                            save.poem = set.poem.clone();
                            save.total_strawberries = set.total_strawberries;
                        }
                    });
                }
            }
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
            .add(DragValue::new(&mut stats.deaths).clamp_range(0 ..= i64::MAX))
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


#[allow(clippy::too_many_arguments)]
pub fn area_mode_widget(
    ui: &mut Ui,
    name: &str,
    sid: &str,
    safety_off: bool,
    total_deaths: &mut u64,
    total_time: &mut FileTime,
    modes: &mut Modes,
    add_strawberry_buff: &mut String,
) -> CollapsingResponse<bool> {
    let mut changed = false;

    // *Pretty sure* that there can only ever be a, b, and c sides
    // If modes can be of any length we could use .enumerated() and use the index
    // to get the side name "{(idx + 101) as char}-Side"

    ui.collapsing(sid, |ui| {
        for (mode, side_name) in modes.iter_mut().zip(["A-Side", "B-Side", "C-Side"]) {
            let id_name = format!("{name}/{sid}/{side_name}");
            CollapsingHeader::new(RichText::new(side_name))
                .id_source(id_name)
                .show(ui, |ui| {
                    changed |= mode_widget(
                        ui,
                        sid,
                        safety_off,
                        total_deaths,
                        total_time,
                        mode,
                        add_strawberry_buff,
                    );
                });
        }
        changed
    })
}
