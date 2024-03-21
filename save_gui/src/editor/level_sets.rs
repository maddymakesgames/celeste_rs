use celeste_rs::saves::{everest::LevelSetStats, util::FileTime, vanilla::Modes};
use eframe::egui::{
    CollapsingHeader,
    CollapsingResponse,
    DragValue,
    RichText,
    ScrollArea,
    TextStyle,
    Ui,
};

use crate::editor::{entity_id_list_widget, file_time_widget, CelesteEditorUiExt, EditorScreen};

impl EditorScreen {
    pub fn show_level_sets(&mut self, ui: &mut Ui) {
        let save = &mut self.save;
        ui.separator();
        ui.label(
            RichText::new(
                "Each level in a level set has an a, b, and c-side in the save file.\nThis does \
                 not mean that the level actually includes 3 different sides. So make sure you \
                 know what you're editing before you change anything.",
            )
            .weak(),
        );

        ui.horizontal(|ui| {
            ui.label("Search for a levelset: ");
            ui.text_edit_singleline(&mut self.level_sets_search);
        });

        let search_text = self.level_sets_search.to_ascii_lowercase();

        let row_height = ui.text_style_height(&TextStyle::Body);
        let row_count = save
            .all_level_sets()
            .iter()
            .filter(|(l, _)| l.name.to_ascii_lowercase().contains(&search_text))
            .count();
        // This isn't necessarily the correct way to implement this *but* its probably good enough to work.
        ScrollArea::vertical().auto_shrink(false).show_rows(
            ui,
            row_height,
            row_count,
            |ui, row_range| {
                let mut range_start = row_range.start;
                if range_start == 0
                    && ("celeste".contains(&search_text) || "vanilla".contains(&search_text))
                {
                    let widget_return = level_set_widget(
                        ui,
                        self.safety_off,
                        &mut save.total_deaths,
                        &mut save.time,
                        &mut self.vanilla_level_set,
                    )
                    .body_returned
                    .unwrap_or_default();
                    range_start = 1;
                    if widget_return {
                        save.areas = self.vanilla_level_set.areas.clone();
                        save.poem = self.vanilla_level_set.poem.clone();
                        save.total_strawberries = self.vanilla_level_set.total_strawberries;
                    }
                }

                // BUG:
                // Because of how show_rows works this fucks everything up when something is open
                // Specifically we'll skip an entry if the level set id is scrolled past
                // when we want more complex logic to skip over it when it's open.
                // This is too much for me to think about right now and while it's bad
                // for testing and an initial release its pretty much functional
                //
                // The biggest issue is that we do want to keep using show_rows (or show_viewport)
                // due to how large the level_sets list can get
                for (idx, level_set) in save
                    .level_sets
                    .iter_mut()
                    .chain(save.level_set_recycle_bin.iter_mut())
                    .filter(|l| l.name.to_ascii_lowercase().contains(&search_text))
                    .skip(range_start)
                    .enumerate()
                {
                    if idx > row_range.end {
                        break;
                    }

                    if level_set.name == "Celeste" {
                        continue;
                    }

                    level_set_widget(
                        ui,
                        self.safety_off,
                        &mut save.total_deaths,
                        &mut save.time,
                        level_set,
                    );
                }
            },
        );
    }
}

pub fn level_set_widget(
    ui: &mut Ui,
    safety_off: bool,
    total_deaths: &mut u64,
    total_time: &mut FileTime,
    level_set: &mut LevelSetStats,
) -> CollapsingResponse<bool> {
    ui.collapsing(&level_set.name, |ui| {
        let mut changed = false;
        let name = level_set.name.clone();
        for area in level_set.areas.iter_mut() {
            changed |= area_mode_widget(
                ui,
                &name,
                &area.def.sid,
                safety_off,
                total_deaths,
                total_time,
                &mut area.modes,
            )
            .body_returned
            .unwrap_or_default();
        }

        changed
    })
}

pub fn area_mode_widget(
    ui: &mut Ui,
    name: &str,
    sid: &str,
    safety_off: bool,
    total_deaths: &mut u64,
    total_time: &mut FileTime,
    modes: &mut Modes,
) -> CollapsingResponse<bool> {
    let mut changed = false;

    // *Pretty sure* that there can only ever be a, b, and c sides
    // But this should work for extensions.
    // If modes can be of any length we could use .enumerated() and use the index
    // to get the side name "{(idx + 101) as char}-Side"

    ui.collapsing(sid, |ui| {
        for (mode, side_name) in modes
            .iter_mut()
            .zip(["A-Side", "B-Side", "C-Side", "D-Side", "E-Side"])
        {
            let id_name = format!("{name}/{sid}/{side_name}");
            CollapsingHeader::new(RichText::new(side_name))
                .id_source(id_name)
                .show(ui, |ui| {
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
                        ui.info_hover("TODO");
                    });

                    ui.collapsing("Strawberries", |ui| {
                        entity_id_list_widget(
                            ui,
                            &format!("strawberry_{sid}_{side_name}"),
                            "Strawberry",
                            &mut mode.strawberries,
                            safety_off,
                            &mut stats.total_strawberries,
                            &mut String::new(),
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
                });
        }
        changed
    })
}
