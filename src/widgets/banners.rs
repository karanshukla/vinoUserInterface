//! Banners and notices: what the user should know, across the column.

use egui::{Align, Color32, CornerRadius, Layout, Rect, Response, RichText, Ui, vec2};

use super::{Button, CONTROL_HEIGHT, SMALL_HEIGHT, Tone, palette};
use crate::a11y;
use crate::icons::{self, Icon};
use crate::theme::Palette;

/// Width of the semantic left accent bar on a banner or notice.
const ACCENT_BAR: f32 = 3.0;

/// Padding inside an edge-anchored strip: leaves room past the left accent bar.
fn banner_margin() -> egui::Margin {
    egui::Margin {
        left: 15,
        right: 12,
        top: 10,
        bottom: 10,
    }
}

/// A banner's fill, left accent bar colour, and mark. Only a warning or a
/// failure has a mark, so the tone isn't carried by colour alone; an info
/// banner's text says it all, on a quiet neutral strip.
fn tone_look(p: &Palette, tone: Tone) -> (Color32, Color32, Option<(Icon, Color32)>) {
    match tone {
        Tone::Warn => (p.warn_tint, p.warn, Some((Icon::Warning, p.warn))),
        Tone::Error => (p.error_tint, p.error, Some((Icon::Error, p.error))),
        Tone::Info | Tone::Neutral => (p.surface, p.rule_strong, None),
    }
}

/// A banner across the column: a tinted surface with a left accent bar and,
/// for a warning or a failure, its mark; text that wraps and whatever
/// `actions` adds at the right. The text is read out when it appears.
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

/// What a banner and a notice share: the tone's strip and mark, `trailing`
/// at the right and the text in what is left, read out as `announce` says.
fn toned_strip<R>(
    ui: &mut Ui,
    tone: Tone,
    text: &str,
    min_height: f32,
    announce: fn(&Response),
    trailing: impl FnOnce(&mut Ui) -> R,
) -> R {
    let (fill, accent, mark) = tone_look(palette(ui), tone);
    let shown = egui::Frame::new()
        .fill(fill)
        .corner_radius(CornerRadius {
            nw: 0,
            sw: 0,
            ne: 6,
            se: 6,
        })
        .inner_margin(banner_margin())
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
        });
    let bar = Rect::from_min_size(
        shown.response.rect.min,
        vec2(ACCENT_BAR, shown.response.rect.height()),
    );
    ui.painter().rect_filled(bar, 0.0, accent);
    shown.inner
}
