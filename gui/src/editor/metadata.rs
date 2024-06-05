use std::sync::Arc;

use celeste_rs::saves::{Assists, DashMode, Flags, Poem, VanillaFlags};
use eframe::egui::{vec2, Color32, ComboBox, DragValue, Frame, Id, RichText, TextEdit, Ui};
use tokio::{runtime::Runtime, sync::Mutex};

use crate::{
    editor::{CelesteEditorUiExt, EditorTab, GlobalEditorData},
    main_menu::LoadableFiles,
    PopupWindow,
};

pub struct MetadataTab<'a> {
    safety_off: &'a mut bool,
    version: &'a mut String,
    name: &'a mut String,
    theo_sister_name: &'a mut String,
    has_modded_save_data: &'a mut bool,
    flags: &'a mut Flags,
    poem: &'a mut Poem,
}

impl<'a> EditorTab<'a> for MetadataTab<'a> {
    type EditorData = ();

    fn from_files(
        files: &'a mut [LoadableFiles],
        global_data: &'a mut GlobalEditorData,
    ) -> Option<Self> {
        for file in files {
            if let LoadableFiles::SaveData(_, data) = file {
                return Some(MetadataTab {
                    safety_off: &mut global_data.safety_off,
                    version: &mut data.version,
                    name: &mut data.name,
                    theo_sister_name: &mut data.theo_sister_name,
                    has_modded_save_data: &mut data.has_modded_save_data,
                    flags: &mut data.flags,
                    poem: &mut data.poem,
                });
            }
        }
        None
    }

    fn display(
        mut self,
        ui: &mut Ui,
        _: &mut Self::EditorData,
        _: &Runtime,
        _: &Arc<Mutex<Vec<PopupWindow>>>,
    ) -> eframe::egui::Response {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Save Version: ");
                ui.add_enabled(*self.safety_off, TextEdit::singleline(self.version));
                ui.info_hover(
                    "Modifying this could make the save not load if the game version you try to \
                     load the save with doesn't match this.",
                )
            });
            ui.horizontal(|ui| {
                ui.label("Save Name: ");
                ui.text_edit_singleline(self.name);
            });

            ui.horizontal(|ui| {
                ui.label("Theo's Sister's Name:");
                ui.text_edit_singleline(self.theo_sister_name);
                ui.info_hover(
                    "The name of Theo's sister.\nDefaults to 'Alex,' is changed to 'Maddy' if the \
                     player's name is 'Alex.'\nMight not actually update what's in game as this \
                     is stored in the dialogue files too.",
                );
            });

            ui.checkbox(self.has_modded_save_data, "Has modded data");

            self.show_flags(ui);
            show_poem(self.poem, ui);
        })
        .response
    }
}

// TODO: add summit gems
impl<'a> MetadataTab<'a> {
    pub fn show_flags(&mut self, ui: &mut Ui) {
        let met_theo = self.flags.contains(&VanillaFlags::MetTheo.into());
        let mut met_theo2 = met_theo;
        let theo_knows_name = self.flags.contains(&VanillaFlags::TheoKnowsName.into());
        let mut theo_knows_name2 = theo_knows_name;

        ui.heading("Vanilla Flags");
        ui.checkbox(&mut met_theo2, "Met Theo");
        ui.checkbox(&mut theo_knows_name2, "Theo Knows Name");

        if met_theo && !met_theo2 {
            // We know this will exist because we've checked that the flag is in the vec earlier
            let idx = self
                .flags
                .iter()
                .position(|f| *f == VanillaFlags::MetTheo.into())
                .unwrap();
            self.flags.remove(idx);
        }

        if theo_knows_name && !theo_knows_name2 {
            // We know this will exist because we've checked that the flag is in the vec earlier
            let idx = self
                .flags
                .iter()
                .position(|f| *f == VanillaFlags::TheoKnowsName.into())
                .unwrap();
            self.flags.remove(idx);
        }

        if !met_theo && met_theo2 {
            self.flags.push(VanillaFlags::MetTheo.into());
        }

        if !theo_knows_name && theo_knows_name2 {
            self.flags.push(VanillaFlags::TheoKnowsName.into());
        }
    }
}

pub fn show_poem(poem: &mut Poem, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label("Poem order: ");
        // Drag and drop code adapted from https://github.com/emilk/egui/blob/master/crates/egui_demo_lib/src/demo/drag_and_drop.rs

        // If there is a drop, store the location of the item being dragged, and the destination for the drop.
        let mut from = None;
        let mut to = None;

        ui.dnd_drop_zone::<usize, _>(Frame::default().inner_margin(4.0), |ui| {
            ui.set_min_size(vec2(64.0, 32.0));

            for (idx, entry) in poem.iter().enumerate() {
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
            let item = poem.remove(*from);

            to = to.min(poem.len());
            poem.insert(to, item);
        }
    });
}


pub struct AssistsTab<'a> {
    safety_off: &'a mut bool,
    cheat_mode: &'a mut bool,
    assist_mode: &'a mut bool,
    variant_mode: &'a mut bool,
    assists: &'a mut Assists,
}

impl<'a> EditorTab<'a> for AssistsTab<'a> {
    type EditorData = ();

    fn from_files(
        files: &'a mut [LoadableFiles],
        global_data: &'a mut GlobalEditorData,
    ) -> Option<Self>
    where
        Self: Sized,
    {
        for file in files {
            if let LoadableFiles::SaveData(_, data) = file {
                return Some(AssistsTab {
                    safety_off: &mut global_data.safety_off,
                    cheat_mode: &mut data.cheat_mode,
                    assist_mode: &mut data.assist_mode,
                    variant_mode: &mut data.variant_mode,
                    assists: &mut data.assists,
                });
            }
        }
        None
    }

    fn display(
        mut self,
        ui: &mut Ui,
        _: &mut Self::EditorData,
        _: &Runtime,
        _: &Arc<Mutex<Vec<PopupWindow>>>,
    ) -> eframe::egui::Response {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Cheats Enabled:");
                ui.checkbox(self.cheat_mode, "");
                ui.label("Assists Enabled:");
                ui.checkbox(self.assist_mode, "");
                ui.label("Variants Enabled:");
                ui.checkbox(self.variant_mode, "");
            });

            let assist_editing_enabled = *self.assist_mode | *self.variant_mode | *self.safety_off;

            let assists = &mut self.assists;
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
                            ui.selectable_value(
                                &mut assists.dash_mode,
                                DashMode::Infinite,
                                "Infinite",
                            );
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
        })
        .response
    }
}
