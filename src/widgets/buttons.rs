//! The five kinds of button, plus connected split buttons.

use egui::{
    Align, Color32, CornerRadius, FontFamily, FontId, Layout, Rect, Response, Role, Sense, Stroke,
    StrokeKind, Ui, WidgetInfo, pos2, vec2,
};

use super::{CONTROL_HEIGHT, ICON, SMALL_HEIGHT, faded, lerp_color, paint_focus_ring, palette};
use crate::icons::{self, Icon};
use crate::theme::{self, Palette};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonKind {
    /// Ink on light, near-white on dark: one per view.
    Primary,
    /// Soft filled container between `Primary` and `Secondary` for contextual
    /// recommended actions without spending the view's single primary button.
    Tonal,
    /// Field-coloured with a rule.
    Secondary,
    /// Bare text until hovered.
    Ghost,
    /// Deep red with white text: only inside a confirmation.
    Destructive,
}

/// Between a button's icon and the text after it.
const ICON_TEXT_GAP: f32 = 6.0;
/// Resting and pressed corner radii for stateful corner morphing.
const REST_RADIUS: f32 = 8.0;
const PRESSED_RADIUS: f32 = 5.0;
/// Inner corner radius and gap between halves of a connected split button.
const SPLIT_INNER_RADIUS: u8 = 2;
const SPLIT_GAP: f32 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Corners {
    All,
    Leading,
    Trailing,
}

impl Corners {
    fn radius(self, outer: u8) -> CornerRadius {
        match self {
            Corners::All => CornerRadius::same(outer),
            Corners::Leading => CornerRadius {
                nw: outer,
                sw: outer,
                ne: SPLIT_INNER_RADIUS,
                se: SPLIT_INNER_RADIUS,
            },
            Corners::Trailing => CornerRadius {
                nw: SPLIT_INNER_RADIUS,
                sw: SPLIT_INNER_RADIUS,
                ne: outer,
                se: outer,
            },
        }
    }
}

struct ButtonStates {
    rest: Color32,
    hovered: Color32,
    pressed: Color32,
    stroke: Stroke,
    text: Color32,
    active_text: Color32,
}

struct ButtonMetrics {
    width: f32,
    height: f32,
    text_w: f32,
    icon_w: f32,
    galley: std::sync::Arc<egui::Galley>,
}

impl ButtonKind {
    /// The fill, outline and text colour at rest, hovered and pressed.
    fn states(self, p: &Palette) -> ButtonStates {
        match self {
            ButtonKind::Primary => ButtonStates {
                rest: p.primary_fill,
                hovered: p.primary_fill.lerp_to_gamma(p.bg, 0.12),
                pressed: p.primary_fill.lerp_to_gamma(p.bg, 0.22),
                stroke: Stroke::NONE,
                text: p.primary_text,
                active_text: p.primary_text,
            },
            ButtonKind::Tonal => ButtonStates {
                rest: p.tonal_fill,
                hovered: p.tonal_fill.lerp_to_gamma(p.primary_fill, 0.08),
                pressed: p.tonal_fill.lerp_to_gamma(p.primary_fill, 0.16),
                stroke: Stroke::NONE,
                text: p.text,
                active_text: p.text,
            },
            ButtonKind::Secondary => ButtonStates {
                rest: p.field,
                hovered: p.hovered(),
                pressed: p.pressed,
                stroke: Stroke::new(1.0_f32, p.rule_strong),
                text: p.text,
                active_text: p.text,
            },
            ButtonKind::Ghost => ButtonStates {
                rest: Color32::TRANSPARENT,
                hovered: p.field,
                pressed: p.pressed,
                stroke: Stroke::NONE,
                text: p.muted,
                active_text: p.text,
            },
            ButtonKind::Destructive => ButtonStates {
                rest: p.danger,
                hovered: p.danger.lerp_to_gamma(Color32::BLACK, 0.1),
                pressed: p.danger.lerp_to_gamma(Color32::BLACK, 0.18),
                stroke: Stroke::NONE,
                text: p.danger_text,
                active_text: p.danger_text,
            },
        }
    }

