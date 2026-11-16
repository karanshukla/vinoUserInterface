//! A group and the rows in it.

use std::cell::RefCell;

use egui::{
    Align, Color32, FontId, Id, Layout, Rect, Response, RichText, Sense, Shape, Stroke, StrokeKind,
    Ui, UiBuilder, pos2, vec2,
};

use super::{CONTROL_HEIGHT, GROUP_RADIUS, LABEL_MIN, NARROW, ROW_PADDING, Tone, palette};
use crate::theme;

/// Between a row's label and the control beside it.
const CONTROL_GAP: f32 = 16.0;
/// Between a row's label and the controls stacked under it.
const STACK_GAP: f32 = 10.0;
/// A row's content and its label are never laid out narrower than these.
const MIN_CONTENT_WIDTH: f32 = 40.0;
const MIN_LABEL_WIDTH: f32 = 80.0;
/// A row drawn for the first time guesses its control at this width, and
/// leaves its label no less.
const UNSEEN_CONTROL_WIDTH: f32 = 120.0;
/// Left of stacked controls, so a rounding error doesn't wrap the last one.
const WRAP_SLACK: f32 = 1.0;

/// A surface with a 1 px rule and 10 px corners, holding [`Row`]s separated
/// by hairlines, on the page. The corners stay round whatever a row paints.
pub fn group<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    group_on(ui, palette(ui).bg, add)
}

/// A [`group`] on something other than the page, a side panel's surface say:
/// what a tinted row paints over the corners is masked in `ground`.
pub fn group_on<R>(ui: &mut Ui, ground: Color32, add: impl FnOnce(&mut Ui) -> R) -> R {
    let p = palette(ui);
    let outer = ui.available_rect_before_wrap();
    let fill = ui.painter().add(Shape::Noop);
    // Rows run to the group's edge and the rule is painted inside it, over
    // their padding's outermost pixel, so text lines up with a banner's.
    let inner_rect = Rect::from_min_size(outer.min, vec2(outer.width(), f32::INFINITY));
    let inner = ui.scope_builder(
        UiBuilder::new()
            .max_rect(inner_rect)
            .layout(Layout::top_down(Align::Min)),
        |ui| {
            ui.set_width(inner_rect.width());
            ui.spacing_mut().item_spacing.y = 0.0;
            add(ui)
        },
    );
    let bottom = inner.response.rect.bottom().max(outer.top() + 2.0);
    let rect = Rect::from_min_max(outer.min, pos2(outer.right(), bottom));
    let painter = ui.painter();
    painter.set(fill, Shape::rect_filled(rect, GROUP_RADIUS, p.surface));
    mask_corners(painter, rect, ground);
    painter.rect_stroke(
        rect,
        GROUP_RADIUS,
        Stroke::new(1.0_f32, p.rule),
        StrokeKind::Inside,
    );
    ui.allocate_rect(rect, Sense::hover());
    inner.inner
}

/// Paints `ground` over what a row painted outside the group's round corners.
fn mask_corners(painter: &egui::Painter, rect: Rect, ground: Color32) {
    const ARC_STEPS: usize = 8;
    let radius = GROUP_RADIUS;
    for (corner, dx, dy) in [
        (rect.left_top(), 1.0, 1.0),
        (rect.right_top(), -1.0, 1.0),
        (rect.right_bottom(), -1.0, -1.0),
        (rect.left_bottom(), 1.0, -1.0),
    ] {
        let center = corner + vec2(dx * radius, dy * radius);
        let mut points = vec![corner];
        for step in 0..=ARC_STEPS {
            let angle = step as f32 / ARC_STEPS as f32 * std::f32::consts::FRAC_PI_2;
            points.push(center - vec2(dx * radius * angle.cos(), dy * radius * angle.sin()));
        }
        painter.add(Shape::convex_polygon(points, ground, Stroke::NONE));
    }
}

/// A row in a [`group`]: title and help on the left, the control on the
/// right (under the label below [`NARROW`]), a hairline above it unless it
/// is the first. The control is named by the title for screen readers when
/// it is passed through the function the closure is given.
pub struct Row<'a> {
    title: &'a str,
    help: &'a str,
    tint: Option<Color32>,
    /// The help's colour: muted, or the warning or error colour when the
    /// row needs attention.
    tone: Tone,
}

impl<'a> Row<'a> {
    pub fn new(title: &'a str) -> Self {
        Row {
            title,
            help: "",
            tint: None,
            tone: Tone::Neutral,
        }
    }

