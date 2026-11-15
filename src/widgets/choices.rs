//! Controls that hold a choice or a value: switch, segmented control, filter
//! chip, combo box and fields.

use egui::{
    Align, Color32, FontId, Layout, Rect, Response, Role, Sense, Stroke, StrokeKind, Ui,
    WidgetInfo, pos2, vec2,
};

use super::{CONTROL_HEIGHT, faded, paint_focus_ring, palette, weight};
use crate::icons::{self, Icon};
use crate::theme;

/// A switch: 36 × 20, green when on. `name` is what a screen reader calls it,
/// usually the row's title; nothing is painted beside it.
pub fn switch(ui: &mut Ui, on: &mut bool, name: &str) -> Response {
    let p = palette(ui);
    let enabled = ui.is_enabled();
    let (rect, mut response) = ui.allocate_exact_size(vec2(36.0, 20.0), Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    let is_on = *on;
    response.widget_info(|| WidgetInfo::selected(Role::Switch, enabled, is_on, name));
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let t = ui.ctx().animate_bool_responsive(response.id, is_on);
    let track = p.pressed.lerp_to_gamma(p.on, t);
    let edge = p.edge.lerp_to_gamma(p.on, t);
    let knob = p.edge.lerp_to_gamma(p.switch_knob, t);
    let painter = ui.painter();
    painter.rect(
        rect,
        10.0,
        faded(track, enabled),
        Stroke::new(1.0_f32, faded(edge, enabled)),
        StrokeKind::Inside,
    );
    let x = rect.left() + 10.0 + 16.0 * t;
    painter.circle_filled(pos2(x, rect.center().y), 7.0, faded(knob, enabled));
    if response.has_focus() {
        paint_focus_ring(painter, rect.expand(3.0), 13.0, p);
    }
    response
}

/// Two to four short options in one control; a radio group for screen
/// readers, with Left and Right moving the choice. True when it changed.
pub fn segmented<T: Copy + PartialEq>(
    ui: &mut Ui,
    name: &str,
    current: &mut T,
    options: &[(T, &str)],
) -> bool {
    let p = palette(ui);
    let enabled = ui.is_enabled();
    let before = *current;
    let galleys: Vec<_> = options
        .iter()
        .map(|(value, label)| {
            let selected = *current == *value;
            ui.painter().layout_no_wrap(
                (*label).to_owned(),
                FontId::new(13.0, weight(selected)),
                Color32::PLACEHOLDER,
            )
        })
        .collect();
    // Sized up front: a row lays its controls out from the right.
    let total = galleys.iter().map(|g| g.size().x + 20.0).sum::<f32>()
        + 2.0 * options.len().saturating_sub(1) as f32;
    let recessed_track = egui::Frame::new()
        .fill(faded(p.pressed, enabled))
        .corner_radius(8.0)
        .inner_margin(2.0);
    recessed_track.show(ui, |ui| {
        ui.ctx().accesskit_node_builder(ui.unique_id(), |node| {
            node.set_role(Role::RadioGroup);
            node.set_label(name);
        });
        ui.spacing_mut().item_spacing.x = 2.0;
        ui.allocate_ui_with_layout(
            vec2(total, 24.0),
            Layout::left_to_right(Align::Center),
            |ui| {
                let mut ids = Vec::with_capacity(options.len());
                let mut move_focus = None;
                for (i, ((value, label), galley)) in options.iter().zip(galleys).enumerate() {
                    let selected = *current == *value;
                    let size = vec2(galley.size().x + 20.0, 24.0);
                    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
                    ids.push(response.id);
                    response.widget_info(|| {
                        WidgetInfo::selected(Role::RadioButton, enabled, selected, *label)
                    });
                    if response.clicked() {
                        *current = *value;
                    }
                    if response.has_focus() {
                        // The arrows are ours: egui would move focus by them
                        // too, and it can't wrap round.
                        ui.memory_mut(|m| {
                            m.set_focus_lock_filter(
                                response.id,
                                egui::EventFilter {
                                    horizontal_arrows: true,
                                    vertical_arrows: true,
                                    ..Default::default()
                                },
                            )
                        });
                        let step = arrow_step(ui);
                        if step != 0 {
                            let to = (i as i32 + step).rem_euclid(options.len() as i32) as usize;
                            *current = options[to].0;
                            move_focus = Some(to);
                        }
                    }
                    let painter = ui.painter();
                    if selected {
                        painter.rect(
                            rect,
                            6.0,
                            faded(p.surface, enabled),
                            Stroke::new(1.0_f32, faded(p.rule_strong, enabled)),
                            StrokeKind::Inside,
                        );
                    } else if response.hovered() && enabled {
                        painter.rect_filled(rect, 6.0, p.field);
                    }
                    let color = if selected { p.text } else { p.muted };
                    painter.galley(
                        rect.center() - galley.size() / 2.0,
                        galley,
                        faded(color, enabled),
                    );
                    if response.has_focus() {
                        paint_focus_ring(painter, rect.expand(2.0), 8.0, p);
                    }
                }
                // Focus follows the choice, as in any radio group.
                if let Some(to) = move_focus {
                    ui.memory_mut(|m| m.request_focus(ids[to]));
                }
            },
        );
    });
    *current != before
}

/// Which way the arrow keys move a choice this frame: 1 on, -1 back, 0 not
/// at all.
fn arrow_step(ui: &Ui) -> i32 {
    ui.input(|input| {
        if input.key_pressed(egui::Key::ArrowRight) || input.key_pressed(egui::Key::ArrowDown) {
            1
        } else if input.key_pressed(egui::Key::ArrowLeft) || input.key_pressed(egui::Key::ArrowUp) {
            -1
        } else {
            0
        }
    })
}

/// A filter chip: a pill that is on or off, with an optional count.
pub fn toggle_chip(ui: &mut Ui, on: &mut bool, label: &str, count: Option<usize>) -> Response {
    let p = palette(ui);
    let enabled = ui.is_enabled();
    let font = FontId::new(13.0, weight(*on));
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font, Color32::PLACEHOLDER);
    let count_galley = count.map(|n| {
        ui.painter().layout_no_wrap(
            n.to_string(),
            FontId::new(13.0, theme::semibold()),
            Color32::PLACEHOLDER,
        )
    });
    let count_w = count_galley.as_ref().map_or(0.0, |g| g.size().x + 6.0);
    let size = vec2(galley.size().x + count_w + 24.0, 28.0);
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    let is_on = *on;
    let name = match count {
        Some(n) => format!("{label} ({n})"),
        None => label.to_owned(),
    };
    response.widget_info(|| WidgetInfo::selected(Role::CheckBox, enabled, is_on, &name));
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let painter = ui.painter();
    let (fill, stroke, color, count_color) = if is_on {
        (p.tint, Stroke::NONE, p.accent, p.accent)
    } else if response.hovered() && enabled {
        (
            p.hovered(),
            Stroke::new(1.0_f32, p.rule_strong),
            p.text,
            p.muted,
        )
    } else {
        (
            Color32::TRANSPARENT,
            Stroke::new(1.0_f32, p.rule_strong),
            p.text,
            p.muted,
        )
    };
    painter.rect(rect, 14.0, faded(fill, enabled), stroke, StrokeKind::Inside);
    let mut x = rect.left() + 12.0;
    let y = rect.center().y - galley.size().y / 2.0;
    painter.galley(pos2(x, y), galley.clone(), faded(color, enabled));
    x += galley.size().x + 6.0;
    if let Some(count) = count_galley {
        painter.galley(pos2(x, y), count, faded(count_color, enabled));
    }
    if response.has_focus() {
        paint_focus_ring(painter, rect.expand(2.0), 16.0, p);
    }
    response
}

