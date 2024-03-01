use std::{
    fs::OpenOptions,
    io::{Cursor, Write},
};

use celeste_rs::saves::{
    everest::LevelSetStats,
    ops::DeError,
    util::FileTime,
    DashMode,
    SaveData,
    VanillaFlags,
};
use eframe::{
    egui::{
        CentralPanel,
        CollapsingHeader,
        CollapsingResponse,
        ComboBox,
        DragValue,
        Frame,
        Id,
        InnerResponse,
        Response,
        RichText,
        ScrollArea,
        SidePanel,
        TextEdit,
        TextStyle,
        TopBottomPanel,
        Ui,
        WidgetText,
    },
    epaint::{vec2, Color32},
};
use tokio::{
    runtime::Runtime,
    sync::oneshot::{error::TryRecvError, Receiver},
};

use crate::{celeste_save_dir, spawn};

pub struct EditorScreen {
    file_name: String,
    save: SaveData,
    safety_off: bool,
    level_sets_search: String,
    vanilla_level_set: LevelSetStats,
    merge_file_listener: Option<Receiver<Option<Vec<u8>>>>,
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
            merge_file_listener: None,
        })
    }

    pub fn display(&mut self, ui: &mut Ui, rt: &Runtime) {
        self.update_listeners();

        TopBottomPanel::top("operations_panel").show_inside(ui, |ui| self.show_operations(ui, rt));

        SidePanel::left("stats_panel").show_inside(ui, |ui| {
            ScrollArea::both()
                .auto_shrink([true, false])
                .show(ui, |ui| {
                    CollapsingHeader::new(RichText::new("Save Metadata").heading2())
                        .default_open(true)
                        .show(ui, |ui| self.show_metadata(ui));
                    CollapsingHeader::new(RichText::new("Flags").heading2())
                        .default_open(false)
                        .show(ui, |ui| self.show_flags(ui));
                    CollapsingHeader::new(RichText::new("Assists").heading2())
                        .default_open(false)
                        .show(ui, |ui| self.show_assists(ui));
                    CollapsingHeader::new(RichText::new("Stats").heading2())
                        .default_open(true)
                        .show(ui, |ui: &mut Ui| self.show_stats(ui));
                });
        });
        CentralPanel::default().show_inside(ui, |ui| {
            ScrollArea::horizontal().show(ui, |ui| self.show_level_sets(ui));
        });
    }

    fn show_operations(&mut self, ui: &mut Ui, rt: &Runtime) {
        ui.vertical(|ui| {
            // TODO: remove most expects from this impl
            ui.horizontal(|ui| {
                if ui.button(RichText::new("Save File").heading2()).clicked() {
                    self.save_file(rt);
                }

                if ui
                    .button(RichText::new("Merge in file").heading2())
                    .clicked()
                {
                    self.merge_file(rt);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Disable Safety Checks:");
                ui.checkbox(&mut self.safety_off, "");
                ui.info_hover(
                    "Check this to enable editing every field.\nThis is off by default as some \
                     values should not be independently edited.\nMake sure you know what you're \
                     doing when you check this.\nYou can hover on a disable item to see why it \
                     might be unsafe.\n(as of alpha version not all tooltips implemented and not \
                     all auto-editing implemented)",
                )
            });
        });
    }

    pub fn show_metadata(&mut self, ui: &mut Ui) {
        let save = &mut self.save;
        ui.horizontal(|ui| {
            ui.label("Save Version: ");
            ui.add_enabled(self.safety_off, TextEdit::singleline(&mut save.version));
            ui.info_hover(
                "Check this to enable editing every field.\nThis is off by default as some values \
                 should not be independently edited.\nMake sure you know what you're doing when \
                 you check this.\nYou can hover on a disable item to see why it might be \
                 unsafe.\n(as of alpha version not all tooltips implemented and not all \
                 independent editing implemented)",
            )
        });
        ui.horizontal(|ui| {
            ui.label("Save Name: ");
            ui.text_edit_singleline(&mut save.name);
        });

        ui.horizontal(|ui| {
            ui.label("Theo's Sister's Name:");
            ui.text_edit_singleline(&mut save.theo_sister_name);
            ui.info_hover(
                "The name of Theo's sister.\nDefaults to 'Alex,' is changed to 'Maddy' if the \
                 player's name is 'Alex.'\nMight not actually update what's in game as this is \
                 stored in the dialogue files too.",
            );
        });

        ui.checkbox(&mut save.has_modded_save_data, "Has modded data");
    }

    pub fn show_assists(&mut self, ui: &mut Ui) {
        let save = &mut self.save;

        ui.horizontal(|ui| {
            ui.label("Cheats Enabled:");
            ui.checkbox(&mut save.cheat_mode, "");
            ui.label("Assists Enabled:");
            ui.checkbox(&mut save.assist_mode, "");
            ui.label("Variants Enabled:");
            ui.checkbox(&mut save.variant_mode, "");
        });

        let assist_editing_enabled = save.assist_mode | save.variant_mode | self.safety_off;

        let assists = &mut save.assists;
        ui.info(
            "You can only edit assists if you have enabled assist mode, variant mode, or have \
             disabled the safety checks.",
        );
        ui.add_enabled_ui(assist_editing_enabled, |ui| {
            ui.horizontal(|ui| {
                ui.label("Game Speed:");
                ui.add(
                    DragValue::new(&mut assists.game_speed)
                        .clamp_range(5 ..= 10)
                        .custom_formatter(|n, _| format!("{}%", (n * 10.0))),
                );
            });
            ui.checkbox(&mut assists.invincible, "Invincible:");
            ui.horizontal(|ui| {
                ui.label("Dash Mode:");
                ComboBox::new("Dash Mode", "")
                    .selected_text(assists.dash_mode.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut assists.dash_mode, DashMode::Normal, "Normal");
                        ui.selectable_value(&mut assists.dash_mode, DashMode::Two, "Two");
                        ui.selectable_value(&mut assists.dash_mode, DashMode::Infinite, "Infinite");
                    });
            });

            ui.checkbox(&mut assists.dash_assist, "Dash Assist");
            ui.checkbox(&mut assists.infinite_stamina, "Infinite Stamina");
            ui.checkbox(&mut assists.mirror_mode, "Mirror Mode");
            ui.checkbox(&mut assists.full_dashing, "360° Dashing");
            ui.checkbox(&mut assists.invisible_motion, "Invisible Motion");
            ui.checkbox(&mut assists.no_grabbing, "No Grabbing");
            ui.checkbox(&mut assists.low_friction, "Low Friction");
            ui.checkbox(&mut assists.super_dash, "Super Dashing");
            ui.checkbox(&mut assists.hiccups, "Hiccups");
            ui.checkbox(&mut assists.badeline, "Play as Badeline");
        });
    }

    pub fn show_stats(&mut self, ui: &mut Ui) {
        let save = &mut self.save;

        ui.horizontal(|ui| {
            ui.label("Total Playtime: ");
            ui.horizontal(|ui| {
                ui.add_enabled_ui(self.safety_off, |ui| file_time_widget(&mut save.time, ui));
                ui.info_hover(
                    "We update this based on modifications in the playtime of individual \
                     levels.\nModifying this means the total playtime of your levels will not be \
                     the same as the displayed file playtime.",
                );
            });
        });

        // If we have 2 verticals wrapped in a horizontal
        // we can have all the DragValues be aligned
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("Total Deaths:");
                ui.label("Vanilla Strawberries:");
                ui.label("Total Golden Strawberries:");
                ui.label("Jump Count:");
                ui.label("Wall Jump Count:");
                ui.label("Dash Count:");
            });
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.add_enabled(self.safety_off, DragValue::new(&mut save.total_deaths));
                    ui.info_hover(
                        "We update this based on any modifications to the death counts of \
                         individual levels.\nModifying this means the death counts of all your \
                         levels won't add up to the total deaths on the save.",
                    );
                });

                ui.horizontal(|ui| {
                    ui.add_enabled(
                        self.safety_off,
                        DragValue::new(&mut save.total_strawberries),
                    );
                    ui.info_hover(
                        "We update the strawberry count based on modifications to the \
                         strawberries in vanilla levels.\nModifying this means the total \
                         strawberry count won't equal the number of vanilla strawberries actually \
                         collected.",
                    );
                });

                // TODO: add tooltip
                ui.horizontal(|ui| {
                    ui.add_enabled(
                        self.safety_off,
                        DragValue::new(&mut save.total_golden_strawberries),
                    );
                    ui.info_hover("TODO")
                });

                ui.horizontal(|ui| {
                    ui.add(DragValue::new(&mut save.total_jumps));
                });

                ui.horizontal(|ui| {
                    ui.add(DragValue::new(&mut save.total_wall_jumps));
                });

                ui.horizontal(|ui| {
                    ui.add(DragValue::new(&mut save.total_dashes));
                });

                ui.horizontal(|ui| {
                    ui.add(DragValue::new(&mut save.unlocked_areas).clamp_range(1 ..= 10));
                });
            });
        });

        ui.checkbox(&mut save.revealed_farewell, "Revealed Farewell");
    }

    pub fn show_flags(&mut self, ui: &mut Ui) {
        let save = &mut self.save;

        let met_theo = save.flags.contains(&VanillaFlags::MetTheo.into());
        let mut met_theo2 = met_theo;
        let theo_knows_name = save.flags.contains(&VanillaFlags::TheoKnowsName.into());
        let mut theo_knows_name2 = theo_knows_name;

        ui.heading("Vanilla Flags");
        ui.checkbox(&mut met_theo2, "Met Theo");
        ui.checkbox(&mut theo_knows_name2, "Theo Knows Name");

        if met_theo && !met_theo2 {
            // We know this will exist because we've checked that the flag is in the vec earlier
            let idx = save
                .flags
                .iter()
                .position(|f| *f == VanillaFlags::MetTheo.into())
                .unwrap();
            save.flags.remove(idx);
        }

        if theo_knows_name && !theo_knows_name2 {
            // We know this will exist because we've checked that the flag is in the vec earlier
            let idx = save
                .flags
                .iter()
                .position(|f| *f == VanillaFlags::TheoKnowsName.into())
                .unwrap();
            save.flags.remove(idx);
        }

        if !met_theo && met_theo2 {
            save.flags.push(VanillaFlags::MetTheo.into());
        }

        if !theo_knows_name && theo_knows_name2 {
            save.flags.push(VanillaFlags::TheoKnowsName.into());
        }

        ui.horizontal(|ui| {
            ui.label("Poem order: ");
            // Drag and drop code adapted from https://github.com/emilk/egui/blob/master/crates/egui_demo_lib/src/demo/drag_and_drop.rs

            // If there is a drop, store the location of the item being dragged, and the destination for the drop.
            let mut from = None;
            let mut to = None;

            ui.dnd_drop_zone::<usize>(Frame::default().inner_margin(4.0), |ui| {
                ui.set_min_size(vec2(64.0, 32.0));

                for (idx, entry) in save.poem.iter().enumerate() {
                    let item_id = Id::new(("poem_drag_drop", idx));

                    let response = ui
                        .dnd_drag_source(item_id, idx, |ui| {
                            ui.label(RichText::new(entry).strong());
                        })
                        .response;

                    if let (Some(pointer), Some(hovered_payload)) = (
                        ui.input(|i| i.pointer.interact_pos()),
                        response.dnd_hover_payload::<usize>(),
                    ) {
                        let rect = response.rect;

                        // Preview insertion
                        let stroke = eframe::egui::Stroke::new(1.0, Color32::WHITE);

                        let insert_idx = if *hovered_payload == idx {
                            // We are dragged onto ourselves
                            ui.painter().vline(rect.center().x, rect.y_range(), stroke);
                            idx
                        } else if pointer.x < rect.center().x {
                            // Above us
                            ui.painter().vline(rect.left(), rect.y_range(), stroke);

                            idx.saturating_sub(1)
                        } else {
                            // Below us
                            ui.painter().vline(rect.right(), rect.y_range(), stroke);
                            idx
                        };

                        if let Some(dragged_payload) = response.dnd_release_payload::<usize>() {
                            // The user dropped onto this item.
                            from = Some(dragged_payload);
                            to = Some(insert_idx);
                        }
                    }
                }
            });

            if let (Some(from), Some(mut to)) = (from, to) {
                let item = save.poem.remove(*from);

                to = to.min(save.poem.len());
                save.poem.insert(to, item);
            }
        });

        // TODO: add summit gems
    }

    pub fn show_level_sets(&mut self, ui: &mut Ui) {
        let save = &mut self.save;

        ui.heading2("Level Sets");
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

        ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            if ("celeste".contains(&search_text) || "vanilla".contains(&search_text))
                && level_set_widget(ui, self.safety_off, &mut self.vanilla_level_set)
                    .body_returned
                    .unwrap_or_default()
            {
                save.areas = self.vanilla_level_set.areas.clone();
                save.poem = self.vanilla_level_set.poem.clone();
                save.total_strawberries = self.vanilla_level_set.total_strawberries;
            }

            for (level_set, _) in save
                .all_level_sets_mut()
                .into_iter()
                .filter(|(l, _)| l.name.to_ascii_lowercase().contains(&search_text))
            {
                if level_set.name == "Celeste" {
                    continue;
                }
                level_set_widget(ui, self.safety_off, level_set);
            }
        });
    }

    fn save_file(&self, rt: &Runtime) {
        let file_dialogue = rfd::AsyncFileDialog::new().set_file_name(&self.file_name);
        let serialized = self.save.to_string().expect("Error serializing file");

        spawn(rt, async move {
            if let Some(file) = file_dialogue.save_file().await {
                #[cfg(not(target_family = "wasm"))]
                {
                    let mut file = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .open(file.path())
                        .expect("Error opening file");

                    file.write_all(serialized.as_bytes())
                        .expect("Error writing to file");
                }
                #[cfg(target_family = "wasm")]
                {
                    file.write(serialized.as_bytes())
                        .await
                        .expect("Error writing to file");
                }
            }
        });
    }

    fn merge_file(&mut self, rt: &Runtime) {
        let file_dialogue = rfd::AsyncFileDialog::new()
            .add_filter("Celeste Save File", &["celeste"])
            .set_directory(celeste_save_dir().unwrap_or_default());

        let (send, recv) = tokio::sync::oneshot::channel();
        self.merge_file_listener = Some(recv);
        spawn(rt, async move {
            if let Some(file) = file_dialogue.pick_file().await {
                let contents = file.read().await;
                send.send(Some(contents))
                    .expect("Error sending to reciever");
            } else {
                send.send(None).expect("Error sending to reciever");
            }
        });
    }

    fn update_listeners(&mut self) {
        if let Some(recv) = &mut self.merge_file_listener {
            match recv.try_recv() {
                Ok(contents) => {
                    if let Some(contents) = contents {
                        let save = SaveData::from_reader(contents.as_slice())
                            .expect("Invalid file provided to merge operation");
                        self.save.merge_data(&save);

                        self.vanilla_level_set = LevelSetStats {
                            name: "Celeste".to_owned(),
                            areas: self.save.areas.clone(),
                            poem: self.save.poem.clone(),
                            unlocked_areas: self.save.unlocked_areas,
                            total_strawberries: self.save.total_strawberries,
                        };
                    }
                    self.merge_file_listener = None;
                }
                Err(TryRecvError::Closed) => {
                    eprintln!("Sender closed before we got merge contents");
                    self.merge_file_listener = None;
                }
                Err(TryRecvError::Empty) => {}
            }
        }
    }
}