    pub fn help(mut self, help: &'a str) -> Self {
        self.help = help;
        self
    }

    /// Paint the row in a colour: the error tint while a confirmation waits.
    pub fn tint(mut self, tint: Color32) -> Self {
        self.tint = Some(tint);
        self
    }

    /// Colour the help: `Tone::Warn` for something that needs the user,
    /// `Tone::Error` for something that failed.
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    pub fn show<R>(
        self,
        ui: &mut Ui,
        control: impl FnOnce(&mut Ui, &dyn Fn(Response) -> Response) -> R,
    ) -> R {
        let p = palette(ui);
        let title = self.title;
        let help = self.help;
        let help_color = match self.tone {
            Tone::Warn => p.warn,
            Tone::Error => p.error,
            Tone::Info | Tone::Neutral => p.muted,
        };
        let estimate = |ui: &Ui, width: f32| -> f32 {
            let title_h = ui
                .painter()
                .layout(
                    title.to_owned(),
                    FontId::new(14.0, theme::medium()),
                    p.text,
                    width,
                )
                .size()
                .y;
            if help.is_empty() {
                title_h
            } else {
                let help_h = ui
                    .painter()
                    .layout(help.to_owned(), FontId::proportional(12.5), p.muted, width)
                    .size()
                    .y;
                title_h + 2.0 + help_h
            }
        };
        let label = move |ui: &mut Ui| -> Option<Id> {
            ui.spacing_mut().item_spacing.y = 2.0;
            let shown = ui.add(
                egui::Label::new(RichText::new(title).size(14.0).family(theme::medium())).wrap(),
            );
            if !help.is_empty() {
                ui.add(egui::Label::new(RichText::new(help).size(12.5).color(help_color)).wrap());
            }
            Some(shown.id)
        };
        show_row(ui, self.tint, &estimate, label, control)
    }

    /// A row whose left side is drawn by `label` instead of a title: a path
    /// with an icon, a stat strip. Controls go unnamed unless `label`
    /// returns an id to name them by.
    pub fn custom<R>(
        self,
        ui: &mut Ui,
        label: impl FnOnce(&mut Ui) -> Option<Id>,
        control: impl FnOnce(&mut Ui, &dyn Fn(Response) -> Response) -> R,
    ) -> R {
        show_row(ui, self.tint, &|_, _| CONTROL_HEIGHT, label, control)
    }
}

/// Where a row's label and controls go, and what it remembers between frames.
#[derive(Clone, Copy)]
struct RowSpace {
    left: f32,
    top: f32,
    /// The width inside the row's padding.
    content_width: f32,
    salt: Id,
    /// Where the controls' width is kept for the next frame.
    width_id: Id,
    /// Last frame's: a row seen for the first time has none, and stacks.
    last_control_width: Option<f32>,
}

/// What laying a row out gives back: the control closure's result, the id
/// to name the controls by, and the height of the content between the padding.
type RowContent<R> = (R, Option<Id>, f32);

fn show_row<R>(
    ui: &mut Ui,
    tint: Option<Color32>,
    estimate: &dyn Fn(&Ui, f32) -> f32,
    label: impl FnOnce(&mut Ui) -> Option<Id>,
    control: impl FnOnce(&mut Ui, &dyn Fn(Response) -> Response) -> R,
) -> R {
    let p = palette(ui);
    let full = ui.available_rect_before_wrap();
    let top = ui.cursor().top();
    let first = top <= ui.max_rect().top() + 0.5;
    let bg = ui.painter().add(Shape::Noop);
    let line = ui.painter().add(Shape::Noop);
    let salt = ui.next_auto_id();
    let width_id = salt.with("control width");
    let space = RowSpace {
        left: full.left() + ROW_PADDING.x,
        top,
        content_width: (full.width() - 2.0 * ROW_PADDING.x).max(MIN_CONTENT_WIDTH),
        salt,
        width_id,
        last_control_width: ui.data(|d| d.get_temp::<f32>(width_id)),
    };
    let stacked = stacks(full.width(), space.content_width, space.last_control_width);
    let controls: RefCell<Vec<Id>> = RefCell::new(Vec::new());
    let named = |response: Response| -> Response {
        controls.borrow_mut().push(response.id);
        response
    };
    let control = |ui: &mut Ui| control(ui, &named);
    let (out, label_id, content_height) = if stacked {
        stacked_row(ui, &space, label, control)
    } else {
        beside_row(ui, &space, estimate, label, control)
    };
    if let Some(label_id) = label_id {
        name_controls(ui, &controls.borrow(), label_id);
    }
    let rect = Rect::from_min_size(
        full.min,
        vec2(full.width(), content_height + 2.0 * ROW_PADDING.y),
    );
    let painter = ui.painter();
    if let Some(tint) = tint {
        painter.set(bg, Shape::rect_filled(rect, 0.0, tint));
    }
    if !first {
        painter.set(
            line,
            Shape::line_segment(
                [rect.left_top(), rect.right_top()],
                Stroke::new(1.0_f32, p.rule),
            ),
        );
    }
    ui.allocate_rect(rect, Sense::hover());
    out
}

