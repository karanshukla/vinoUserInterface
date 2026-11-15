//! The page itself: its scroll area, its columns, and how the frame round it
//! grows with the window.

use egui::{Align, Color32, CornerRadius, Layout, Rect, Ui, UiBuilder, pos2, vec2};

use super::{
    COLUMN_GAP, CONTENT_MAX, FLUID_TO, PAGE_INSET, PAGE_INSET_MAX, SCROLL_GAP, SIDEBAR_FROM,
    SIDEBAR_MAX, SIDEBAR_WIDTH, TWO_COLUMNS_FROM, WIDE_MAX, palette,
};
use crate::a11y;

/// Fades the last lines of a scroll area out while more is below: the bar
/// alone is easy to miss.
pub fn fade_below<R>(ui: &Ui, scrolled: &egui::scroll_area::ScrollAreaOutput<R>, bg: Color32) {
    let view = scrolled.inner_rect;
    if scrolled.state.clamped_offset().y + view.height() >= scrolled.content_size.y - 1.0 {
        return;
    }
    let fade = Rect::from_min_max(pos2(view.left(), view.bottom() - 48.0), view.right_bottom());
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(fade.left_top(), Color32::TRANSPARENT);
    mesh.colored_vertex(fade.right_top(), Color32::TRANSPARENT);
    mesh.colored_vertex(fade.right_bottom(), bg);
    mesh.colored_vertex(fade.left_bottom(), bg);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    ui.painter().add(mesh);
}

/// The vertical scroll area every page uses: a solid bar always shown, the
/// drag-to-scroll area (for touch) off since it took a Tab stop, and the
/// content kept off the bar. `max_width` is `CONTENT_MAX`, or `WIDE_MAX` for a
/// page of [`columns`], which stays at `CONTENT_MAX` until it is wide enough to
/// split, or anything else for content that is the point (`f32::INFINITY` for
/// full width), which is never capped.
pub fn page_scroll<R>(
    ui: &mut Ui,
    what: &str,
    max_width: f32,
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    let p = palette(ui);
    let scrolled = egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
        .scroll_source(egui::scroll_area::ScrollSource {
            drag: egui::scroll_area::DragScroll::Never,
            ..Default::default()
        })
        .show(ui, |ui| {
            // The scroll area already keeps its bar `bar_inner_margin` off the
            // content; the rest of SCROLL_GAP comes off here, so the gap is
            // SCROLL_GAP and SIDEBAR_FROM leaves a row exactly NARROW wide.
            let gap = (SCROLL_GAP - ui.spacing().scroll.bar_inner_margin).max(0.0);
            let width = ui.available_width() - gap;
            // Too narrow for two columns, a WIDE_MAX page is one column like
            // any other. Only WIDE_MAX: full-width content stays full width.
            let max_width = if max_width == WIDE_MAX && width < TWO_COLUMNS_FROM {
                max_width.min(CONTENT_MAX)
            } else {
                max_width
            };
            ui.set_max_width(width.min(max_width));
            let out = add(ui);
            ui.add_space(16.0);
            out
        });
    a11y::name_scrolling(ui, &scrolled, what);
    fade_below(ui, &scrolled, p.bg);
    scrolled.inner
}

/// How far a window this wide is from `SIDEBAR_FROM` to `FLUID_TO`, 0 to 1.
fn fluid(window: f32) -> f32 {
    ((window - SIDEBAR_FROM) / (FLUID_TO - SIDEBAR_FROM)).clamp(0.0, 1.0)
}

/// The sidebar's width in a window this wide: [`SIDEBAR_WIDTH`] at
/// [`SIDEBAR_FROM`], growing with the window to [`SIDEBAR_MAX`]. Whole
/// points, so the panel's edge stays on a pixel.
pub fn sidebar_width(window: f32) -> f32 {
    (SIDEBAR_WIDTH + (SIDEBAR_MAX - SIDEBAR_WIDTH) * fluid(window)).round()
}

/// The page's left margin beside the sidebar in a window this wide:
/// [`PAGE_INSET`] growing to [`PAGE_INSET_MAX`], as [`sidebar_width`] does.
pub fn page_inset(window: f32) -> f32 {
    (PAGE_INSET + (PAGE_INSET_MAX - PAGE_INSET) * fluid(window)).round()
}

/// One of a [`columns`] page's two columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Column {
    Left,
    Right,
}

/// A page's groups in two columns side by side when the page is at least
/// [`TWO_COLUMNS_FROM`] wide, the left one above the right otherwise. `add`
/// is called once per column, left first, so both can borrow the same state.
/// Each column stops at `CONTENT_MAX`, as a one-column page does. Give the
/// page's [`page_scroll`] [`WIDE_MAX`] so it has the room. Text size counts:
/// at 2x zoom a page needs twice the window to split.
pub fn columns<R>(ui: &mut Ui, mut add: impl FnMut(&mut Ui, Column) -> R) -> [R; 2] {
    let width = ui.available_width();
    if width < TWO_COLUMNS_FROM {
        return [Column::Left, Column::Right].map(|column| {
            ui.scope(|ui| {
                ui.set_max_width(width.min(CONTENT_MAX));
                add(ui, column)
            })
            .inner
        });
    }
    let column_w = ((width - COLUMN_GAP) / 2.0).min(CONTENT_MAX);
    let top_left = ui.available_rect_before_wrap().min;
    let mut height: f32 = 0.0;
    let out = [Column::Left, Column::Right].map(|column| {
        let x = match column {
            Column::Left => 0.0,
            Column::Right => column_w + COLUMN_GAP,
        };
        let mut child = ui.new_child(
            UiBuilder::new()
                .max_rect(Rect::from_min_size(
                    pos2(top_left.x + x, top_left.y),
                    vec2(column_w, f32::INFINITY),
                ))
                .layout(Layout::top_down(Align::Min)),
        );
        let out = add(&mut child, column);
        height = height.max(child.min_rect().height());
        out
    });
    ui.advance_cursor_after_rect(Rect::from_min_size(
        top_left,
        vec2(2.0 * column_w + COLUMN_GAP, height),
    ));
    out
}

/// Whatever `CornerRadius` a group's own corner needs.
pub fn corner(radius: f32) -> CornerRadius {
    CornerRadius::same(radius as u8)
}
