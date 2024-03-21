use celeste_rs::saves::{session::SavedSession, util::FileTime};
use eframe::egui::{CollapsingHeader, DragValue, RichText, TextEdit, Ui};

use crate::{
    editor::{
        entity_id_list_widget,
        file_time_widget,
        level_sets::area_mode_widget,
        CelesteEditorRichTextExt,
        CelesteEditorUiExt,
        EditorScreen,
    },
    tabbed::TabbedContentWidget,
};

impl EditorScreen {
    pub fn show_session(&mut self, ui: &mut Ui) {
        #[allow(clippy::overly_complex_bool_expr)]
        if self.save.current_session_safe.is_some() && self.save.current_session.is_some() {
            let mut selected = self.selected_session_panel;
            TabbedContentWidget::show(
                ui,
                &mut selected,
                ["Vanilla Session", "Modded Session"],
                |selection, ui| match selection {
                    0 => {
                        // TODO: check that this isn't misinformation
                        ui.info(
                            "Since you have a modded session this will not load when you are \
                             running Everest. This will only be used if you boot into vanilla \
                             Celeste.",
                        );
                        if let Some(session) = self.save.current_session.as_mut() {
                            Self::show_session_impl(
                                ui,
                                session,
                                self.safety_off,
                                &mut self.save.total_deaths,
                                &mut self.save.time,
                                "vanilla_session",
                                &mut self.session_add_strawb_buff,
                            );
                        }
                    }
                    1 =>
                        if let Some(session) = self.save.current_session_safe.as_mut() {
                            Self::show_session_impl(
                                ui,
                                session,
                                self.safety_off,
                                &mut self.save.total_deaths,
                                &mut self.save.time,
                                "modded_session",
                                &mut self.session_add_strawb_buff,
                            );
                        },
                    _ => {
                        ui.info("Invalid session panel selected");
                    }
                },
            );
            self.selected_session_panel = selected;
        } else if let Some(session) = self.save.current_session_safe.as_mut() {
            Self::show_session_impl(
                ui,
                session,
                self.safety_off,
                &mut self.save.total_deaths,
                &mut self.save.time,
                "modded_session",
                &mut self.session_add_strawb_buff,
            );
        } else if let Some(session) = self.save.current_session.as_mut() {
            Self::show_session_impl(
                ui,
                session,
                self.safety_off,
                &mut self.save.total_deaths,
                &mut self.save.time,
                "vanilla_session",
                &mut self.session_add_strawb_buff,
            );
        } else {
            ui.info("No saved session found.");
        }
    }