/// Whether a row's control goes under its label: in a row under [`NARROW`],
/// unless the control is small enough (a switch, a number) to leave the label
/// [`LABEL_MIN`] beside it.
fn stacks(row_width: f32, content_width: f32, last_control_width: Option<f32>) -> bool {
    let fits_beside = |control_width: f32| content_width - control_width - CONTROL_GAP >= LABEL_MIN;
    row_width < NARROW && !last_control_width.is_some_and(fits_beside)
}

/// The label, then the controls under it. They are laid out from the right,
/// as beside the label, so they keep their order: in a box as wide as they
/// were last frame, pushed to the left edge.
fn stacked_row<R>(
    ui: &mut Ui,
    space: &RowSpace,
    label: impl FnOnce(&mut Ui) -> Option<Id>,
    control: impl FnOnce(&mut Ui) -> R,
) -> RowContent<R> {
    let RowSpace {
        salt,
        width_id,
        last_control_width,
        ..
    } = *space;
    let label_top = space.top + ROW_PADDING.y;
    let label_rect = Rect::from_min_size(
        pos2(space.left, label_top),
        vec2(space.content_width, f32::INFINITY),
    );
    let mut label_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(label_rect)
            .layout(Layout::top_down(Align::Min))
            .id_salt(salt.with("label")),
    );
    let label_id = label(&mut label_ui);
    let label_height = label_ui.min_rect().height().max(0.0);
    let gap = if label_height > 0.0 { STACK_GAP } else { 0.0 };
    let control_top = space.top + ROW_PADDING.y + label_height + gap;
    let box_width = last_control_width
        .filter(|width| *width > 0.0)
        .map_or(space.content_width, |width| width.min(space.content_width));
    let control_rect = Rect::from_min_max(
        pos2(space.left - WRAP_SLACK, control_top),
        pos2(space.left + box_width, control_top + CONTROL_HEIGHT),
    );
    let mut control_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(control_rect)
            .layout(Layout::right_to_left(Align::Center).with_main_wrap(true))
            .id_salt(salt.with("control")),
    );
    let out = control(&mut control_ui);
    let control_width = control_ui.min_rect().width().max(0.0);
    remember_control_width(ui, width_id, last_control_width, control_width);
    let control_height = control_ui.min_rect().height().max(0.0);
    let bottom = if control_height > 0.0 {
        control_top + control_height.max(CONTROL_HEIGHT)
    } else {
        space.top + ROW_PADDING.y + label_height
    };
    (out, label_id, bottom - space.top - ROW_PADDING.y)
}

/// Keeps this frame's control width for the next, and redraws at once when
/// it is new or changed, so the control lands beside the label if it fits
/// there now, or in a box of its new width.
fn remember_control_width(ui: &Ui, width_id: Id, last: Option<f32>, now: f32) {
    ui.data_mut(|d| d.insert_temp(width_id, now));
    let changed = last.is_none_or(|width| (width - now).abs() > 0.5);
    if now > 0.0 && changed {
        ui.ctx().request_discard("a row's control changed width");
    }
}

