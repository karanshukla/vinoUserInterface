//! The four kinds of button.

use egui::{
    Color32, FontFamily, FontId, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui, WidgetInfo,
    pos2, vec2,
};

use super::{CONTROL_HEIGHT, ICON, SMALL_HEIGHT, faded, paint_focus_ring, palette};
use crate::icons::{self, Icon};
use crate::theme::{self, Palette};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonKind {
    /// Ink on light, near-white on dark: one per view.
    Primary,
    /// Field-coloured with a rule.
    Secondary,
    /// Bare text until hovered.
    Ghost,
    /// Deep red with white text: only inside a confirmation.
    Destructive,
}

/// Between a button's icon and the text after it.
const ICON_TEXT_GAP: f32 = 6.0;

impl ButtonKind {
    /// The fill, the outline and the text colour, at rest, hovered or pressed.
    fn look(self, p: &Palette, pressed: bool, hovered: bool) -> (Color32, Stroke, Color32) {
        match self {
            ButtonKind::Primary => (
                if pressed {
                    p.primary_fill.lerp_to_gamma(p.bg, 0.22)
                } else if hovered {
                    p.primary_fill.lerp_to_gamma(p.bg, 0.12)
                } else {
                    p.primary_fill
                },
                Stroke::NONE,
                p.primary_text,
            ),
            ButtonKind::Secondary => (
                if pressed {
                    p.pressed
                } else if hovered {
                    p.hovered()
                } else {
                    p.field
                },
                Stroke::new(1.0_f32, p.rule_strong),
                p.text,
            ),
            ButtonKind::Ghost => (
                if pressed {
                    p.pressed
                } else if hovered {
                    p.field
                } else {
                    Color32::TRANSPARENT
                },
                Stroke::NONE,
                if pressed || hovered { p.text } else { p.muted },
            ),
            ButtonKind::Destructive => (
                if pressed {
                    p.danger.lerp_to_gamma(Color32::BLACK, 0.18)
                } else if hovered {
                    p.danger.lerp_to_gamma(Color32::BLACK, 0.1)
                } else {
                    p.danger
                },
                Stroke::NONE,
                p.danger_text,
            ),
        }
    }

    fn text_weight(self) -> FontFamily {
        match self {
            ButtonKind::Primary | ButtonKind::Destructive => theme::semibold(),
            ButtonKind::Secondary | ButtonKind::Ghost => theme::medium(),
        }
    }
}

/// A button in one of the four kinds, 30 tall (26 small), with an optional
/// icon before its text.
pub struct Button {
    kind: ButtonKind,
    text: String,
    icon: Option<Icon>,
    small: bool,
    min_width: f32,
}

impl Button {
    pub fn new(kind: ButtonKind, text: impl Into<String>) -> Self {
        Button {
            kind,
            text: text.into(),
            icon: None,
            small: false,
            min_width: 0.0,
        }
    }

    pub fn primary(text: impl Into<String>) -> Self {
        Button::new(ButtonKind::Primary, text)
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
}

impl egui::Widget for Button {
    fn ui(self, ui: &mut Ui) -> Response {
        let p = palette(ui);
        let enabled = ui.is_enabled();
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
        let width = (text_w + icon_w + 2.0 * pad).max(self.min_width);
        let width = if self.text.is_empty() && self.icon.is_some() {
            height
        } else {
            width
        };
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
        let pressed = response.is_pointer_button_down_on();
        let hovered = response.hovered() && enabled;
        let (fill, stroke, color) = self.kind.look(p, pressed, hovered);
        let painter = ui.painter();
        painter.rect(
            rect,
            8.0,
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
            paint_focus_ring(painter, rect.expand(2.0), 10.0, p);
        }
        response
    }
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
