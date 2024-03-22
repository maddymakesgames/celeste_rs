use celeste_rs::saves::{DashMode, VanillaFlags};
use eframe::egui::{vec2, Color32, ComboBox, DragValue, Frame, Id, RichText, TextEdit, Ui};

use crate::editor::{CelesteEditorUiExt, EditorScreen};

impl EditorScreen {
    pub fn show_metadata(&mut self, ui: &mut Ui) {
        let save = &mut self.save;
        ui.horizontal(|ui| {
            ui.label("Save Version: ");
            ui.add_enabled(self.safety_off, TextEdit::singleline(&mut save.version));
            ui.info_hover(
                "Modifying this could make the save not load if the game version you try to load \
                 the save with doesn't match this.",
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

        self.show_flags(ui);
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

            ui.dnd_drop_zone::<usize, _>(Frame::default().inner_margin(4.0), |ui| {
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
}