fn file_time_widget(filetime: &mut FileTime, ui: &mut Ui) -> InnerResponse<bool> {
    ui.horizontal(|ui| {
        let mut changed = false;
        let (mut hours, mut mins, mut secs, mut millis) = filetime.as_parts();
        changed |= ui.add(DragValue::new(&mut hours)).changed();
        ui.label("hours");
        changed |= ui.add(DragValue::new(&mut mins)).changed();
        ui.label("minutes");
        changed |= ui.add(DragValue::new(&mut secs)).changed();
        ui.label("seconds");
        changed |= ui.add(DragValue::new(&mut millis)).changed();
        ui.label("milliseconds");

        *filetime = FileTime::from_parts(hours, mins, secs, millis);
        changed
    })
}

fn level_set_widget(
    ui: &mut Ui,
    safety_off: bool,
    level_set: &mut LevelSetStats,
) -> CollapsingResponse<bool> {
    ui.collapsing(&level_set.name, |ui| {
        let mut changed = false;
        let name = level_set.name.clone();
        for area in level_set.areas.iter_mut() {
            // *Pretty sure* that there can only ever be a, b, and c sides
            // But this should work for extensions.
            // If modes can be of any length we could use .enumerated() and use the index
            // to get the side name "{(idx + 101) as char}-Side"
            let sid = area.def.sid.clone();
            ui.collapsing(&area.def.sid, |ui| {
                for (mode, side_name) in area
                    .modes
                    .iter_mut()
                    .zip(["A-Side", "B-Side", "C-Side", "D-Side", "E-Side"])
                {
                    let stats = &mut mode.stats;
                    let id_name = format!("{name}/{sid}/{side_name}");
                    CollapsingHeader::new(RichText::new(side_name))
                        .id_source(id_name)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Play Time:");
                                changed |= file_time_widget(&mut stats.time_played, ui)
                                    .response
                                    .changed()
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
                                    .add_enabled(
                                        safety_off,
                                        DragValue::new(&mut stats.total_strawberries),
                                    )
                                    .changed();
                                ui.info_hover("TODO");
                            });

                            ui.horizontal(|ui| {
                                ui.label("Deaths:");
                                changed |= ui.add(DragValue::new(&mut stats.deaths)).changed()
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
            });
        }

        changed
    })
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
}
