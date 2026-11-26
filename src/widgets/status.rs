//! Status and facts in passing: status text, tags, key caps, bars and the
//! spinner.

use egui::{
    Align, Color32, FontId, Layout, Rect, Response, RichText, Role, Sense, Shape, Stroke,
    StrokeKind, Ui, WidgetInfo, pos2, vec2,
};

use super::{TAG_PAD, palette, small};
use crate::a11y;
use crate::theme::{self, Palette};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipKind {
    Ok,
    Warn,
    Err,
    Neutral,
}

/// The colour a status is written in: muted unless it needs the user (amber)
/// or failed (red). All is well needs no colour.
fn chip_color(p: &Palette, kind: ChipKind) -> Color32 {
    match kind {
        ChipKind::Warn => p.warn,
        ChipKind::Err => p.error,
        ChipKind::Ok | ChipKind::Neutral => p.muted,
    }
}

/// A status in a few words: its text and nothing else, 24 tall to sit in a
/// row, read as its text. Muted for `Ok` and `Neutral`, amber for `Warn`,
/// red for `Err`. No dot and no pill: the words say it.
pub fn chip(ui: &mut Ui, kind: ChipKind, text: &str) -> Response {
    let color = chip_color(palette(ui), kind);
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        FontId::new(12.5, theme::medium()),
        Color32::PLACEHOLDER,
    );
    let (rect, response) = ui.allocate_exact_size(vec2(galley.size().x, 24.0), Sense::hover());
    response.widget_info(|| WidgetInfo::labeled(Role::Label, true, text));
    if ui.is_rect_visible(rect) {
        ui.painter().galley(
            pos2(rect.left(), rect.center().y - galley.size().y / 2.0),
            galley,
            color,
        );
    }
    response
}

/// Was a chip with a turning dot. The words already say something is under
/// way, so it now draws exactly as [`chip`].
#[deprecated(
    since = "0.2.1",
    note = "draws as `chip` now; use `chip`, and a `spinner` beside it only for a wait over about a second that nothing else shows"
)]
pub fn chip_busy(ui: &mut Ui, kind: ChipKind, text: &str) -> Response {
    chip(ui, kind, text)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// Worth knowing: a banner's icon in the text colour.
    Info,
    /// Needs the user: amber.
    Warn,
    /// Failed: red.
    Error,
    /// In passing: grey throughout.
    Neutral,
}

/// A small caps tag: a kind or "Beta" in grey, amber only when it needs the
/// user.
pub fn tag(ui: &mut Ui, tone: Tone, text: &str) -> Response {
    let p = palette(ui);
    let (fill, color) = match tone {
        Tone::Warn => (p.warn_tint, p.warn),
        Tone::Error => (p.error_tint, p.error),
        Tone::Info | Tone::Neutral => (p.field, p.muted),
    };
    let galley = ui.painter().layout_no_wrap(
        text.to_uppercase(),
        FontId::new(10.5, theme::semibold()),
        Color32::PLACEHOLDER,
    );
    let size = vec2(galley.size().x + 2.0 * TAG_PAD, 16.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    response.widget_info(|| WidgetInfo::labeled(Role::Label, true, text));
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        painter.rect_filled(rect, 5.0, fill);
        painter.galley(rect.center() - galley.size() / 2.0, galley, color);
    }
    response
}

/// One key as a cap.
pub fn keycap(ui: &mut Ui, text: &str) -> Response {
    let p = palette(ui);
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        FontId::new(12.5, theme::semibold()),
        Color32::PLACEHOLDER,
    );
    let size = vec2(galley.size().x + 16.0, 24.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    response.widget_info(|| WidgetInfo::labeled(Role::Label, true, text));
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        painter.rect(
            rect,
            5.0,
            p.field,
            Stroke::new(1.0_f32, p.rule_strong),
            StrokeKind::Inside,
        );
        painter.hline(
            rect.left() + 1.0..=rect.right() - 1.0,
            rect.bottom() - 1.5,
            Stroke::new(1.0_f32, p.rule_strong),
        );
        painter.galley(
            pos2(
                rect.left() + 8.0,
                rect.center().y - galley.size().y / 2.0 - 0.5,
            ),
            galley,
            p.text,
        );
    }
    response
}

/// Sized up front and laid out left to right, as a segmented control is.
fn keys_width(ui: &Ui, keys: &[&str]) -> f32 {
    let measure = |text: &str, font: FontId| {
        ui.painter()
            .layout_no_wrap(text.to_owned(), font, Color32::PLACEHOLDER)
            .size()
            .x
    };
    let plus = measure("+", FontId::proportional(12.5));
    keys.iter()
        .map(|key| measure(key, FontId::new(12.5, theme::semibold())) + 16.0)
        .sum::<f32>()
        + keys.len().saturating_sub(1) as f32 * (plus + 8.0)
}