    fn look(self, p: &Palette, hover_t: f32, press_t: f32) -> (Color32, Stroke, Color32) {
        let s = self.states(p);
        let fill = lerp_color(lerp_color(s.rest, s.hovered, hover_t), s.pressed, press_t);
        let text = lerp_color(s.text, s.active_text, hover_t.max(press_t));
        (fill, s.stroke, text)
    }

    fn text_weight(self) -> FontFamily {
        match self {
            ButtonKind::Primary | ButtonKind::Tonal | ButtonKind::Destructive => theme::semibold(),
            ButtonKind::Secondary | ButtonKind::Ghost => theme::medium(),
        }
    }
}

/// A button in one of the five kinds, 30 tall (26 small), with an optional
/// icon before its text.
pub struct Button {
    kind: ButtonKind,
    text: String,
    icon: Option<Icon>,
    small: bool,
    min_width: f32,
    corners: Corners,
}

impl Button {
    pub fn new(kind: ButtonKind, text: impl Into<String>) -> Self {
        Button {
            kind,
            text: text.into(),
            icon: None,
            small: false,
            min_width: 0.0,
            corners: Corners::All,
        }
    }

    pub fn primary(text: impl Into<String>) -> Self {
        Button::new(ButtonKind::Primary, text)
    }

    pub fn tonal(text: impl Into<String>) -> Self {
        Button::new(ButtonKind::Tonal, text)
    }

    pub fn secondary(text: impl Into<String>) -> Self {
        Button::new(ButtonKind::Secondary, text)
    }

    pub fn ghost(text: impl Into<String>) -> Self {
        Button::new(ButtonKind::Ghost, text)
    }

    pub fn destructive(text: impl Into<String>) -> Self {
        Button::new(ButtonKind::Destructive, text)
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn small(mut self) -> Self {
        self.small = true;
        self
    }

    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }

    fn metrics(&self, ui: &Ui) -> ButtonMetrics {
        let (height, pad, size) = if self.small {
            (SMALL_HEIGHT, 10.0, 12.5)
        } else {
            (CONTROL_HEIGHT, 12.0, 13.5)
        };
        let galley = ui.painter().layout_no_wrap(
            self.text.clone(),
            FontId::new(size, self.kind.text_weight()),
            Color32::PLACEHOLDER,
        );
        let text_w = if self.text.is_empty() {
            0.0
        } else {
            galley.size().x
        };
        let icon_w = match (self.icon.is_some(), text_w > 0.0) {
            (true, true) => ICON + ICON_TEXT_GAP,
            (true, false) => ICON,
            (false, _) => 0.0,
        };
        let width = if self.text.is_empty() && self.icon.is_some() {
            height
        } else {
            (text_w + icon_w + 2.0 * pad).max(self.min_width)
        };
        ButtonMetrics {
            width,
            height,
            text_w,
            icon_w,
            galley,
        }
    }
}

impl egui::Widget for Button {
    fn ui(self, ui: &mut Ui) -> Response {
        let p = palette(ui);
        let enabled = ui.is_enabled();
        let ButtonMetrics {
            width,
            height,
            text_w,
            icon_w,
            galley,
        } = self.metrics(ui);
        let (rect, response) = ui.allocate_exact_size(
            vec2(width, height),
            if enabled {
                Sense::click()
            } else {
                Sense::hover()
            },
        );
        let text = self.text.clone();
        response.widget_info(|| WidgetInfo::labeled(Role::Button, enabled, &text));
        if !ui.is_rect_visible(rect) {
            return response;
        }
        let pressed = response.is_pointer_button_down_on() && enabled;
        let hovered = response.hovered() && enabled;
        let hover_t = theme::animate_effect(ui.ctx(), response.id.with("hover"), hovered);
        let press_t = theme::animate_spatial(ui.ctx(), response.id.with("press"), pressed);
        let (fill, stroke, color) = self.kind.look(p, hover_t, press_t);
        let outer_r = (REST_RADIUS + (PRESSED_RADIUS - REST_RADIUS) * press_t).round() as u8;
        let radius = self.corners.radius(outer_r);
        let painter = ui.painter();
        painter.rect(
            rect,
            radius,
            faded(fill, enabled),
            Stroke::new(stroke.width, faded(stroke.color, enabled)),
            StrokeKind::Inside,
        );
        let color = faded(color, enabled);
        let content_w = text_w + icon_w;
        let mut x = rect.center().x - content_w / 2.0;
        if let Some(icon) = self.icon {
            icons::paint(
                painter,
                Rect::from_center_size(pos2(x + ICON / 2.0, rect.center().y), vec2(ICON, ICON)),
                icon,
                color,
            );
            x += icon_w;
        }
        if text_w > 0.0 {
            painter.galley(
                pos2(x, rect.center().y - galley.size().y / 2.0),
                galley,
                color,
            );
        }
        if response.has_focus() {
            paint_focus_ring(painter, rect.expand(2.0), f32::from(outer_r) + 2.0, p);
        }
        response
    }
}

