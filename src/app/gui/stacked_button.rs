use egui::{Vec2, text::LayoutJob};

pub fn stacked_button(
    ui: &mut egui::Ui,
    job: LayoutJob,
    selected: bool,
    padding: Vec2,
) -> egui::Response {
    let galley = ui.fonts_mut(|f| f.layout_job(job));
    let galley_size = galley.size();

    let extra_w = padding.x;
    let extra_h = padding.y;
    let desired = Vec2::new(galley_size.x + extra_w, galley_size.y + extra_h);

    let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact_selectable(&response, selected);
        let painter = ui.painter();

        painter.rect_filled(rect, visuals.corner_radius, visuals.weak_bg_fill);

        let x = rect.left() + galley_size.x / 2.0 + extra_w / 2.0;
        let y = rect.top() + (rect.height() - galley_size.y) / 2.0;
        let pos = egui::pos2(x, y);
        painter.galley(pos, galley.clone(), visuals.text_color());
    }

    response
}
