//! Every control the crate draws, audited the way a screen reader and a
//! keyboard meet it: through the AccessKit tree egui hands the platform.

use egui::accesskit::{Action, Role};
use egui::{Theme, Ui};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use vino_user_interface::widgets::{self, NARROW};
use vino_user_interface::{Button, ButtonKind, ChipKind, Icon, Row, Tone, a11y, theme};

#[derive(Clone, Copy, PartialEq)]
enum Device {
    Npu,
    Gpu,
    Cpu,
}

struct State {
    styled: bool,
    theme: Theme,
    on: bool,
    minutes: u32,
    device: Device,
    images: bool,
    folder: String,
    query: String,
    model: &'static str,
    page: usize,
}

fn gallery(ui: &mut Ui, state: &mut State) {
    if !state.styled {
        theme::apply(ui.ctx());
        ui.ctx().set_theme(state.theme);
        state.styled = true;
        ui.ctx().request_repaint();
        return;
    }
    let header = theme::header(ui.ctx(), None, true);
    ui.horizontal(|ui| {
        a11y::tab_list(ui, "Pages");
        for (i, name) in ["General", "Models"].into_iter().enumerate() {
            if widgets::tab(ui, name, state.page == i, true, header).clicked() {
                state.page = i;
            }
        }
    });
    widgets::page_scroll(ui, "Settings", widgets::CONTENT_MAX, |ui| {
        let title = widgets::title(ui, "Settings");
        a11y::heading(&title, "Settings", 1);
        widgets::search_field(ui, &mut state.query, "Search files", "Search", 300.0);
        widgets::group_label(ui, "What to index");
        widgets::group(ui, |ui| {
            Row::new("Images")
                .help("By what's in them.")
                .show(ui, |ui, _| {
                    widgets::switch(ui, &mut state.on, "Index images");
                });
            Row::new("Unload after").show(ui, |ui, named| {
                named(ui.add(egui::DragValue::new(&mut state.minutes)));
            });
            Row::new("Device").show(ui, |ui, _| {
                widgets::segmented(
                    ui,
                    "Device",
                    &mut state.device,
                    &[
                        (Device::Npu, "NPU"),
                        (Device::Gpu, "GPU"),
                        (Device::Cpu, "CPU"),
                    ],
                );
            });
            Row::new("Folder").show(ui, |ui, named| {
                named(ui.add(widgets::field(&mut state.folder, "~/Pictures", 180.0)));
            });
            Row::new("Model").show(ui, |ui, named| {
                let picked =
                    widgets::combo(ui, "model", state.model.to_owned(), 160.0).show_ui(ui, |ui| {
                        for model in ["Small", "Large"] {
                            ui.selectable_value(&mut state.model, model, model);
                        }
                    });
                named(picked.response);
            });
            Row::new("Download").show(ui, |ui, _| {
                ui.add(Button::secondary("Verify"));
                widgets::chip(ui, ChipKind::Ok, "Downloaded, 1.7 GB");
            });
        });
        ui.horizontal_wrapped(|ui| {
            widgets::toggle_chip(ui, &mut state.images, "Images", Some(8));
            ui.add(Button::primary("Save"));
            ui.add(Button::ghost("Cancel").small());
            ui.add(Button::destructive("Delete index").icon(Icon::Cleanup));
            let remove = widgets::icon_button(ui, ButtonKind::Ghost, Icon::Excluded, "Remove");
            a11y::for_what(remove, "~/Downloads");
        });
        widgets::banner(
            ui,
            Tone::Warn,
            "The vision model isn't downloaded yet.",
            |ui| {
                ui.add(Button::primary("Download").small());
            },
        );
        widgets::notice(ui, Tone::Error, "Indexing stopped.");
        widgets::nav_item(
            ui,
            Icon::Cleanup,
            "Cleanup",
            "Cleanup (beta)",
            false,
            Some("Beta"),
        );
    });
}

fn harness(theme: Theme, width: f32) -> Harness<'static, State> {
    Harness::builder().with_size((width, 900.0)).build_ui_state(
        gallery,
        State {
            styled: false,
            theme,
            on: false,
            minutes: 15,
            device: Device::Npu,
            images: true,
            folder: String::new(),
            query: String::new(),
            model: "Small",
            page: 0,
        },
    )
}