/// A shortcut as key caps: "Meta+Alt+C" is three caps with pluses between.
/// Read as the whole trigger.
pub fn keys(ui: &mut Ui, trigger: &str) -> Response {
    let p = palette(ui);
    let keys: Vec<&str> = trigger.split('+').map(str::trim).collect();
    let total = keys_width(ui, &keys);
    let response = ui
        .allocate_ui_with_layout(
            vec2(total, 24.0),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                for (i, key) in keys.iter().enumerate() {
                    if i > 0 {
                        let plus = ui.label(RichText::new("+").size(12.5).color(p.muted));
                        a11y::decorative(ui, plus.id);
                    }
                    let cap = keycap(ui, key);
                    a11y::decorative(ui, cap.id);
                }
            },
        )
        .response;
    response.widget_info(|| WidgetInfo::labeled(Role::Label, true, trigger));
    response
}

/// A thin bar of progress, `width` wide, with a 2 px separation between the
/// filled segment and the remaining track.
pub fn progress(ui: &mut Ui, fraction: f32, width: f32) -> Response {
    let p = palette(ui);
    let (rect, response) = ui.allocate_exact_size(vec2(width, 6.0), Sense::hover());
    a11y::decorative(ui, response.id);
    if ui.is_rect_visible(rect) {
        let clamped = fraction.clamp(0.0, 1.0);
        let shown = theme::animate_value(ui.ctx(), response.id, clamped, theme::MOTION_SPATIAL)
            .clamp(0.0, 1.0);
        let painter = ui.painter();
        let done_w = rect.width() * shown;
        const GAP: f32 = 2.0;
        if done_w <= 0.5 {
            painter.rect_filled(rect, 3.0, p.pressed);
        } else if done_w >= rect.width() - 0.5 {
            painter.rect_filled(rect, 3.0, p.primary_fill);
        } else {
            let done = Rect::from_min_size(rect.min, vec2(done_w, rect.height()));
            painter.rect_filled(done, 3.0, p.primary_fill);
            let rest_left = (rect.left() + done_w + GAP).min(rect.right());
            if rect.right() - rest_left > 1.0 {
                let rest = Rect::from_min_max(pos2(rest_left, rect.top()), rect.right_bottom());
                painter.rect_filled(rest, 3.0, p.pressed);
            }
        }
    }
    response
}

/// Shares of a whole as one bar: (fraction, colour) each, 2 px apart.
pub fn share_bar(ui: &mut Ui, shares: &[(f32, Color32)], height: f32) -> Response {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    a11y::decorative(ui, response.id);
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let total: f32 = shares.iter().map(|(f, _)| f.max(0.0)).sum();
    if total <= 0.0 {
        ui.painter()
            .rect_filled(rect, height / 2.0, palette(ui).pressed);
        return response;
    }
    let gaps = 2.0
        * (shares
            .iter()
            .filter(|(f, _)| *f > 0.0)
            .count()
            .saturating_sub(1)) as f32;
    let mut x = rect.left();
    for (fraction, color) in shares {
        if *fraction <= 0.0 {
            continue;
        }
        let w = (rect.width() - gaps) * fraction / total;
        let piece = Rect::from_min_size(pos2(x, rect.top()), vec2(w, height));
        ui.painter().rect_filled(piece, height / 2.0, *color);
        x += w + 2.0;
    }
    response
}

/// A legend entry: a small square in a colour, then text.
pub fn legend(ui: &mut Ui, color: Color32, text: &str) -> Response {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        let (rect, swatch) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
        a11y::decorative(ui, swatch.id);
        ui.painter().rect_filled(rect, 2.0, color);
        small(ui, text)
    })
    .inner
}

/// A spinner, 14 px, drawn at 10 fps rather than every frame: egui's own asks
/// for a repaint as fast as the GPU goes. For a wait over about a second that
/// nothing else on screen shows, beside words saying what it waits for; not
/// for a wait the words already describe. It stands still when the desktop
/// asks for less motion, and a screen reader skips it: the text beside it
/// says what is going on.
pub fn spinner(ui: &mut Ui) -> Response {
    let color = ui.visuals().strong_text_color();
    spinner_colored(ui, 14.0, color)
}

pub fn spinner_colored(ui: &mut Ui, size: f32, color: Color32) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    a11y::decorative(ui, response.id);
    if ui.is_rect_visible(rect) {
        paint_spinner(ui, rect, color);
    }
    response
}

fn paint_spinner(ui: &Ui, rect: Rect, color: Color32) {
    let still = theme::still(ui.ctx());
    let t = if still { 0.0 } else { ui.input(|i| i.time) };
    let start = (t * 6.0) % std::f64::consts::TAU;
    let radius = rect.width() / 2.0 - 1.5;
    let points: Vec<egui::Pos2> = (0..=16)
        .map(|i| {
            let a = start + i as f64 / 16.0 * std::f64::consts::PI * 1.5;
            rect.center() + vec2(a.cos() as f32, a.sin() as f32) * radius
        })
        .collect();
    ui.painter()
        .add(Shape::line(points, Stroke::new(2.0_f32, color)));
    if !still {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(100));
    }
}
