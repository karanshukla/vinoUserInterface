//! The window's own title bar: minimise, maximise and close drawn by the app,
//! a header to drag it by, and edges to resize it from, for a window opened
//! without the desktop's decorations. The page's header is the title bar, so
//! the buttons sit in the app's colours instead of a strip above it.
//!
//! The desktop's bar does things this one can't: its window menu (egui has
//! no way to open it), its button layout setting, its shadow. So a tiling
//! compositor, which wants no buttons at all, keeps the desktop's (none), and
//! `VINO_UI_TITLE_BAR=system` brings the desktop's back anywhere.
//!
//! In an app: open the window with [`viewport`], drag the header with
//! [`drag`] before drawing into it, put [`controls`] at its right end, and
//! call [`edges`] last in the frame. A wide window, whose pages have a
//! sidebar and no header, gets [`bar`] above the page instead. A window with
//! a header at every width puts the buttons up in its corner with [`corner`].

use egui::{
    Align, Color32, CursorIcon, Id, Layout, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui,
    ViewportBuilder, ViewportCommand, WidgetInfo, pos2, vec2,
};

use crate::icons;
use crate::theme;

/// A window button's square.
pub const BUTTON: f32 = 28.0;
/// The gap between two window buttons.
const BUTTON_GAP: f32 = 6.0;
/// How far in from the window's edge a resize can start. The scroll bar is
/// on the right edge, so this stays narrow enough to leave most of it.
const EDGE: f32 = 5.0;
/// A corner's grip, along each edge.
const CORNER: f32 = 14.0;

/// Whether this desktop gets the app's title bar. `VINO_UI_TITLE_BAR=app` or
/// `system` decides; otherwise every desktop but a tiling one, where windows
/// have no title bars at all.
pub fn app_title_bar() -> bool {
    match std::env::var("VINO_UI_TITLE_BAR").as_deref() {
        Ok("app") => return true,
        Ok("system") => return false,
        _ => {}
    }
    !tiling(&std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default())
}

fn tiling(desktops: &str) -> bool {
    const TILING: [&str; 6] = ["sway", "i3", "Hyprland", "niri", "river", "qtile"];
    desktops
        .split(':')
        .any(|name| TILING.iter().any(|t| name.eq_ignore_ascii_case(t)))
}

/// The window without the desktop's decorations when the app draws its own.
pub fn viewport(builder: ViewportBuilder, app_title_bar: bool) -> ViewportBuilder {
    builder.with_decorations(!app_title_bar)
}

/// Whether the window fills the screen, maximised or full screen: it then
/// has no edges to resize from and no outline.
pub fn maximised(ui: &Ui) -> bool {
    ui.input(|input| {
        let viewport = input.viewport();
        viewport.maximized.unwrap_or(false) || viewport.fullscreen.unwrap_or(false)
    })
}

/// Makes `rect` the handle the window moves by: a press and drag moves it, a
/// double click maximises or restores it. Call it before drawing what is in
/// `rect`, so the buttons and tabs there still take their own clicks.
pub fn drag(ui: &Ui, rect: Rect, id: &str) -> Response {
    let response = ui.interact(
        rect,
        Id::unique(("vino-window-drag", id)),
        Sense::click_and_drag(),
    );
    if response.double_clicked() {
        let maximised = maximised(ui);
        ui.ctx()
            .send_viewport_cmd(ViewportCommand::Maximized(!maximised));
    } else if response.drag_started_by(egui::PointerButton::Primary) {
        ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
    }
    response
}

#[derive(Clone, Copy)]
enum Action {
    Minimise,
    Maximise,
    Restore,
    Close,
}

impl Action {
    fn name(self) -> &'static str {
        match self {
            Action::Minimise => "Minimise",
            Action::Maximise => "Maximise",
            Action::Restore => "Restore",
            Action::Close => "Close",
        }
    }

    // On the icons' 24-unit grid, a little lighter than they are.
    fn path(self) -> &'static str {
        match self {
            Action::Minimise => "M7 12h10",
            Action::Maximise => "M7 7h10v10H7z",
            Action::Restore => "M7 10h7v7H7z M10 10V7h7v7h-3",
            Action::Close => "M7 7l10 10M17 7 7 17",
        }
    }
}