/// Each theme at a wide and a narrow width, where rows stack.
fn each_case(check: impl Fn(&mut Harness<'static, State>, &str)) {
    for theme in [Theme::Dark, Theme::Light] {
        for width in [820.0, NARROW - 60.0] {
            let mut h = harness(theme, width);
            h.run();
            check(&mut h, &format!("{theme:?} at {width}"));
        }
    }
}

/// What a focusable node is, in a form a failure can print. The gallery
/// names no two controls of a role alike, so role and name pick one out.
struct Stop {
    role: Role,
    name: String,
    rect: egui::Rect,
}

fn focus_stops(h: &Harness<'static, State>) -> Vec<Stop> {
    h.root()
        .children_recursive()
        .filter(|n| {
            let node = n.accesskit_node();
            node.data().supports_action(Action::Focus) && !node.is_hidden()
        })
        .map(|n| {
            let node = n.accesskit_node();
            Stop {
                role: node.role(),
                name: node.label().unwrap_or_default(),
                rect: n.rect(),
            }
        })
        .collect()
}

/// WCAG 4.1.2: a control says what it is and what it is called.
#[test]
fn every_control_has_a_role_and_a_name() {
    each_case(|h, case| {
        let stops = focus_stops(h);
        assert!(stops.len() >= 15, "{case}: the gallery lost its controls");
        for stop in stops {
            assert!(
                !matches!(stop.role, Role::Unknown | Role::GenericContainer),
                "{case}: {:?} {:?} has no role",
                stop.name,
                stop.role
            );
            assert!(
                !stop.name.trim().is_empty(),
                "{case}: a {:?} has no name",
                stop.role
            );
        }
    });
}

/// WCAG 2.5.8: a target is at least 24 by 24, or a 24 px circle on its
/// centre touches no other target.
#[test]
fn every_control_is_big_enough_to_hit() {
    each_case(|h, case| {
        let stops = focus_stops(h);
        let mut small = Vec::new();
        for stop in &stops {
            // The scroll bar is 10 wide by design and has the wheel and keys besides.
            if stop.role == Role::ScrollBar || stop.rect.width().min(stop.rect.height()) >= 24.0 {
                continue;
            }
            let centre = stop.rect.center();
            let crowded = stops
                .iter()
                .filter(|other| !std::ptr::eq(*other, stop))
                .find(|other| other.rect.distance_to_pos(centre) < 12.0);
            if let Some(other) = crowded {
                small.push(format!(
                    "{:?} {:?} is {} x {} and {:?} {:?} is within 12 px of its centre",
                    stop.role,
                    stop.name,
                    stop.rect.width(),
                    stop.rect.height(),
                    other.role,
                    other.name
                ));
            }
        }
        assert!(small.is_empty(), "{case}:\n{}", small.join("\n"));
    });
}

/// WCAG 2.1.1 and 2.1.2: Tab reaches every control, and gets out of each one.
#[test]
fn tab_reaches_every_control() {
    each_case(|h, case| {
        let stops = focus_stops(h);
        let mut reached = Vec::new();
        for _ in 0..stops.len() * 2 {
            h.key_press(egui::Key::Tab);
            h.run();
            let focused = h
                .root()
                .children_recursive()
                .find(|n| n.accesskit_node().is_focused())
                .map(|n| {
                    let node = n.accesskit_node();
                    (node.role(), node.label().unwrap_or_default())
                });
            if let Some(focused) = focused
                && !reached.contains(&focused)
            {
                reached.push(focused);
            }
        }
        for stop in &stops {
            assert!(
                reached.contains(&(stop.role, stop.name.clone())),
                "{case}: Tab never reaches {:?} {:?}",
                stop.role,
                stop.name
            );
        }
    });
}

/// Names a screen reader lists by: headings, tabs and the live notice.
#[test]
fn structure_is_announced() {
    each_case(|h, case| {
        h.get_by_role_and_label(Role::Heading, "Settings");
        h.get_by_role_and_label(Role::Heading, "What to index");
        let tabs = h.get_by_role_and_label(Role::TabList, "Pages");
        assert_eq!(tabs.children().count(), 2, "{case}: two tabs in the list");
        let general = h.get_by_role_and_label(Role::Tab, "General");
        assert_eq!(general.accesskit_node().is_selected(), Some(true), "{case}");
        h.get_by_role_and_label(Role::Button, "Remove ~/Downloads");
        let notice = h.get_by_label("Indexing stopped.");
        assert_eq!(
            notice.accesskit_node().data().live(),
            Some(egui::accesskit::Live::Assertive),
            "{case}: the notice is read out at once"
        );
    });
}
