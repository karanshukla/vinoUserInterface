//! What a list row or tile is made of: its selected looks and a file's tile.

use egui::{Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui, vec2};

use super::{muted, paint_focus_ring, palette};
use crate::a11y;
use crate::icons::{self, Icon};

const SELECT_BAR: f32 = 3.0;

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

/// A borderless hairline list row's background, top divider, and edge-anchored
/// selection bar, painted into `background` under the row's content.
pub fn paint_list_row(
    ui: &Ui,
    background: egui::layers::ShapeIdx,
    rect: Rect,
    response: &Response,
    selected: bool,
) {
    let p = palette(ui);
    let painter = ui.painter();
    if selected {
        painter.set(background, Shape::rect_filled(rect, 0.0, p.tint));
        painter.rect_filled(
            Rect::from_min_size(rect.min, vec2(SELECT_BAR, rect.height())),
            0.0,
            p.accent,
        );
    } else if response.hovered() {
        painter.set(background, Shape::rect_filled(rect, 0.0, p.field));
    }
    painter.hline(
        rect.left()..=rect.right(),
        rect.top(),
        Stroke::new(1.0_f32, p.rule),
    );
    if response.has_focus() {
        paint_focus_ring(painter, rect, 0.0, p);
    }
}

/// Closes the bottom of a list of rows painted with [`paint_list_row`].
pub fn paint_list_end(ui: &Ui, last_rect: Rect) {
    let p = palette(ui);
    ui.painter().hline(
        last_rect.left()..=last_rect.right(),
        last_rect.bottom(),
        Stroke::new(1.0_f32, p.rule),
    );
}

/// A single-line muted empty state with an optional inline recovery action
/// (such as `"Clear filters"` or `"Add folder…"`), avoiding hero bloat.
pub fn empty_state<R>(ui: &mut Ui, message: &str, action: impl FnOnce(&mut Ui) -> R) -> R {
    ui.add_space(12.0);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        let shown = muted(ui, message);
        a11y::live(&shown);
        action(ui)
    })
    .inner
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
