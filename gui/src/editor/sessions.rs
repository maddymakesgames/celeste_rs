use celeste_rs::saves::{session::SavedSession, util::FileTime, vanilla::Modes};
use eframe::egui::{
    scroll_area::ScrollBarVisibility,
    CollapsingHeader,
    CollapsingResponse,
    DragValue,
    Response,
    RichText,
    Sense,
    TextEdit,
    TextStyle,
    Ui,
    Vec2,
};

use crate::{
    editor::{
        entity_id_list_widget,
        file_time_widget,
        level_sets::mode_widget,
        CelesteEditorRichTextExt,
        CelesteEditorUiExt,
        EditorTab,
    },
    main_menu::LoadableFiles,
    tabbed::TabbedContentWidget,
};

pub struct SessionsTab<'a> {
    sessions: Vec<(String, &'a mut SavedSession)>,
    total_time: Option<&'a mut FileTime>,
    total_deaths: Option<&'a mut u64>,
    safety_off: bool,
}

pub struct SessionsData {
    selected_panel: usize,
    add_strawb_buf: String,
    add_key_buf: String,
    add_dnl_buf: String,
}

impl SessionsData {
    pub fn new() -> Self {
        SessionsData {
            selected_panel: 0,
            add_strawb_buf: String::new(),
            add_key_buf: String::new(),
            add_dnl_buf: String::new(),
        }
    }
}

impl<'a> EditorTab<'a> for SessionsTab<'a> {
    type EditorData = SessionsData;

    fn from_files(
        files: &'a mut [crate::main_menu::LoadableFiles],
        global_data: &'a mut super::GlobalEditorData,
    ) -> Option<Self>
    where
        Self: Sized,
    {
        let mut sessions_vec = Vec::new();
        let mut time = None;
        let mut total_deaths = None;

        for file in files {
            #[allow(clippy::single_match)]
            match file {
                LoadableFiles::SaveData(_, save) => {
                    if let Some(session) = &mut save.current_session {
                        sessions_vec.push(("Vanilla Session".to_owned(), session));
                    }

                    if let Some(session) = &mut save.current_session_safe {
                        sessions_vec.push(("Modded Session".to_owned(), session));
                    }

                    time = Some(&mut save.time);
                    total_deaths = Some(&mut save.total_deaths);
                }
                _ => {}
            }
        }

        if !sessions_vec.is_empty() {
            Some(SessionsTab {
                sessions: sessions_vec,
                safety_off: global_data.safety_off,
                total_deaths,
                total_time: time,
            })
        } else {
            None
        }
    }

    fn display(
        mut self,
        ui: &mut Ui,
        data: &mut Self::EditorData,
        _: &tokio::runtime::Runtime,
        _: &std::sync::Arc<tokio::sync::Mutex<Vec<crate::PopupWindow>>>,
    ) -> eframe::egui::Response {
        self.sessions.sort_by_key(|(s, _)| s.clone());
        self.sessions.reverse();
        let mut default_deaths = 0;
        let mut default_time = FileTime::from_millis(0);

        if self.sessions.len() > 1 {
            let tab_names = self
                .sessions
                .iter()
                .map(|(s, _)| s.to_owned())
                .collect::<Vec<_>>();

            TabbedContentWidget::show(
                ui,
                &mut data.selected_panel,
                tab_names,
                ScrollBarVisibility::VisibleWhenNeeded,
                TextStyle::Body,
                |idx, ui| {
                    let session_entry = &mut self.sessions[idx];
                    Self::show_session_impl(
                        ui,
                        session_entry.1,
                        self.safety_off,
                        self.total_deaths.unwrap_or(&mut default_deaths),
                        self.total_time.unwrap_or(&mut default_time),
                        &session_entry.0,
                        &mut data.add_strawb_buf,
                        &mut data.add_key_buf,
                        &mut data.add_dnl_buf,
                    )
                },
            )
            .response
        } else {
            let (name, session) = &mut self.sessions[0];
            Self::show_session_impl(
                ui,
                session,
                self.safety_off,
                self.total_deaths.unwrap_or(&mut default_deaths),
                self.total_time.unwrap_or(&mut default_time),
                name,
                &mut data.add_strawb_buf,
                &mut data.add_key_buf,
                &mut data.add_dnl_buf,
            )
        }
    }
}

