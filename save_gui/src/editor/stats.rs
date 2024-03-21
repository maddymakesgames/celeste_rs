use eframe::egui::{DragValue, Ui};

use crate::editor::{file_time_widget, CelesteEditorUiExt, EditorScreen};

impl EditorScreen {
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
                         collected.\n\nThis is the count of all vanilla strawberries collected, \
                         including goldens.",
                    );
                });

                ui.horizontal(|ui| {
                    ui.add(DragValue::new(&mut save.total_golden_strawberries));
                    ui.info_hover(
                        "Unlike the total strawberries stat this does take into account modded \
                         goldens.\nDue to the way strawberries are stored in the save file, we \
                         cannot know if a strawberry is normal or golden and thus we cannot \
                         automatically update this value.\nWe assume that all strawberries added \
                         or removed are red berries.",
                    )
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
}
