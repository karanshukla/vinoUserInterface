use super::*;
use crate::Icon;
use egui::accesskit::Toggled;
use egui::{Role, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};

/// What kittest puts round the app on each side.
const KITTEST_MARGIN: f32 = 8.0;

/// A field too wide to share a row narrower than `NARROW` with its label.
const FITS_ONLY_A_NARROW_ROW: f32 = 300.0;

/// A harness `width` wide inside kittest's margin.
fn outer(width: f32) -> f32 {
    width + 2.0 * KITTEST_MARGIN
}

/// Installs the theme on the first frame, which then draws nothing else:
/// true when the caller should return.
fn styling_first_frame(ui: &Ui, styled: &mut bool) -> bool {
    if *styled {
        return false;
    }
    theme::apply(ui.ctx());
    *styled = true;
    ui.ctx().request_repaint();
    true
}

/// A row's width, padding to padding, from its label's and control's rects.
fn row_width(label: egui::Rect, control: egui::Rect) -> f32 {
    control.max.x - label.min.x + 2.0 * ROW_PADDING.x
}

#[derive(Clone, Copy, PartialEq)]
enum Device {
    Npu,
    Gpu,
    Cpu,
}

struct State {
    styled: bool,
    on: bool,
    minutes: u32,
    device: Device,
    images: bool,
    clicks: u32,
}