/// Minimise, maximise (or restore) and close, right to left from the end of
/// the space they are given, centred on its height.
pub fn controls(ui: &mut Ui) {
    let maximise = if maximised(ui) {
        Action::Restore
    } else {
        Action::Maximise
    };
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = BUTTON_GAP;
        for action in [Action::Close, maximise, Action::Minimise] {
            if button(ui, action).clicked() {
                ui.ctx().send_viewport_cmd(match action {
                    Action::Minimise => ViewportCommand::Minimized(true),
                    Action::Maximise => ViewportCommand::Maximized(true),
                    Action::Restore => ViewportCommand::Maximized(false),
                    Action::Close => ViewportCommand::Close,
                });
            }
        }
    });
}

/// How far the buttons sit from the window's top and right edges in [`bar`]:
/// up in the corner, where a desktop's own are, and clear of the strip along
/// each edge that resizes the window ([`edges`]).
pub const BAR_GAP: f32 = 8.0;
/// How much lower than the buttons the bar's title starts. With `BAR_GAP`
/// it leaves the title's row centred on the sidebar logo's.
const TITLE_DROP: f32 = 7.0;
/// The least room between the bar's title and what is beside the buttons.
const TITLE_GAP: f32 = 12.0;

/// A wide window's bar, above the page and beside the sidebar: the page's
/// `title` at `inset` from the left, [`controls`] in the top right corner,
/// and `beside` to their left (a status chip, or nothing). All of it drags
/// the window. A top panel of `root`: show it after the sidebar's panel and
/// before the page's.
pub fn bar(root: &mut Ui, inset: f32, title: impl FnOnce(&mut Ui), beside: impl FnOnce(&mut Ui)) {
    let p = theme::palette(root.ctx());
    egui::Panel::top("window-bar")
        .show_separator_line(false)
        .frame(egui::Frame::NONE.fill(p.bg).inner_margin(egui::Margin {
            left: inset as i8,
            right: BAR_GAP as i8,
            top: BAR_GAP as i8,
            bottom: 6,
        }))
        .show(root, |ui| {
            drag(ui, ui.clip_rect(), "bar");
            ui.style_mut().interaction.selectable_labels = false;
            // The buttons and what is beside them first, from the right, so the
            // title gets what is left and is cut off before it reaches them.
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                ui.allocate_ui_with_layout(
                    vec2(controls_width(), BUTTON),
                    Layout::right_to_left(Align::Center),
                    controls,
                );
                ui.add_space(4.0);
                // As high as a button, so what is beside them is centred on them.
                ui.allocate_ui_with_layout(
                    vec2(ui.available_width(), BUTTON),
                    Layout::right_to_left(Align::Center),
                    beside,
                );
                ui.add_space(TITLE_GAP);
                ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                    let clip = ui.max_rect().intersect(ui.clip_rect());
                    ui.set_clip_rect(clip);
                    ui.vertical(|ui| {
                        ui.add_space(TITLE_DROP);
                        ui.horizontal(title);
                    });
                });
            });
        });
}

/// [`controls`] up in the window's top right corner, [`BAR_GAP`] from its
/// top and right edges, for a window with a header at every width and no
/// [`bar`]. They take no room in `ui`'s layout: keep the width this returns
/// clear at the right end of the header's rows that reach the corner, measured
/// from the window's right edge.
pub fn corner(ui: &mut Ui) -> f32 {
    let window = ui.ctx().content_rect();
    let room = BAR_GAP + controls_width();
    let rect = Rect::from_min_size(
        pos2(window.right() - room, window.top() + BAR_GAP),
        vec2(controls_width(), BUTTON),
    );
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::right_to_left(Align::Center)),
    );
    child.set_clip_rect(rect.expand(4.0));
    controls(&mut child);
    room
}

/// The width [`controls`] takes, for laying out what sits beside them.
pub fn controls_width() -> f32 {
    3.0 * BUTTON + 2.0 * BUTTON_GAP
}

