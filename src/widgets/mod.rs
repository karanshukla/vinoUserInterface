//! The parts every window is made of, one function each. Everything painted
//! here says what it is to a screen reader: a switch is a switch with the
//! row's title for its name, a nav item is a tab, a chip reads as its text.

use egui::{Color32, FontFamily, Painter, Rect, Stroke, StrokeKind, Ui, Vec2, vec2};

use crate::theme::{self, Palette};

mod banners;
mod buttons;
mod choices;
mod lists;
mod navigation;
mod page;
mod rows;
mod status;
mod text;

pub use banners::*;
pub use buttons::*;
pub use choices::*;
pub use lists::*;
pub use navigation::*;
pub use page::*;
pub use rows::*;
pub use status::*;
pub use text::*;

/// A button, field or combo box is this tall.
pub const CONTROL_HEIGHT: f32 = 30.0;
/// A small button, as in a list row.
pub const SMALL_HEIGHT: f32 = 26.0;
/// Below this width a row's control goes under its label.
pub const NARROW: f32 = 440.0;
pub const SIDEBAR_WIDTH: f32 = 200.0;
/// The content's left margin beside the sidebar.
pub const PAGE_INSET: f32 = 28.0;
/// The content column's widest.
pub const CONTENT_MAX: f32 = 620.0;
/// Between the content and the scroll bar on the window's edge.
pub const SCROLL_GAP: f32 = 24.0;
/// The page scroll bar's width.
pub const SCROLL_BAR: f32 = 10.0;
/// Below this window width the sidebar folds into underline tabs: the width
/// at which the sidebar still leaves rows wide enough to keep their controls
/// beside their labels. Half a 1366 or 1280 screen is under it, so a window
/// snapped to one side gets tabs and wide rows, not a sidebar and stacked ones.
pub const SIDEBAR_FROM: f32 = SIDEBAR_WIDTH + PAGE_INSET + NARROW + SCROLL_GAP + SCROLL_BAR;
/// The sidebar's widest, and [`PAGE_INSET`]'s: both grow from their
/// narrowest at [`SIDEBAR_FROM`] to these at [`FLUID_TO`].
pub const SIDEBAR_MAX: f32 = 240.0;
pub const PAGE_INSET_MAX: f32 = 40.0;
/// The window width at which the sidebar and the page inset stop growing.
pub const FLUID_TO: f32 = 1600.0;
/// Between the two columns of a [`columns`] page.
pub const COLUMN_GAP: f32 = 24.0;
/// The narrowest page that [`columns`] splits in two: each column still
/// leaves its rows `NARROW` wide, so no control goes under its label.
pub const TWO_COLUMNS_FROM: f32 = 2.0 * NARROW + COLUMN_GAP;
/// The widest a page of [`columns`] gets: two `CONTENT_MAX` columns. Give it
/// to that page's [`page_scroll`] in place of `CONTENT_MAX`.
pub const WIDE_MAX: f32 = 2.0 * CONTENT_MAX + COLUMN_GAP;
const GROUP_RADIUS: f32 = 10.0;
/// In a narrow row, the room a label needs beside its control before the
/// control goes under it instead.
const LABEL_MIN: f32 = 160.0;
/// Inside a row, a banner or a notice: 16 beside, 12 above and below.
const ROW_PADDING: Vec2 = vec2(16.0, 12.0);
/// An icon on a body line or in a button.
const ICON: f32 = 16.0;
/// Either side of a tag's text.
const TAG_PAD: f32 = 6.0;
/// In a nav item: the icon's centre, and where the text starts.
const NAV_ICON_CENTRE: f32 = 18.0;
const NAV_TEXT: f32 = 36.0;

fn palette(ui: &Ui) -> &'static Palette {
    theme::palette(ui.ctx())
}

fn faded(color: Color32, enabled: bool) -> Color32 {
    if enabled {
        color
    } else {
        color.gamma_multiply(0.45)
    }
}

/// The text weight of a choice: semibold for the one that is on or selected.
fn weight(selected: bool) -> FontFamily {
    if selected {
        theme::semibold()
    } else {
        theme::medium()
    }
}

/// The keyboard focus ring: 2 px of accent inside `rect`.
fn paint_focus_ring(painter: &Painter, rect: Rect, radius: f32, p: &Palette) {
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(2.0_f32, p.accent),
        StrokeKind::Inside,
    );
}

#[cfg(test)]
mod tests;