fn harness(width: f32) -> Harness<'static, State> {
    Harness::builder().with_size((width, 600.0)).build_ui_state(
        |ui, state: &mut State| {
            if styling_first_frame(ui, &mut state.styled) {
                return;
            }
            group_label(ui, "What to index");
            group(ui, |ui| {
                Row::new("Images")
                    .help("By what's in them.")
                    .show(ui, |ui, _| {
                        switch(ui, &mut state.on, "Index images");
                    });
                Row::new("Unload after")
                    .help("0 keeps it loaded.")
                    .show(ui, |ui, named| {
                        named(ui.add(egui::DragValue::new(&mut state.minutes)));
                    });
                Row::new("Device").show(ui, |ui, _| {
                    segmented(
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
                Row::new("Download")
                    .help("Checked against pinned sha256s.")
                    .show(ui, |ui, _| {
                        if ui.add(Button::secondary("Verify")).clicked() {
                            state.clicks += 1;
                        }
                        chip(ui, ChipKind::Ok, "Downloaded, 1.7 GB");
                    });
            });
            toggle_chip(ui, &mut state.images, "Images", Some(8));
            banner(
                ui,
                Tone::Warn,
                "The vision model isn't downloaded yet.",
                |ui| {
                    ui.add(Button::primary("Download").small());
                },
            );
            nav_item(
                ui,
                Icon::Cleanup,
                "Cleanup",
                "Cleanup (beta)",
                false,
                Some("Beta"),
            );
        },
        State {
            styled: false,
            on: false,
            minutes: 15,
            device: Device::Npu,
            images: true,
            clicks: 0,
        },
    )
}

/// A status is still: a busy chip asks for no frames of its own, so
/// `run` settles. It read as its text before and still does.
#[test]
#[allow(deprecated)]
fn a_busy_status_is_words_that_stand_still() {
    let mut h = Harness::builder().with_size((400.0, 100.0)).build_ui_state(
        |ui, styled: &mut bool| {
            if styling_first_frame(ui, styled) {
                return;
            }
            chip_busy(ui, ChipKind::Neutral, "Checking…");
        },
        false,
    );
    h.run();
    h.get_by_role_and_label(Role::Label, "Checking…");
}

#[test]
fn a_switch_is_a_named_switch_that_flips_from_the_keyboard() {
    let mut h = harness(820.0);
    h.run();
    let switch = h.get_by_role_and_label(Role::Switch, "Index images");
    assert_eq!(switch.accesskit_node().toggled(), Some(Toggled::False));
    switch.focus();
    h.run();
    h.key_press(egui::Key::Space);
    h.run();
    assert!(h.state().on);
    let switch = h.get_by_role_and_label(Role::Switch, "Index images");
    assert_eq!(switch.accesskit_node().toggled(), Some(Toggled::True));
    switch.click();
    h.run();
    assert!(!h.state().on);
}

#[test]
fn a_segmented_control_is_a_radio_group_that_arrows_move() {
    let mut h = harness(820.0);
    h.run();
    h.get_by_role_and_label(Role::RadioGroup, "Device");
    let gpu = h.get_by_role_and_label(Role::RadioButton, "GPU");
    assert_eq!(gpu.accesskit_node().toggled(), Some(Toggled::False));
    gpu.click();
    h.run();
    assert!(h.state().device == Device::Gpu);
    h.get_by_role_and_label(Role::RadioButton, "GPU").focus();
    h.run();
    h.key_press(egui::Key::ArrowRight);
    h.run();
    assert!(h.state().device == Device::Cpu);
    h.key_press(egui::Key::ArrowRight);
    h.run();
    assert!(
        h.state().device == Device::Npu,
        "a second press wraps round"
    );
    h.key_press(egui::Key::ArrowRight);
    h.run();
    assert!(
        h.state().device == Device::Gpu,
        "and a third moves on from there"
    );
    h.key_press(egui::Key::ArrowLeft);
    h.run();
    assert!(h.state().device == Device::Npu, "Left goes back");
}

#[test]
fn a_control_passed_through_named_takes_the_rows_title() {
    let mut h = harness(820.0);
    h.run();
    h.get_by_role_and_label(Role::SpinButton, "Unload after");
}

#[test]
fn buttons_chips_tags_and_banners_read_as_their_text() {
    let mut h = harness(820.0);
    h.run();
    h.get_by_role_and_label(Role::Button, "Verify").click();
    h.run();
    assert_eq!(h.state().clicks, 1);
    h.get_by_label("Downloaded, 1.7 GB");
    h.get_by_label("The vision model isn't downloaded yet.");
    h.get_by_role_and_label(Role::Button, "Download");
    h.get_by_role_and_label(Role::Heading, "What to index");
    let chip = h.get_by_role_and_label(Role::CheckBox, "Images (8)");
    assert_eq!(chip.accesskit_node().toggled(), Some(Toggled::True));
    chip.click();
    h.run();
    assert!(!h.state().images);
    let tab = h.get_by_role_and_label(Role::Tab, "Cleanup (beta)");
    assert_eq!(tab.accesskit_node().is_selected(), Some(false));
}

/// Stacked under its label, a row's controls stay in the order they
/// have beside it: the first added is the rightmost either way.
#[test]
fn a_stacked_row_keeps_the_order_it_has_beside_the_label() {
    for width in [820.0, 380.0] {
        let mut h = harness(width);
        h.run_steps(4);
        let verify = h.get_by_role_and_label(Role::Button, "Verify").rect();
        let chip = h.get_by_label("Downloaded, 1.7 GB").rect();
        assert!(
            chip.max.x <= verify.min.x,
            "{width}: the status sits left of the button it was added after"
        );
        let title = h.get_by_role_and_label(Role::Label, "Download").rect();
        if width < NARROW {
            assert!(verify.min.y >= title.max.y, "{width}: stacked");
            assert!(
                (chip.min.x - title.min.x).abs() < 1.0,
                "{width}: the leftmost control starts on the label's column"
            );
        }
    }
}

/// An icon-only button is a square the height of a button.
#[test]
fn an_icon_only_button_is_square() {
    let mut h = Harness::builder().with_size((200.0, 100.0)).build_ui_state(
        |ui, styled: &mut bool| {
            if styling_first_frame(ui, styled) {
                return;
            }
            icon_button(ui, ButtonKind::Secondary, Icon::Refresh, "Refresh");
            ui.add(Button::secondary("Refresh all").icon(Icon::Refresh));
        },
        false,
    );
    h.run();
    let square = h.get_by_role_and_label(Role::Button, "Refresh").rect();
    assert_eq!(square.size(), vec2(CONTROL_HEIGHT, CONTROL_HEIGHT));
}

/// `SIDEBAR_FROM` is the narrowest window where a sidebar, the page inset
/// and the page scroll leave a row `NARROW` wide: a control too wide to
/// share a narrower row with its label still sits beside it there.
#[test]
fn the_sidebar_breakpoint_leaves_a_row_exactly_narrow() {
    let mut h = Harness::builder()
        .with_size((outer(SIDEBAR_FROM), 400.0))
        .build_ui_state(
            |ui, (styled, text): &mut (bool, String)| {
                if styling_first_frame(ui, styled) {
                    return;
                }
                let p = palette(ui);
                egui::Panel::left("pages")
                    .resizable(false)
                    .show_separator_line(false)
                    .exact_size(SIDEBAR_WIDTH)
                    .frame(sidebar_frame(p))
                    .show(ui, |ui| {
                        nav_item(ui, Icon::Settings, "General", "General", true, None);
                    });
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE.inner_margin(egui::Margin {
                        left: PAGE_INSET as i8,
                        right: 0,
                        top: 0,
                        bottom: 0,
                    }))
                    .show(ui, |ui| {
                        page_scroll(ui, "General", CONTENT_MAX, |ui| {
                            group(ui, |ui| {
                                Row::new("Folder").show(ui, |ui, named| {
                                    named(ui.add(field(text, "", FITS_ONLY_A_NARROW_ROW)));
                                });
                            });
                        });
                    });
            },
            (false, String::new()),
        );
    h.run_steps(4);
    let label = h.get_by_role_and_label(Role::Label, "Folder").rect();
    let field = h.get_by_role_and_label(Role::TextInput, "Folder").rect();
    assert!(
        (field.center().y - label.center().y).abs() < 1.0,
        "the field sits beside its label at SIDEBAR_FROM"
    );
    assert_eq!(row_width(label, field), NARROW);
}

#[test]
fn rows_name_their_controls_at_both_widths() {
    for width in [820.0, 380.0] {
        let mut h = harness(width);
        h.run();
        let switch = h.get_by_role_and_label(Role::Switch, "Index images");
        assert!(
            switch.rect().max.x <= width + 0.5,
            "{width}: the switch is inside the window"
        );
        let verify = h.get_by_role_and_label(Role::Button, "Verify");
        let title = h.get_by_role_and_label(Role::Label, "Download");
        if width >= NARROW {
            assert!(
                (verify.rect().center().y - title.rect().center().y).abs() < 14.0,
                "{width}: the control sits beside its label"
            );
        } else {
            assert!(
                verify.rect().min.y >= title.rect().max.y,
                "{width}: the control sits under its label"
            );
        }
    }
}

/// The rule `a_narrow_row_keeps_a_control_that_leaves_the_label_its_minimum`
/// keeps, for the smallest control there is.
#[test]
fn a_switch_stays_beside_its_label_at_any_width() {
    for width in [820.0, 380.0] {
        let mut h = harness(width);
        h.run();
        let switch = h.get_by_role_and_label(Role::Switch, "Index images");
        let images = h.get_by_role_and_label(Role::Label, "Images");
        assert!(
            (switch.rect().center().y - images.rect().bottom()).abs() < 24.0
                && switch.rect().min.x > images.rect().max.x,
            "{width}: the switch sits beside its label"
        );
    }
}

/// The sidebar and the page inset grow with the window, but never so
/// fast that the page itself gets narrower as the window widens.
#[test]
fn the_sidebar_and_inset_grow_without_shrinking_the_page() {
    assert_eq!(sidebar_width(SIDEBAR_FROM), SIDEBAR_WIDTH);
    assert_eq!(page_inset(SIDEBAR_FROM), PAGE_INSET);
    assert_eq!(sidebar_width(FLUID_TO), SIDEBAR_MAX);
    assert_eq!(page_inset(FLUID_TO), PAGE_INSET_MAX);
    assert_eq!(sidebar_width(4000.0), SIDEBAR_MAX);
    let page = |w: f32| w - sidebar_width(w) - page_inset(w);
    let mut last = page(SIDEBAR_FROM);
    let mut w = SIDEBAR_FROM;
    while w < 2400.0 {
        w += 1.0;
        assert!(page(w) >= last, "{w}: the page doesn't shrink");
        last = page(w);
    }
}

fn two_groups(width: f32, zoom: f32) -> Harness<'static, (bool, String, String)> {
    Harness::builder()
        .with_size((outer(width), 600.0))
        .build_ui_state(
            move |ui, (styled, a, b): &mut (bool, String, String)| {
                if styling_first_frame(ui, styled) {
                    ui.ctx().set_zoom_factor(zoom);
                    return;
                }
                columns(ui, |ui, column| {
                    let (label, text) = match column {
                        Column::Left => ("Folder", &mut *a),
                        Column::Right => ("Cache", &mut *b),
                    };
                    group(ui, |ui| {
                        Row::new(label).show(ui, |ui, named| {
                            named(ui.add(field(text, "", FITS_ONLY_A_NARROW_ROW)));
                        });
                    });
                });
            },
            (false, String::new(), String::new()),
        )
}