/// The control first, centred on the label's likely height, measured at the
/// width last frame's control left it; then the label in what is left.
fn beside_row<R>(
    ui: &mut Ui,
    space: &RowSpace,
    estimate: &dyn Fn(&Ui, f32) -> f32,
    label: impl FnOnce(&mut Ui) -> Option<Id>,
    control: impl FnOnce(&mut Ui) -> R,
) -> RowContent<R> {
    let RowSpace {
        content_width,
        salt,
        width_id,
        last_control_width,
        ..
    } = *space;
    let likely_label_width = last_control_width.map_or(
        (content_width - UNSEEN_CONTROL_WIDTH).max(UNSEEN_CONTROL_WIDTH),
        |width| (content_width - width - gap_beside(width)).max(MIN_LABEL_WIDTH),
    );
    let likely_label_height = estimate(ui, likely_label_width);
    let row_height = likely_label_height.max(CONTROL_HEIGHT);
    let content_rect = Rect::from_min_size(
        pos2(space.left, space.top + ROW_PADDING.y),
        vec2(content_width, row_height),
    );
    let mut control_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(content_rect)
            .layout(Layout::right_to_left(Align::Center).with_main_wrap(true))
            .id_salt(salt.with("control")),
    );
    let out = control(&mut control_ui);
    let control_rect = control_ui.min_rect();
    let (control_width, control_height) = if control_rect.is_positive() {
        (control_rect.width(), control_rect.height())
    } else {
        (0.0, 0.0)
    };
    ui.data_mut(|d| d.insert_temp(width_id, control_width));
    let label_width =
        (content_width - control_width - gap_beside(control_width)).max(MIN_LABEL_WIDTH);
    let label_offset = ((row_height - likely_label_height) / 2.0).max(0.0);
    let label_rect = Rect::from_min_size(
        pos2(space.left, content_rect.top() + label_offset),
        vec2(label_width, f32::INFINITY),
    );
    let mut label_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(label_rect)
            .layout(Layout::top_down(Align::Min))
            .id_salt(salt.with("label")),
    );
    let label_id = label(&mut label_ui);
    let label_height = label_ui.min_rect().height().max(0.0);
    let content_height = row_height
        .max(label_offset + label_height)
        .max(control_height);
    (out, label_id, content_height)
}

/// The gap between a label and the control beside it; none without a control.
fn gap_beside(control_width: f32) -> f32 {
    if control_width > 0.0 {
        CONTROL_GAP
    } else {
        0.0
    }
}

/// As `Response::labelled_by`: the label wins over the widget's own text.
fn name_controls(ui: &Ui, controls: &[Id], label_id: Id) {
    for id in controls {
        ui.ctx().accesskit_node_builder(*id, |node| {
            node.clear_label();
            node.push_labelled_by(label_id.accesskit_id());
        });
    }
}

/// Free-form content inside a [`group`], padded like a row: a stats strip, a
/// preview, an adder line.
pub fn group_section<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    let p = palette(ui);
    let full = ui.available_rect_before_wrap();
    let top = ui.cursor().top();
    let first = top <= ui.max_rect().top() + 0.5;
    if !first {
        ui.painter().hline(
            full.left()..=full.right(),
            top,
            Stroke::new(1.0_f32, p.rule),
        );
    }
    let inner = ui.scope_builder(
        UiBuilder::new()
            .max_rect(Rect::from_min_size(
                full.min + ROW_PADDING,
                vec2(full.width() - 2.0 * ROW_PADDING.x, f32::INFINITY),
            ))
            .layout(Layout::top_down(Align::Min)),
        |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            add(ui)
        },
    );
    let bottom = inner.response.rect.bottom() + ROW_PADDING.y;
    ui.allocate_rect(
        Rect::from_min_max(full.min, pos2(full.right(), bottom)),
        Sense::hover(),
    );
    inner.inner
}

#[cfg(test)]
mod tests {
    use super::*;

    fn content_width(row_width: f32) -> f32 {
        row_width - 2.0 * ROW_PADDING.x
    }

    #[test]
    fn a_row_stacks_under_narrow_and_not_at_it() {
        let wide_control = Some(300.0);
        assert!(!stacks(NARROW, content_width(NARROW), wide_control));
        let under = NARROW - 1.0;
        assert!(stacks(under, content_width(under), wide_control));
    }

    #[test]
    fn a_narrow_row_keeps_a_control_that_leaves_the_label_its_minimum() {
        let row = 360.0;
        let content = content_width(row);
        let widest_beside = content - CONTROL_GAP - LABEL_MIN;
        assert!(!stacks(row, content, Some(widest_beside)));
        assert!(stacks(row, content, Some(widest_beside + 1.0)));
    }

    #[test]
    fn a_narrow_row_seen_for_the_first_time_stacks() {
        assert!(stacks(360.0, content_width(360.0), None));
        assert!(!stacks(NARROW, content_width(NARROW), None));
    }
}
