//! The app's own title bar, driven the way a pointer meets it: each button
//! asks the desktop for what it says, the bar drags and maximises the
//! window, the edges resize it, and none of that takes clicks from the page.

#[path = "../examples/gallery.rs"]
mod gallery;

use egui::{Pos2, ResizeDirection, ViewportCommand, ViewportId, pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use gallery::{Gallery, Page};
use vino_user_interface::{egui, theme, window};

struct State {
    gallery: Gallery,
    styled: bool,
}

fn harness(width: f32, height: f32, maximised: bool) -> Harness<'static, State> {
    let mut gallery = Gallery::on(Page::General);
    gallery.app_title_bar = true;
    let mut h = Harness::builder()
        .with_size((width, height))
        // A frame a sixtieth apart, as a real one is: at the default 0.25 s two
        // clicks are never a double click.
        .with_step_dt(1.0 / 60.0)
        .build_ui_state(
            |ui, state: &mut State| {
                if !state.styled {
                    theme::apply(ui.ctx());
                    ui.ctx().all_styles_mut(|style| style.animation_time = 0.0);
                    state.styled = true;
                    ui.ctx().request_repaint();
                    return;
                }
                state.gallery.ui(ui);
                window::edges(ui);
            },
            State {
                gallery,
                styled: false,
            },
        );
    if maximised {
        h.input_mut()
            .viewports
            .entry(ViewportId::ROOT)
            .or_default()
            .maximized = Some(true);
    }
    h.run_steps(3);
    h
}

fn commands(h: &Harness<'_, State>) -> Vec<ViewportCommand> {
    h.output()
        .viewport_output
        .get(&ViewportId::ROOT)
        .map(|v| v.commands.clone())
        .unwrap_or_default()
}

fn button(pos: Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    }
}

/// One frame per event, and every viewport command the frames sent.
fn frames(h: &mut Harness<'_, State>, events: Vec<egui::Event>) -> Vec<ViewportCommand> {
    let mut sent = Vec::new();
    for event in events {
        h.event(event);
        h.step();
        sent.extend(commands(h));
    }
    sent
}

fn click(h: &mut Harness<'_, State>, at: Pos2) -> Vec<ViewportCommand> {
    frames(
        h,
        vec![
            egui::Event::PointerMoved(at),
            button(at, true),
            button(at, false),
        ],
    )
}

fn drag(h: &mut Harness<'_, State>, from: Pos2) -> Vec<ViewportCommand> {
    let to = from + vec2(30.0, 20.0);
    frames(
        h,
        vec![
            egui::Event::PointerMoved(from),
            button(from, true),
            egui::Event::PointerMoved(from + vec2(10.0, 5.0)),
            egui::Event::PointerMoved(to),
            button(to, false),
        ],
    )
}

fn centre(h: &Harness<'_, State>, label: &str) -> Pos2 {
    h.get_by_label(label).rect().center()
}

#[test]
fn each_button_asks_for_what_it_says() {
    let mut h = harness(960.0, 640.0, false);
    let close = centre(&h, "Close");
    assert!(click(&mut h, close).contains(&ViewportCommand::Close));
    let mut h = harness(960.0, 640.0, false);
    let minimise = centre(&h, "Minimise");
    assert!(click(&mut h, minimise).contains(&ViewportCommand::Minimized(true)));
    let maximise = centre(&h, "Maximise");
    assert!(click(&mut h, maximise).contains(&ViewportCommand::Maximized(true)));
}

#[test]
fn a_maximised_window_offers_restore_and_has_no_edges() {
    let mut h = harness(960.0, 640.0, true);
    assert!(h.query_by_label("Maximise").is_none());
    let restore = centre(&h, "Restore");
    assert!(click(&mut h, restore).contains(&ViewportCommand::Maximized(false)));
    let sent = drag(&mut h, pos2(958.0, 320.0));
    assert!(
        !sent
            .iter()
            .any(|c| matches!(c, ViewportCommand::BeginResize(_))),
        "{sent:?}"
    );
}

#[test]
fn the_bar_moves_and_maximises_the_window() {
    let mut h = harness(960.0, 640.0, false);
    // Empty bar, left of the buttons.
    let bar = centre(&h, "Minimise") - vec2(200.0, 0.0);
    let sent = drag(&mut h, bar);
    assert!(sent.contains(&ViewportCommand::StartDrag), "{sent:?}");
    let mut h = harness(960.0, 640.0, false);
    let sent = frames(
        &mut h,
        vec![
            egui::Event::PointerMoved(bar),
            button(bar, true),
            button(bar, false),
            button(bar, true),
            button(bar, false),
        ],
    );
    assert!(sent.contains(&ViewportCommand::Maximized(true)), "{sent:?}");
    // The sidebar's brand is a handle too.
    let sent = drag(&mut h, pos2(100.0, 20.0));
    assert!(sent.contains(&ViewportCommand::StartDrag), "{sent:?}");
}

#[test]
fn the_edges_resize_the_window() {
    let mut h = harness(960.0, 640.0, false);
    for (at, direction) in [
        (pos2(958.0, 320.0), ResizeDirection::East),
        (pos2(2.0, 320.0), ResizeDirection::West),
        (pos2(480.0, 638.0), ResizeDirection::South),
        (pos2(480.0, 1.0), ResizeDirection::North),
        (pos2(957.0, 637.0), ResizeDirection::SouthEast),
        (pos2(3.0, 3.0), ResizeDirection::NorthWest),
    ] {
        let sent = drag(&mut h, at);
        assert!(
            sent.contains(&ViewportCommand::BeginResize(direction)),
            "{direction:?} at {at:?}: {sent:?}"
        );
    }
}

#[test]
fn the_page_keeps_its_clicks() {
    let mut h = harness(960.0, 640.0, false);
    let models = centre(&h, "Models");
    let sent = click(&mut h, models);
    assert!(sent.is_empty(), "{sent:?}");
    h.run_steps(2);
    assert_eq!(h.state().gallery.page, Page::Models);
}

#[test]
fn a_narrow_window_has_the_buttons_in_its_header() {
    let mut h = harness(420.0, 560.0, false);
    let close = centre(&h, "Close");
    assert!(close.y < 40.0, "{close:?}");
    let sent = drag(&mut h, close - vec2(150.0, 0.0));
    assert!(sent.contains(&ViewportCommand::StartDrag), "{sent:?}");
    let shortcuts = centre(&h, "Shortcuts");
    let sent = click(&mut h, shortcuts);
    assert!(sent.is_empty(), "{sent:?}");
    h.run_steps(2);
    assert_eq!(h.state().gallery.page, Page::Shortcuts);
}
