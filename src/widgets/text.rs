//! Titles, labels and body text.

use egui::{Color32, Response, RichText, Ui};

use super::palette;
use crate::a11y;
use crate::theme;

/// The page title: 22 bold, a level-1 heading.
pub fn title(ui: &mut Ui, text: &str) -> Response {
    let response =
        ui.add(egui::Label::new(RichText::new(text).size(22.0).family(theme::bold())).truncate());
    a11y::heading(&response, text, 1);
    response
}

/// A window's title in its header: 16 semibold, a level-1 heading.
pub fn window_title(ui: &mut Ui, text: &str, color: Color32) -> Response {
    let response = ui.add(
        egui::Label::new(
            RichText::new(text)
                .size(16.0)
                .family(theme::semibold())
                .color(color),
        )
        .truncate(),
    );
    a11y::heading(&response, text, 1);
    response
}

/// A group's label above it: 13 semibold, muted, in sentence case as
/// passed, a level-2 heading.
pub fn group_label(ui: &mut Ui, text: &str) -> Response {
    ui.add_space(14.0);
    let p = palette(ui);
    let response = ui.add(
        egui::Label::new(
            RichText::new(text)
                .size(13.0)
                .family(theme::semibold())
                .color(p.muted),
        )
        .truncate(),
    );
    a11y::heading(&response, text, 2);
    response
}

/// A short line under a group label.
pub fn group_help(ui: &mut Ui, text: &str) -> Response {
    let p = palette(ui);
    ui.add(egui::Label::new(RichText::new(text).size(12.5).color(p.muted)).wrap())
}

/// Muted body text.
pub fn muted(ui: &mut Ui, text: impl Into<String>) -> Response {
    let p = palette(ui);
    ui.add(egui::Label::new(RichText::new(text).color(p.muted)).wrap())
}

/// Small muted text: help, facts, captions.
pub fn small(ui: &mut Ui, text: impl Into<String>) -> Response {
    let p = palette(ui);
    ui.add(egui::Label::new(RichText::new(text).size(12.5).color(p.muted)).wrap())
}

/// A path or command, in the monospace face, on one line.
pub fn mono(ui: &mut Ui, text: &str) -> Response {
    ui.add(egui::Label::new(RichText::new(text).monospace()).truncate())
        .on_hover_text(text)
}

/// A big number with what it counts under it.
pub fn stat(ui: &mut Ui, value: &str, label: &str) -> Response {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        let shown = ui.label(RichText::new(value).size(22.0).family(theme::bold()));
        small(ui, label);
        shown
    })
    .inner
}