/// From `TWO_COLUMNS_FROM` the columns sit side by side and each row is
/// still `NARROW` wide; below it, and at 2x text, they stack.
#[test]
fn columns_split_only_where_both_keep_wide_rows() {
    let mut h = two_groups(TWO_COLUMNS_FROM, 1.0);
    h.run_steps(4);
    let folder = h.get_by_role_and_label(Role::TextInput, "Folder").rect();
    let cache = h.get_by_role_and_label(Role::TextInput, "Cache").rect();
    let label = h.get_by_role_and_label(Role::Label, "Folder").rect();
    assert!(
        (folder.center().y - cache.center().y).abs() < 1.0,
        "side by side"
    );
    assert!(
        cache.min.x > folder.max.x + COLUMN_GAP,
        "the right column is right"
    );
    assert!(
        (folder.center().y - label.center().y).abs() < 1.0,
        "the field sits beside its label"
    );
    assert_eq!(row_width(label, folder), NARROW);

    for (width, zoom) in [(TWO_COLUMNS_FROM - 1.0, 1.0), (1.5 * TWO_COLUMNS_FROM, 2.0)] {
        let mut h = two_groups(width, zoom);
        h.run_steps(4);
        let folder = h.get_by_role_and_label(Role::TextInput, "Folder").rect();
        let cache = h.get_by_role_and_label(Role::TextInput, "Cache").rect();
        assert!(cache.min.y > folder.max.y, "{width} at {zoom}x: stacked");
        assert!(
            folder.max.x <= CONTENT_MAX + 8.0,
            "{width} at {zoom}x: a stacked column stops at CONTENT_MAX"
        );
    }
}