/// The chevron a combo box shows, in the muted colour.
pub fn combo_chevron(ui: &Ui, rect: Rect, _visuals: &egui::style::WidgetVisuals, _open: bool) {
    let p = palette(ui);
    icons::paint(
        ui.painter(),
        Rect::from_center_size(rect.center(), vec2(14.0, 14.0)),
        Icon::ChevronDown,
        p.muted,
    );
}

/// A combo box at the row's height, with the chevron and a width that never
/// passes the edge.
pub fn combo(ui: &Ui, id_salt: &str, selected: String, width: f32) -> egui::ComboBox<'static> {
    egui::ComboBox::from_id_salt(id_salt)
        .width(width.min(ui.available_width()))
        .height(260.0)
        .selected_text(selected)
        .icon(combo_chevron)
        .truncate()
}

/// A text field at the row's height.
pub fn field<'t>(text: &'t mut String, hint: &str, width: f32) -> egui::TextEdit<'t> {
    egui::TextEdit::singleline(text)
        .hint_text(hint)
        .desired_width(width)
        .min_size(vec2(0.0, CONTROL_HEIGHT))
        .margin(egui::Margin::symmetric(10, 5))
        .align(egui::Align2::LEFT_CENTER)
}

/// A larger search field with a search icon and an accent ring while it has
/// focus. `name` is its accessible name.
pub fn search_field(
    ui: &mut Ui,
    text: &mut String,
    hint: &str,
    name: &str,
    width: f32,
) -> Response {
    let p = palette(ui);
    let id = ui.scope_id().with(("search-field", name));
    let focused = ui.memory(|m| m.has_focus(id));
    let stroke = if focused {
        Stroke::new(2.0_f32, p.accent)
    } else {
        Stroke::new(1.0_f32, p.edge)
    };
    // A Frame adds its stroke outside the margin, so the stroke's width comes
    // off the margin and the height: the field stays `width` x 38 and its
    // text where it was when the ring thickens on focus.
    let ring = stroke.width as i8;
    let frame = egui::Frame::new()
        .fill(p.field)
        .stroke(stroke)
        .corner_radius(10.0)
        .inner_margin(egui::Margin::symmetric(12 - ring, 0));
    frame
        .show(ui, |ui| {
            ui.set_width(width - 24.0);
            ui.set_height(38.0 - 2.0 * stroke.width);
            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                icons::icon(ui, Icon::Search, 16.0, p.muted);
                let edit = ui.add(
                    egui::TextEdit::singleline(text)
                        .id(id)
                        .frame(egui::Frame::NONE)
                        .font(FontId::proportional(15.0))
                        .hint_text(hint)
                        .desired_width(f32::INFINITY)
                        .margin(egui::Margin::symmetric(0, 8))
                        .align(egui::Align2::LEFT_CENTER),
                );
                ui.ctx()
                    .accesskit_node_builder(edit.id, |node| node.set_label(name));
                edit
            })
            .inner
        })
        .inner
}