    fn show_session_impl(
        ui: &mut Ui,
        session: &mut SavedSession,
        safety_off: bool,
        total_deaths: &mut u64,
        total_time: &mut FileTime,
        id_filler: &'static str,
        strawb_add_buff: &mut String,
    ) {
        ui.horizontal(|ui| {
            ui.label("Current area sid: ");
            // Modded session levels will ALWAYS have a session id so this will always show
            if let Some(s_id) = &mut session.area.s_id {
                ui.add_enabled(safety_off, TextEdit::singleline(s_id));
            }


            ui.info_hover(
                "You probably shouldn't change the map the session is in as the rest of the data \
                 will likely be invalid.",
            );
        });

        ui.label("Respawn point");
        ui.horizontal(|ui| {
            ui.label("x");
            ui.add_enabled(safety_off, DragValue::new(&mut session.respawn_point.x));
            ui.label("y");
            ui.add_enabled(safety_off, DragValue::new(&mut session.respawn_point.y));
            ui.info_hover(
                "Changing the respawn point manually seems like a bad idea! You can open the \
                 debug map in everest with f6 and then use that to manually set a respawn point \
                 with at least an idea of where you'll end up.",
            );
        });

        ui.heading2("Inventory");
        ui.horizontal(|ui| {
            ui.label("Dashes");
            ui.add(DragValue::new(&mut session.inventory.dashes));
        });

        ui.checkbox(&mut session.inventory.dream_dash, "Dream dash");
        ui.checkbox(&mut session.inventory.backpack, "Backpack");
        ui.checkbox(&mut session.inventory.no_refills, "No refills");

        if !session.counters.is_empty() {
            ui.heading2("Counters");
            for counter in session.counters.iter_mut() {
                ui.horizontal(|ui| {
                    ui.label(&counter.key);
                    ui.add(DragValue::new(&mut counter.value));
                });
            }
        }

        ui.collapsing("Collected Strawberries", |ui| {
            entity_id_list_widget(
                ui,
                "session_strawberries",
                "Strawberries",
                &mut session.strawberries,
                safety_off,
                &mut 0,
                &mut String::new(),
            );
        });

        ui.collapsing("Held Keys", |ui| {
            entity_id_list_widget(
                ui,
                "session_keys",
                "Keys",
                &mut session.keys,
                safety_off,
                &mut 0,
                &mut String::new(),
            )
        });

        ui.collapsing("Entities marked 'do not load'", |ui| {
            entity_id_list_widget(
                ui,
                "session_dnl",
                "Entity",
                &mut session.do_not_load,
                safety_off,
                &mut 0,
                &mut String::new(),
            );
        });


        CollapsingHeader::new(RichText::new("Session Stats").heading2())
            .default_open(true)
            .show(ui, |ui| {
                let stats = &mut session.stats;

                ui.info("These are the stats for the current session.");
                ui.horizontal(|ui| {
                    ui.label("Screen Name: ");
                    ui.add_enabled(safety_off, TextEdit::singleline(&mut stats.level));
                    ui.info_hover("You need to make sure the screen name is valid for the map.");
                });

                ui.horizontal(|ui| {
                    ui.label("Session Time:");
                    file_time_widget(&mut stats.time, ui);
                });

                ui.checkbox(
                    &mut stats.started_from_beginning,
                    "Session started from the beginning: ",
                );

                ui.horizontal(|ui| {
                    ui.label("Session Deaths: ");
                    let deaths = stats.deaths;
                    if ui.add(DragValue::new(&mut stats.deaths)).changed() {
                        if deaths > stats.deaths {
                            *total_deaths -= deaths.abs_diff(stats.deaths);
                        } else {
                            *total_deaths += deaths.abs_diff(stats.deaths);
                        }
                    }
                });

                ui.labeled("Session Dashes: ", DragValue::new(&mut stats.dashes));
                ui.labeled(
                    "Dashes at start: ",
                    DragValue::new(&mut stats.dashes_at_start),
                );
                ui.labeled(
                    "Deaths in current level: ",
                    DragValue::new(&mut stats.session_deaths),
                );

                ui.checkbox(&mut stats.in_area, "In Area: ");
                ui.checkbox(&mut stats.first_level, "Is the first level played: ");
                ui.checkbox(&mut stats.cassette, "Cassette collected: ");
                ui.checkbox(&mut stats.heart_gem, "Crystal heart collected: ");
                ui.checkbox(&mut stats.dreaming, "Dreaming: ");

                ui.checkbox(&mut stats.grabbed_golden, "Has a golden: ");
                ui.checkbox(&mut stats.hit_checkpoint, "Hit checkpoint: ");
            });

        ui.collapsing(RichText::new("Old stats").heading2(), |ui| {
            ui.info("These are the stats you had before you started the current session.");
            ui.checkbox(&mut session.old_stats.area.cassette, "Cassette collected");

            ui.horizontal(|ui| {
                ui.add_enabled_ui(safety_off, |ui| {
                    area_mode_widget(
                        ui,
                        id_filler,
                        &session.old_stats.area.sid,
                        safety_off,
                        total_deaths,
                        total_time,
                        &mut session.old_stats.modes,
                        strawb_add_buff,
                    );
                });
                ui.info_hover(
                    "These stats should be identical to the stats in the LevelStats tab.\nIf you \
                     want to change the stats you should change them there instead.",
                );
            });
        });

        ui.horizontal(|ui| {
            ui.label("Furthest Seen Level");
            if let Some(furthest_seen_level) = session.furthest_seen_level.as_mut() {
                ui.add_enabled(safety_off, TextEdit::singleline(furthest_seen_level));
            } else {
                let mut buf = String::new();
                ui.add_enabled(safety_off, TextEdit::singleline(&mut buf));
                if !buf.is_empty() {
                    session.furthest_seen_level = Some(buf);
                }
            }
            ui.info_hover("This needs to actually be a valid room inside the map.");
        });


        ui.checkbox(&mut session.beat_best_time, "Beat best time");
        ui.checkbox(&mut session.restarted_from_golden, "Restarted from golden");
    }
}
