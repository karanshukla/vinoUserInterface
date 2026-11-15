//! Banners and notices: what the user should know, across the column.

use egui::{Align, Color32, Layout, Response, RichText, Stroke, Ui};

use super::{Button, CONTROL_HEIGHT, GROUP_RADIUS, ROW_PADDING, SMALL_HEIGHT, Tone, palette};
use crate::a11y;
use crate::icons::{self, Icon};
use crate::theme::Palette;

/// A banner or notice is padded as a row is, so its text lines up with the
/// rows above and below it. Its 1 px hairline is drawn outside the margin,
/// so the margin is a pixel less: 16 x 12 from the outer edge, as in a group.
fn row_margin() -> egui::Margin {
    egui::Margin::symmetric(ROW_PADDING.x as i8 - 1, ROW_PADDING.y as i8 - 1)
}

/// A banner's fill and hairline, and its mark and the mark's colour. Only a
/// warning or a failure has a mark, so the tone isn't carried by colour
/// alone; an info banner's text says it all, on a plain surface.
fn tone_look(p: &Palette, tone: Tone) -> (Color32, Color32, Option<(Icon, Color32)>) {
    match tone {
        Tone::Warn => (p.warn_tint, p.warn_rule, Some((Icon::Warning, p.warn))),
        Tone::Error => (p.error_tint, p.error_rule, Some((Icon::Error, p.error))),
        Tone::Info | Tone::Neutral => (p.surface, p.rule, None),
    }
}

/// A banner across the column: a tinted surface with, for a warning or a
/// failure, its mark; text that wraps and whatever `actions` adds at the
/// right. The text is read out when it appears.
pub fn banner<R>(ui: &mut Ui, tone: Tone, text: &str, actions: impl FnOnce(&mut Ui) -> R) -> R {
    toned_strip(ui, tone, text, CONTROL_HEIGHT, a11y::live, actions)
}

/// A notice that just appeared, read out at once: a banner with a Dismiss
/// button. True when dismissed.
pub fn notice(ui: &mut Ui, tone: Tone, text: &str) -> bool {
    toned_strip(ui, tone, text, SMALL_HEIGHT, a11y::assertive, |ui| {
        ui.add(Button::ghost("Dismiss").small()).clicked()
    })
}

/// What a banner and a notice share: the tone's surface and mark, `trailing`
/// at the right and the text in what is left, read out as `announce` says.
fn toned_strip<R>(
    ui: &mut Ui,
    tone: Tone,
    text: &str,
    min_height: f32,
    announce: fn(&Response),
    trailing: impl FnOnce(&mut Ui) -> R,
) -> R {
    let (fill, rule, mark) = tone_look(palette(ui), tone);
    egui::Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0_f32, rule))
        .corner_radius(GROUP_RADIUS)
        .inner_margin(row_margin())
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0;
                ui.set_min_height(min_height);
                if let Some((icon, color)) = mark {
                    icons::icon(ui, icon, 16.0, color);
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let out = trailing(ui);
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        let shown = ui.add(egui::Label::new(RichText::new(text)).wrap());
                        announce(&shown);
                    });
                    out
                })
                .inner
            })
            .inner
        })
        .inner
}