/// What [`split_button`] returns: the leading action's response, the trailing
/// chevron button's response, and whatever the menu closure returned while open.
pub struct SplitResponse<R> {
    pub primary: Response,
    pub menu: Response,
    pub menu_output: Option<R>,
}

/// A leading action button paired with a connected chevron menu trigger
/// (outer corners `8 px`, inner corners `2 px`, `2 px` gap). Sized up front
/// so it keeps left-to-right order inside a [`crate::Row`] (pinned by
/// `widgets::tests::a_split_button_runs_its_action_and_opens_its_menu`).
pub fn split_button<R>(
    ui: &mut Ui,
    mut button: Button,
    menu_name: &str,
    menu: impl FnOnce(&mut Ui) -> R,
) -> SplitResponse<R> {
    let kind = button.kind;
    let small = button.small;
    button.corners = Corners::Leading;
    let metrics = button.metrics(ui);
    let lead_w = metrics.width;
    let height = metrics.height;
    let trigger_w = height;
    let total_w = lead_w + SPLIT_GAP + trigger_w;
    let mut trigger = Button::new(kind, "").icon(Icon::ChevronDown);
    if small {
        trigger = trigger.small();
    }
    trigger.corners = Corners::Trailing;

    ui.allocate_ui_with_layout(
        vec2(total_w, height),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = SPLIT_GAP;
            let primary = ui.add(button);
            let menu_btn = ui.add(trigger);
            let enabled = ui.is_enabled();
            menu_btn.widget_info(|| WidgetInfo::labeled(Role::Button, enabled, menu_name));
            let menu_btn = menu_btn.on_hover_text(menu_name);
            let menu_output = egui::Popup::menu(&menu_btn)
                .show(menu)
                .map(|inner| inner.inner);
            SplitResponse {
                primary,
                menu: menu_btn,
                menu_output,
            }
        },
    )
    .inner
}

/// An icon-only button, square, named for a screen reader.
pub fn icon_button(ui: &mut Ui, kind: ButtonKind, icon: Icon, name: &str) -> Response {
    let response = ui.add(Button::new(kind, "").icon(icon));
    let enabled = ui.is_enabled();
    response.widget_info(|| WidgetInfo::labeled(Role::Button, enabled, name));
    response.on_hover_text(name)
}

const COPIED_FOR: f64 = 1.5;

/// A small secondary button that copies `text` to the clipboard and briefly
/// confirms with `"Copied"` before resetting to `label` (pinned by
/// `widgets::tests::a_copy_button_confirms_and_resets`).
pub fn copy_button(ui: &mut Ui, text: &str, label: &str) -> Response {
    let id = ui.scope_id().with(("vino-copy", label));
    let now = ui.input(|i| i.time);
    let copied_at = ui.data(|d| d.get_temp::<f64>(id));
    let recently_copied = copied_at.is_some_and(|at| now - at < COPIED_FOR);
    if let Some(at) = copied_at
        && now - at < COPIED_FOR
    {
        let left = (COPIED_FOR - (now - at)).max(0.05);
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs_f64(left));
    }
    let button = if recently_copied {
        Button::secondary("Copied").icon(Icon::Check).small()
    } else {
        Button::secondary(label).small()
    };
    let response = ui.add(button);
    if response.clicked() {
        ui.ctx().copy_text(text.to_owned());
        ui.data_mut(|d| d.insert_temp(id, now));
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs_f64(COPIED_FOR));
    }
    response
}
