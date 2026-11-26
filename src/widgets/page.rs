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
            let width = ui.available_width() - gap_before_bar(ui);
            ui.set_max_width(width.min(cap_for(max_width, width)));
            let out = add(ui);
            ui.add_space(16.0);
            out
        });
    a11y::name_scrolling(ui, &scrolled, what);
    fade_below(ui, &scrolled, p.bg);
    scrolled.inner
}

/// What `SCROLL_GAP` leaves after the `bar_inner_margin` the scroll area
/// already keeps, so the gap is `SCROLL_GAP` and `SIDEBAR_FROM` leaves a row
/// exactly `NARROW` wide (pinned by
/// `widgets::tests::the_sidebar_breakpoint_leaves_a_row_exactly_narrow`).
fn gap_before_bar(ui: &Ui) -> f32 {
    (SCROLL_GAP - ui.spacing().scroll.bar_inner_margin).max(0.0)
}

/// A `WIDE_MAX` page too narrow for two columns is one column like any
/// other; any other cap stays, full width included (pinned by
/// `widgets::tests::only_a_wide_max_page_is_capped_when_narrow`).
fn cap_for(max_width: f32, width: f32) -> f32 {
    if max_width == WIDE_MAX && width < TWO_COLUMNS_FROM {
        max_width.min(CONTENT_MAX)
    } else {
        max_width
    }
}

/// Narrowest horizontal inset on quarter-tiled windows (`340 px` wide).
pub const PAGE_INSET_MIN: f32 = 12.0;
const MIN_WINDOW: f32 = 340.0;

/// Window size classes tuned for laptop splits and desktop monitors:
///
/// - `Compact`: under `560 px` (quarter tile, narrow split, or 2x text scale)
/// - `Medium`: `560..900 px` (half-tiled laptop display)
/// - `Expanded`: `900 px` and up (full laptop or desktop window)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeClass {
    Compact,
    Medium,
    Expanded,
}

/// Upper boundary (exclusive) of [`SizeClass::Compact`].
pub const COMPACT_MAX: f32 = 560.0;
/// Upper boundary (exclusive) of [`SizeClass::Medium`].
pub const MEDIUM_MAX: f32 = 900.0;

impl SizeClass {
    pub fn of(width: f32) -> Self {
        if width < COMPACT_MAX {
            SizeClass::Compact
        } else if width < MEDIUM_MAX {
            SizeClass::Medium
        } else {
            SizeClass::Expanded
        }
    }

    pub fn for_ui(ui: &Ui) -> Self {
        Self::of(ui.ctx().content_rect().width())
    }

    pub fn is_compact(self) -> bool {
        self == SizeClass::Compact
    }
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

/// Fluid horizontal page margin across all window sizes: [`PAGE_INSET_MIN`] at
/// `340 px`, rising smoothly to [`PAGE_INSET`] at [`SIDEBAR_FROM`] and
/// [`PAGE_INSET_MAX`] at [`FLUID_TO`] without shrinking the page as the window
/// widens (pinned by
/// `widgets::tests::content_inset_scales_smoothly_without_shrinking_the_page`).
pub fn content_inset(window: f32) -> f32 {
    if window >= SIDEBAR_FROM {
        page_inset(window)
    } else {
        let t = ((window - MIN_WINDOW) / (SIDEBAR_FROM - MIN_WINDOW)).clamp(0.0, 1.0);
        (PAGE_INSET_MIN + (PAGE_INSET - PAGE_INSET_MIN) * t).round()
    }
}

/// Safe maximum width for a popover or hover card inside the current viewport.
pub fn popup_width(ctx: &egui::Context, max: f32) -> f32 {
    (ctx.content_rect().width() - 32.0).clamp(200.0, max.max(200.0))
}

/// How many columns of at least `min_item_width` (separated by `gap`) fit in
/// `available`, capped at `max_columns` (pinned by
/// `widgets::tests::grid_columns_adapts_from_one_to_max`).
pub fn grid_columns(available: f32, min_item_width: f32, gap: f32, max_columns: usize) -> usize {
    let max = max_columns.max(1);
    let step = (min_item_width + gap).max(1.0);
    let fit = ((available + gap) / step).floor().max(1.0) as usize;
    fit.clamp(1, max)
}

/// A responsive grid of equal-width cells that adapts from `max_columns` down
/// to `1` as the window narrows or text scale increases, passing each cell its
/// allocated width.
pub fn responsive_grid<T>(
    ui: &mut Ui,
    items: &[T],
    min_item_width: f32,
    gap: f32,
    max_columns: usize,
    mut cell: impl FnMut(&mut Ui, &T, f32),
) {
    if items.is_empty() {
        return;
    }
    let width = ui.available_width();
    let cols = grid_columns(width, min_item_width, gap, max_columns);
    let cell_w = ((width - gap * (cols - 1) as f32) / cols as f32).max(1.0);
    let rows = items.chunks(cols);
    let total_rows = rows.len();
    for (row_idx, chunk) in rows.enumerate() {
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for item in chunk {
                ui.allocate_ui_with_layout(vec2(cell_w, 0.0), Layout::top_down(Align::Min), |ui| {
                    ui.set_width(cell_w);
                    cell(ui, item, cell_w);
                });
            }
        });
        if row_idx + 1 < total_rows {
            ui.add_space(gap);
        }
    }
}

/// A header or toolbar row with leading content on the left and trailing
/// actions on the right; below `break_width`, stacks cleanly onto two wrapped
/// left-aligned rows (pinned by
/// `widgets::tests::a_responsive_bar_stacks_below_its_breakpoint`).
pub fn responsive_bar(
    ui: &mut Ui,
    break_width: f32,
    leading: impl FnOnce(&mut Ui),
    trailing: impl FnOnce(&mut Ui),
) {
    if ui.available_width() >= break_width {
        ui.horizontal(|ui| {
            leading(ui);
            ui.with_layout(Layout::right_to_left(Align::Center), trailing);
        });
    } else {
        ui.horizontal_wrapped(leading);
        ui.horizontal_wrapped(trailing);
    }
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
