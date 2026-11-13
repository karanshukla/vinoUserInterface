//! Input as the desktop means it, where egui's reading of it falls short.

use egui::{Event, MouseWheelUnit, RawInput};

/// How many times the distance a touchpad reports a page scrolls by.
pub const TOUCHPAD_SCROLL: f32 = 3.0;

/// Touchpad scrolling at a usable pace. egui moves a page by exactly the
/// distance the compositor reports for a swipe (measured on Plasma at 150%:
/// about 6 points an event, 1,000 for several full swipes, a window and a
/// half), a fraction of what the desktop's own apps move for the same swipe.
/// A wheel reports lines, and is left alone.
///
/// Call it from `eframe::App::raw_input_hook`:
///
/// ```ignore
/// fn raw_input_hook(&mut self, _ctx: &egui::Context, raw: &mut egui::RawInput) {
///     vino_ui::input::faster_touchpad(raw);
/// }
/// ```
pub fn faster_touchpad(raw: &mut RawInput) {
    for event in &mut raw.events {
        if let Event::MouseWheel {
            unit: MouseWheelUnit::Point,
            delta,
            ..
        } = event
        {
            *delta *= TOUCHPAD_SCROLL;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Modifiers, MouseWheelSource, TouchPhase, vec2};

    fn wheel(unit: MouseWheelUnit) -> Event {
        Event::MouseWheel {
            unit,
            delta: vec2(1.0, -6.0),
            phase: TouchPhase::Move,
            source: MouseWheelSource::Unknown,
            modifiers: Modifiers::NONE,
        }
    }

    fn delta(event: &Event) -> egui::Vec2 {
        match event {
            Event::MouseWheel { delta, .. } => *delta,
            _ => unreachable!(),
        }
    }

    #[test]
    fn a_touchpad_goes_further_and_a_wheel_as_far_as_it_did() {
        let mut raw = RawInput {
            events: vec![wheel(MouseWheelUnit::Point), wheel(MouseWheelUnit::Line)],
            ..Default::default()
        };
        faster_touchpad(&mut raw);
        assert_eq!(delta(&raw.events[0]), vec2(3.0, -18.0));
        assert_eq!(delta(&raw.events[1]), vec2(1.0, -6.0));
    }
}
