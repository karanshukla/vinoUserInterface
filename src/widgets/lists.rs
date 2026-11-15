//! What a list row or tile is made of: its selected looks and a file's tile.

use egui::{Rect, Response, Sense, Stroke, StrokeKind, Ui, vec2};

use super::palette;
use crate::a11y;
use crate::icons::{self, Icon};

/// A selected list row's look: the tint, with a 1 px accent inside.
pub fn paint_selected(ui: &Ui, rect: Rect, radius: f32) {
    let p = palette(ui);
    let painter = ui.painter();
    painter.rect(
        rect,
        radius,
        p.tint,
        Stroke::new(1.0_f32, p.accent),
        StrokeKind::Inside,
    );
}

/// A selected tile's look: a 2 px accent ring with a gap of page colour
/// between it and the tile.
pub fn paint_ring(ui: &Ui, rect: Rect, radius: f32) {
    let p = palette(ui);
    let painter = ui.painter();
    painter.rect_stroke(
        rect.expand(2.0),
        radius + 2.0,
        Stroke::new(2.0_f32, p.bg),
        StrokeKind::Inside,
    );
    painter.rect_stroke(
        rect.expand(4.0),
        radius + 4.0,
        Stroke::new(2.0_f32, p.accent),
        StrokeKind::Inside,
    );
}

/// A square tile for a file icon, as a list row starts.
pub fn file_tile(ui: &mut Ui, icon: Icon, size: f32) -> Response {
    let p = palette(ui);
    let (rect, response) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    a11y::decorative(ui, response.id);
    if ui.is_rect_visible(rect) {
        ui.painter().rect_filled(rect, 8.0, p.field);
        icons::paint(
            ui.painter(),
            Rect::from_center_size(rect.center(), vec2(16.0, 16.0)),
            icon,
            p.muted,
        );
    }
    response
}
