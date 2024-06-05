use std::hash::Hash;

use eframe::egui::{
    scroll_area::ScrollBarVisibility,
    Align,
    Color32,
    FontSelection,
    InnerResponse,
    Pos2,
    Rect,
    Rounding,
    ScrollArea,
    Sense,
    Stroke,
    TextStyle,
    Ui,
    Vec2,
    WidgetText,
};


pub struct TabbedContentWidget;

impl TabbedContentWidget {
    pub fn show<W, R>(
        ui: &mut Ui,
        selected: &mut usize,
        tabs: impl AsRef<[W]>,
        scroll_bar: ScrollBarVisibility,
        text_style: TextStyle,
        show_ui: impl FnOnce(usize, &mut Ui) -> R,
    ) -> InnerResponse<R>
    where
        W: Into<WidgetText> + Clone + Hash,
    {
        ui.vertical(|ui| {
            let mut selected_rect = Rect::NOTHING;
            let mut farthest_right = -1.0;
            ScrollArea::horizontal()
                .id_source(tabs.as_ref())
                .auto_shrink(true)
                .scroll_bar_visibility(scroll_bar)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for (idx, label) in tabs.as_ref().iter().enumerate() {
                            let label: WidgetText = label.clone().into();

                            let job = label.into_layout_job(
                                ui.style(),
                                FontSelection::Style(text_style.clone()),
                                Align::Center,
                            );


                            let galley = ui.painter().layout_job(job);

                            let (res, painter) =
                                ui.allocate_painter(galley.rect.expand(4.0).size(), Sense::click());

                            let mut color = ui.style().visuals.window_fill();


                            if res.clicked() {
                                *selected = idx;
                            }

                            if *selected == idx {
                                selected_rect = res.rect;
                                color = Color32::from_gray(64);
                            }

                            if res.hovered() {
                                color = Color32::from_gray(80);
                            }

                            painter.rect(
                                res.rect,
                                Rounding::same(0.0),
                                color,
                                Stroke::new(0.0, color),
                            );

                            painter.galley(
                                res.rect.shrink(4.0).left_top(),
                                galley,
                                ui.style().visuals.text_color(),
                            );

                            if res.rect.right() > farthest_right {
                                farthest_right = res.rect.right();
                            }
                        }
                    });
                });

            let available_x = ui.available_width();


            let (_id, rect) = ui.allocate_space(Vec2::new(available_x, 3.0));


            let separator_stroke = Stroke::new(2.0, Color32::from_gray(128));
            let painter = ui.painter();

            if selected_rect.left() <= rect.left() {
                selected_rect.set_left(rect.left());
            }

            if selected_rect.right() <= rect.left() {
                selected_rect.set_right(rect.left());
            }

            if selected_rect.left() >= rect.right() {
                selected_rect.set_left(rect.right());
            }

            if selected_rect.right() >= rect.right() {
                selected_rect.set_right(rect.right());
            }


            if selected_rect.left() != rect.left() {
                painter.line_segment(
                    [
                        Pos2::new(rect.left(), selected_rect.bottom()),
                        Pos2::new(selected_rect.left(), selected_rect.bottom()),
                    ],
                    separator_stroke,
                );
            }

            if selected_rect.left() != rect.right() && selected_rect.left() != rect.left() {
                painter.line_segment(
                    [
                        Pos2::new(selected_rect.left(), selected_rect.bottom()),
                        Pos2::new(selected_rect.left(), selected_rect.top()),
                    ],
                    separator_stroke,
                );
            }

            if selected_rect.left() != selected_rect.right() {
                painter.line_segment(
                    [
                        Pos2::new(selected_rect.left(), selected_rect.top()),
                        Pos2::new(selected_rect.right(), selected_rect.top()),
                    ],
                    separator_stroke,
                );
            }

            if selected_rect.right() != rect.left() && selected_rect.right() != rect.right() {
                painter.line_segment(
                    [
                        Pos2::new(selected_rect.right(), selected_rect.top()),
                        Pos2::new(selected_rect.right(), selected_rect.bottom()),
                    ],
                    separator_stroke,
                );
            }

            if selected_rect.right() != rect.right() {
                painter.line_segment(
                    [
                        Pos2::new(selected_rect.right(), selected_rect.bottom()),
                        Pos2::new(rect.right(), selected_rect.bottom()),
                    ],
                    separator_stroke,
                );
            }

            show_ui(*selected, ui)
        })
    }
}