fn scrolled_field(width: f32, max_width: f32) -> Harness<'static, String> {
    Harness::builder()
        .with_size((outer(width), 300.0))
        .build_ui_state(
            move |ui, text: &mut String| {
                page_scroll(ui, "Results", max_width, |ui| {
                    let w = ui.available_width();
                    search_field(ui, text, "Search", "Search", w);
                });
            },
            String::new(),
        )
}

/// Only a `WIDE_MAX` page falls back to `CONTENT_MAX` when it is too
/// narrow to split; full-width content keeps the whole page.
#[test]
fn only_a_wide_max_page_is_capped_when_narrow() {
    let width = TWO_COLUMNS_FROM - 100.0;
    let field_w = |max_width| {
        let mut h = scrolled_field(width, max_width);
        h.run_steps(2);
        h.get_by_role(Role::TextInput).rect().width()
    };
    assert!(field_w(WIDE_MAX) <= CONTENT_MAX);
    assert!(field_w(f32::INFINITY) > CONTENT_MAX + 100.0);
}

/// A focused search field's ring is 2 px where the resting one is 1, and the
/// text inside doesn't move for it.
#[test]
fn a_search_field_stays_put_when_its_ring_thickens() {
    let mut h = Harness::builder().with_size((400.0, 100.0)).build_ui_state(
        |ui, (styled, text): &mut (bool, String)| {
            if styling_first_frame(ui, styled) {
                return;
            }
            search_field(ui, text, "Search", "Search", 300.0);
        },
        (false, String::new()),
    );
    h.run();
    let resting = h.get_by_role(Role::TextInput).rect();
    h.get_by_role(Role::TextInput).focus();
    h.run();
    let focused = h.get_by_role(Role::TextInput);
    assert!(focused.is_focused());
    assert_eq!(focused.rect(), resting);
}

#[test]
fn a_search_field_clears_from_its_inline_button_and_escape() {
    let mut h = Harness::builder().with_size((400.0, 100.0)).build_ui_state(
        |ui, (styled, text): &mut (bool, String)| {
            if styling_first_frame(ui, styled) {
                return;
            }
            search_field(ui, text, "Search", "Search", 300.0);
        },
        (false, "invoice".to_owned()),
    );
    h.run();
    h.get_by_role_and_label(Role::Button, "Clear search")
        .click();
    h.run();
    assert!(h.state().1.is_empty());
    assert!(
        h.query_by_role_and_label(Role::Button, "Clear search")
            .is_none()
    );

    h.state_mut().1 = "receipt".to_owned();
    h.run();
    h.get_by_role(Role::TextInput).focus();
    h.run();
    h.key_press(egui::Key::Escape);
    h.run();
    assert!(h.state().1.is_empty());
}
