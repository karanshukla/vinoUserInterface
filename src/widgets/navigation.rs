//! Getting around a window: sidebar entries, tabs, and the frames of the
//! header and the sidebar.

use egui::{
    Align, Color32, FontId, Layout, Rect, Response, RichText, Sense, Stroke, Ui, UiBuilder, pos2,
    vec2,
};

use super::{
    ICON, NAV_ICON_CENTRE, NAV_TEXT, TAG_PAD, Tone, paint_focus_ring, palette, small, tag, weight,
};
use crate::a11y;
use crate::icons::{self, Icon};
use crate::theme::{self, Palette};

/// A sidebar entry: icon, text and an optional grey tag ("Beta"), tinted
/// when it is the current page. A tab for screen readers, named `name`.
pub fn nav_item(
    ui: &mut Ui,
    icon: Icon,
    text: &str,
    name: &str,
    selected: bool,
    badge: Option<&str>,
) -> Response {
    let p = palette(ui);
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 34.0), Sense::click());
    a11y::tab(&response, name, selected);
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let (fill, color, icon_color, family) = if selected {
        (p.tint, p.text, p.accent, theme::semibold())
    } else if response.hovered() {
        (p.field, p.text, p.text, theme::medium())
    } else {
        (Color32::TRANSPARENT, p.muted, p.muted, theme::medium())
    };
    let painter = ui.painter();
    painter.rect_filled(rect, 8.0, fill);
    icons::paint(
        painter,
        Rect::from_center_size(
            pos2(rect.left() + NAV_ICON_CENTRE, rect.center().y),
            vec2(ICON, ICON),
        ),
        icon,
        icon_color,
    );
    let badge_w = badge.map_or(0.0, |badge| {
        painter
            .layout_no_wrap(
                badge.to_uppercase(),
                FontId::new(10.5, theme::semibold()),
                Color32::PLACEHOLDER,
            )
            .size()
            .x
            + 2.0 * TAG_PAD
            + 8.0
    });
    let mut job =
        egui::text::LayoutJob::simple_singleline(text.to_owned(), FontId::new(14.0, family), color);
    let room_before_badge = (rect.width() - NAV_TEXT - 10.0 - badge_w).max(0.0);
    job.wrap = egui::text::TextWrapping::truncate_at_width(room_before_badge);
    let galley = painter.layout_job(job);
    painter.galley(
        pos2(
            rect.left() + NAV_TEXT,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        color,
    );
    if response.has_focus() {
        paint_focus_ring(painter, rect, 8.0, p);
    }
    if let Some(badge) = badge {
        let mut badge_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(Rect::from_min_max(
                    pos2(rect.left(), rect.top()),
                    pos2(rect.right() - 10.0, rect.bottom()),
                ))
                .layout(Layout::right_to_left(Align::Center)),
        );
        let shown = tag(&mut badge_ui, Tone::Neutral, badge);
        a11y::decorative(ui, shown.id);
    }
    response
}

/// An underlined tab, for a row of them under a narrow window's header or
/// in a viewer. Painted in `colors`, the header's.
pub fn tab(
    ui: &mut Ui,
    name: &str,
    active: bool,
    enabled: bool,
    colors: theme::Header,
) -> Response {
    let p = palette(ui);
    let galley = ui.painter().layout_no_wrap(
        name.to_owned(),
        FontId::new(14.0, weight(active)),
        Color32::PLACEHOLDER,
    );
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(galley.size() + vec2(0.0, 14.0), sense);
    let response = if enabled {
        response
    } else {
        response.on_hover_cursor(egui::CursorIcon::NotAllowed)
    };
    let lit = enabled && (active || response.hovered() || response.has_focus());
    let color = if lit {
        colors.text
    } else if enabled {
        colors.muted
    } else {
        colors.muted.gamma_multiply(0.5)
    };
    let painter = ui.painter();
    painter.galley(rect.left_top() + vec2(0.0, 5.0), galley, color);
    if active {
        painter.hline(
            rect.left()..=rect.right(),
            rect.bottom() - 1.0,
            Stroke::new(2.0_f32, p.accent),
        );
    }
    if response.has_focus() {
        paint_focus_ring(painter, rect.expand2(vec2(4.0, 0.0)), 4.0, p);
    }
    let mut response = response;
    if !enabled {
        response.flags.remove(egui::response::Flags::ENABLED);
    }
    a11y::tab(&response, name, active);
    response
}

/// The header strip's frame: the surface (or KDE's header colour) with a
/// hairline under it, 20 x 10/12 inside. An `inner_margin` of your own is
/// measured from the strip's edge too: the hairline is drawn outside it.
pub fn header_frame(colors: theme::Header, p: &Palette) -> egui::Frame {
    egui::Frame::new()
        .fill(colors.bg)
        .stroke(Stroke::new(1.0_f32, p.rule))
        // A Frame draws its stroke outside the margin: pulled back a pixel
        // on three sides, only the hairline under the strip is left in view.
        .outer_margin(egui::Margin {
            left: -1,
            right: -1,
            top: -1,
            bottom: 0,
        })
        .inner_margin(egui::Margin {
            left: 20,
            right: 20,
            top: 10,
            bottom: 12,
        })
}

/// The sidebar's frame: the surface, 10 beside, 14 above and 12 below.
pub fn sidebar_frame(p: &Palette) -> egui::Frame {
    egui::Frame::new()
        .fill(p.surface)
        .inner_margin(egui::Margin {
            left: 10,
            right: 10,
            top: 14,
            bottom: 12,
        })
}

/// The brand's logo: on the nav icons' centre, it leaves 6 to the text column.
const BRAND_LOGO: f32 = 24.0;

/// The sidebar's top: the app's logo on the nav items' icon column, its name
/// (15 bold) beside it on their text column, and a line of small text under
/// the name. The logo is drawn by `logo` at the size it is given, or left out.
pub fn sidebar_brand(
    ui: &mut Ui,
    logo: Option<impl FnOnce(&mut Ui, f32) -> Response>,
    name: &str,
    line: &str,
) {
    let size = BRAND_LOGO;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        if let Some(logo) = logo {
            ui.add_space(NAV_ICON_CENTRE - size / 2.0);
            logo(ui, size);
            ui.add_space(NAV_TEXT - NAV_ICON_CENTRE - size / 2.0);
        } else {
            ui.add_space(NAV_TEXT);
        }
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.add(
                egui::Label::new(RichText::new(name).size(15.0).family(theme::bold())).truncate(),
            );
            small(ui, line);
        });
    });
}

/// The sidebar's foot, pinned to its bottom: the app's one status `chip`,
/// usually. Its contents start on the nav items' icon column.
pub fn sidebar_foot<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.add_space(NAV_ICON_CENTRE - ICON / 2.0);
            add(ui)
        })
        .inner
    })
    .inner
}

/// Which edge of the window a side panel sits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Left,
    Right,
}

/// The hairline on a panel's inner edge, drawn after its content: on the
/// panel's last column of pixels, the full height of the panel, whatever
/// margin its frame has.
pub fn panel_rule(ui: &Ui, side: Edge) {
    let p = palette(ui);
    let panel_with_margin = ui.clip_rect();
    let x = match side {
        Edge::Left => panel_with_margin.right() - 0.5,
        Edge::Right => panel_with_margin.left() + 0.5,
    };
    ui.painter()
        .vline(x, panel_with_margin.y_range(), Stroke::new(1.0_f32, p.rule));
}
