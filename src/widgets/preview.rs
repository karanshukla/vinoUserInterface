//! In-context hover previews: surface secondary details without leaving the
//! current task or navigating to another page.

use egui::{Response, RichText, Ui};

use super::{palette, small};
use crate::theme;

/// Maximum width of a hover preview card so prose stays scannable.
const PREVIEW_MAX: f32 = 280.0;

/// Attaches a styled hover-to-preview card to `response`: a medium title above
/// `body`, capped at a scannable width inside the theme's popup surface.
pub fn hover_card(response: Response, title: &str, body: impl FnOnce(&mut Ui)) -> Response {
    let title = title.to_owned();
    response.on_hover_ui(move |ui| {
        let p = palette(ui);
        ui.set_max_width(PREVIEW_MAX);
        ui.spacing_mut().item_spacing.y = 4.0;
        ui.add(
            egui::Label::new(
                RichText::new(&title)
                    .size(13.5)
                    .family(theme::semibold())
                    .color(p.text),
            )
            .wrap(),
        );
        body(ui);
    })
}

/// A [`hover_card`] with a single muted detail paragraph under `title`.
pub fn hover_card_text(response: Response, title: &str, detail: &str) -> Response {
    let detail = detail.to_owned();
    hover_card(response, title, move |ui| {
        small(ui, detail);
    })
}