impl<'a> SessionsTab<'a> {
    #[allow(clippy::too_many_arguments)]
    fn show_session_impl(
        ui: &mut Ui,
        session: &mut SavedSession,
        safety_off: bool,
        total_deaths: &mut u64,
        total_time: &mut FileTime,
        id_filler: &str,
        strawb_add_buff: &mut String,
        key_add_buf: &mut String,
        dnl_add_buf: &mut String,
    ) -> Response {
        ui.horizontal(|ui| {
            ui.label("Current area sid: ");
            // Modded session levels will ALWAYS have a session id so this will always show
            if let Some(sid) = &mut session.area.sid {
                ui.add_enabled(safety_off, TextEdit::singleline(sid));
            }

            ui.info_hover(
                "You probably shouldn't change the map the session is in as the rest of the data \
                 will likely be invalid.",
            );
        });

        ui.horizontal(|ui| {
            ui.label("Respawn point: ");
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
        if session.restarted_from_golden.is_some() {
            ui.checkbox(
                session.restarted_from_golden.as_mut().unwrap(),
                "Restarted from golden",
            );
        }

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

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Session Deaths: ");
                        ui.label("Session Dashes: ");
                        ui.label("Dashes at Start: ");
                        ui.label("Deaths in Level: ");
                    });

                    ui.vertical(|ui| {
                        let deaths = stats.deaths;
                        if ui.add(DragValue::new(&mut stats.deaths)).changed() {
                            if deaths > stats.deaths {
                                *total_deaths -= deaths.abs_diff(stats.deaths);
                            } else {
                                *total_deaths += deaths.abs_diff(stats.deaths);
                            }
                            ui.add(DragValue::new(&mut stats.dashes));
                            ui.add(DragValue::new(&mut stats.dashes_at_start));
                            ui.add(DragValue::new(&mut stats.session_deaths));
                        }
                    });
                });

                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("In Area: ");
                        ui.label("First Level Played: ");
                        ui.label("Cassette Collected: ");
                        ui.label("Crystal Heart Collected: ");
                        ui.label("Dreaming: ");
                        ui.label("Started From Beginning: ");
                        ui.label("Has A Golden: ");
                        ui.label("Hit Checkpoint: ");
                    });

                    ui.vertical(|ui| {
                        ui.checkbox(&mut stats.in_area, "");
                        ui.checkbox(&mut stats.first_level, "");
                        ui.checkbox(&mut stats.cassette, "");
                        ui.checkbox(&mut stats.heart_gem, "");
                        ui.checkbox(&mut stats.dreaming, "");
                        ui.checkbox(&mut stats.started_from_beginning, "");
                        ui.checkbox(&mut stats.grabbed_golden, "");
                        ui.checkbox(&mut stats.hit_checkpoint, "");
                    });
                });
            });

        ui.collapsing(RichText::new("Inventory").heading2(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Dashes");
                ui.add(DragValue::new(&mut session.inventory.dashes));
            });

            ui.checkbox(&mut session.inventory.dream_dash, "Dream dash");
            ui.checkbox(&mut session.inventory.backpack, "Backpack");
            ui.checkbox(&mut session.inventory.no_refills, "No refills");
        });

        if !session.counters.is_empty() {
            ui.collapsing(RichText::new("Counters").heading2(), |ui| {
                for counter in session.counters.iter_mut() {
                    ui.horizontal(|ui| {
                        ui.label(&counter.key);
                        ui.add(DragValue::new(&mut counter.value));
                    });
                }
            });
        }

        ui.collapsing(RichText::new("Collected Strawberries").heading2(), |ui| {
            entity_id_list_widget(
                ui,
                "session_strawberries",
                "Strawberries",
                &mut session.strawberries,
                safety_off,
                None,
                strawb_add_buff,
            );
        });

        ui.collapsing(RichText::new("Held Keys").heading2(), |ui| {
            entity_id_list_widget(
                ui,
                "session_keys",
                "Keys",
                &mut session.keys,
                safety_off,
                None,
                key_add_buf,
            )
        });

        ui.collapsing(
            RichText::new("Entities marked 'do not load'").heading2(),
            |ui| {
                entity_id_list_widget(
                    ui,
                    "session_dnl",
                    "Entity",
                    &mut session.do_not_load,
                    safety_off,
                    None,
                    dnl_add_buf,
                );
            },
        );

        ui.collapsing(RichText::new("Old stats").heading2(), |ui| {
            ui.info("These are the stats you had before you started the current session.");
            ui.checkbox(&mut session.old_stats.area.cassette, "Cassette collected");

            ui.horizontal(|ui| {
                ui.add_enabled_ui(safety_off, |ui| {
                    area_mode_widget(
                        ui,
                        id_filler,
                        session.old_stats.area.sid(),
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

        ui.allocate_response(Vec2::new(0.0, 0.0), Sense::focusable_noninteractive())
    }
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