fn button(ui: &mut Ui, action: Action) -> Response {
    let p = theme::palette(ui.ctx());
    let (rect, response) = ui.allocate_exact_size(vec2(BUTTON, BUTTON), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(Role::Button, true, action.name()));
    let response = response.on_hover_text(action.name());
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let pressed = response.is_pointer_button_down_on();
    let hovered = response.hovered();
    // Close turns the destructive red under the pointer, as a desktop's does;
    // the others are ghost buttons.
    let (fill, color) = match action {
        Action::Close if pressed => (p.danger.lerp_to_gamma(Color32::BLACK, 0.18), p.danger_text),
        Action::Close if hovered => (p.danger, p.danger_text),
        _ if pressed => (p.pressed, p.text),
        _ if hovered => (p.field, p.text),
        _ => (Color32::TRANSPARENT, p.muted),
    };
    let painter = ui.painter();
    painter.rect_filled(rect, BUTTON / 2.0, fill);
    let glyph = Rect::from_center_size(rect.center(), vec2(icons::SIZE, icons::SIZE));
    icons::paint_path(painter, glyph, action.path(), 1.6, color);
    if response.has_focus() {
        painter.circle_stroke(
            rect.center(),
            BUTTON / 2.0 + 2.0,
            Stroke::new(2.0_f32, p.accent),
        );
    }
    response
}

/// The window's edges and corners to resize it from, and a hairline round it
/// so it stands apart from a window of the same colour behind: the desktop
/// draws neither without its decorations. Call it last in the frame, so the
/// edges are on top of what is under them. Nothing when maximised.
pub fn edges(ui: &Ui) {
    if maximised(ui) {
        return;
    }
    use egui::ResizeDirection as D;
    let r = ui.ctx().content_rect();
    let (l, t, rt, b) = (r.left(), r.top(), r.right(), r.bottom());
    let grips = [
        (
            D::NorthWest,
            Rect::from_min_max(pos2(l, t), pos2(l + CORNER, t + CORNER)),
        ),
        (
            D::NorthEast,
            Rect::from_min_max(pos2(rt - CORNER, t), pos2(rt, t + CORNER)),
        ),
        (
            D::SouthWest,
            Rect::from_min_max(pos2(l, b - CORNER), pos2(l + CORNER, b)),
        ),
        (
            D::SouthEast,
            Rect::from_min_max(pos2(rt - CORNER, b - CORNER), pos2(rt, b)),
        ),
        (D::North, Rect::from_min_max(pos2(l, t), pos2(rt, t + EDGE))),
        (D::South, Rect::from_min_max(pos2(l, b - EDGE), pos2(rt, b))),
        (D::West, Rect::from_min_max(pos2(l, t), pos2(l + EDGE, b))),
        (D::East, Rect::from_min_max(pos2(rt - EDGE, t), pos2(rt, b))),
    ];
    // Where a corner and an edge overlap, the corner: egui gives the pointer
    // to the last one registered, so the edges go in first.
    for (i, (direction, rect)) in grips.into_iter().enumerate().rev() {
        let response = ui.interact(rect, Id::unique(("vino-window-edge", i)), Sense::drag());
        if response.hovered() || response.dragged() {
            ui.ctx().set_cursor_icon(cursor(direction));
        }
        if response.drag_started_by(egui::PointerButton::Primary) {
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::BeginResize(direction));
        }
    }
    let p = theme::palette(ui.ctx());
    ui.ctx()
        .layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            Id::unique("vino-window-outline"),
        ))
        .rect_stroke(
            r,
            0.0,
            Stroke::new(1.0_f32, p.rule_strong),
            StrokeKind::Inside,
        );
}

fn cursor(direction: egui::ResizeDirection) -> CursorIcon {
    use egui::ResizeDirection as D;
    match direction {
        D::North => CursorIcon::ResizeNorth,
        D::South => CursorIcon::ResizeSouth,
        D::West => CursorIcon::ResizeWest,
        D::East => CursorIcon::ResizeEast,
        D::NorthWest => CursorIcon::ResizeNorthWest,
        D::NorthEast => CursorIcon::ResizeNorthEast,
        D::SouthWest => CursorIcon::ResizeSouthWest,
        D::SouthEast => CursorIcon::ResizeSouthEast,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiling_compositors_keep_no_title_bar() {
        assert!(tiling("sway"));
        assert!(tiling("Hyprland"));
        assert!(tiling("niri:GNOME"));
        assert!(!tiling("KDE"));
        assert!(!tiling("GNOME"));
        assert!(!tiling("ubuntu:GNOME"));
        assert!(!tiling(""));
    }
}
