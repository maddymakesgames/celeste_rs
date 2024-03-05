use eframe::egui::{
    Align,
    Color32,
    FontSelection,
    InnerResponse,
    Pos2,
    Rect,
    Rounding,
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
        mut show_ui: impl FnMut(usize, &mut Ui) -> R,
    ) -> InnerResponse<R>
    where
        W: Into<WidgetText> + Clone,
    {
        ui.vertical(|ui| {
            let mut selected_rect = Rect::NOTHING;
            ui.horizontal(|ui| {
                for (idx, label) in tabs.as_ref().iter().enumerate() {
                    let label: WidgetText = label.clone().into();

                    let job = label.into_layout_job(
                        ui.style(),
                        FontSelection::Style(TextStyle::Name("header2".into())),
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
                }
            });

            let available_x = ui.available_width();

            let (_id, rect) = ui.allocate_space(Vec2::new(available_x, 3.0));


            let separator_stroke = Stroke::new(2.0, Color32::from_gray(128));
            let painter = ui.painter();

            painter.line_segment(
                [
                    Pos2::new(rect.left(), selected_rect.bottom()),
                    Pos2::new(selected_rect.left(), selected_rect.bottom()),
                ],
                separator_stroke,
            );

            painter.line_segment(
                [
                    Pos2::new(selected_rect.left(), selected_rect.bottom()),
                    Pos2::new(selected_rect.left(), selected_rect.top()),
                ],
                separator_stroke,
            );

            painter.line_segment(
                [
                    Pos2::new(selected_rect.left(), selected_rect.top()),
                    Pos2::new(selected_rect.right(), selected_rect.top()),
                ],
                separator_stroke,
            );

            painter.line_segment(
                [
                    Pos2::new(selected_rect.right(), selected_rect.top()),
                    Pos2::new(selected_rect.right(), selected_rect.bottom()),
                ],
                separator_stroke,
            );

            painter.line_segment(
                [
                    Pos2::new(selected_rect.right(), selected_rect.bottom()),
                    Pos2::new(rect.right(), selected_rect.bottom()),
                ],
                separator_stroke,
            );

            // unwrap is fine cause selected should never be out of bounds of the iter unless tabs is empty
            show_ui(*selected, ui)
        })
    }
}
