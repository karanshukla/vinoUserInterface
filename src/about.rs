//! The About page every app's window ends with: which app and version this
//! is, a link to vinopilot.ca, and the other vinoPilot apps, each with a
//! link to its part of the site. One copy, so the family reads the same
//! from every window and a new app shows up in all of them at once.

use egui::Ui;

use crate::a11y;
use crate::family::{self, App};
use crate::widgets::{self, Button, Row};

/// The page's title, for the app's page list.
pub const TITLE: &str = "About";

/// The About page's body, under the page title the app draws. `version` is
/// the app's own, usually `env!("CARGO_PKG_VERSION")`.
pub fn page(ui: &mut Ui, this: App, version: &str) {
    let heading = format!("{} {version}", this.name());
    widgets::group(ui, |ui| {
        Row::new(&heading).help(this.blurb()).show(ui, |ui, _| {
            link(ui, family::SITE_LABEL, family::SITE);
        });
    });
    widgets::group_label(ui, "Other vinoPilot apps");
    widgets::group(ui, |ui| {
        for app in this.others() {
            Row::new(app.name()).help(app.blurb()).show(ui, |ui, _| {
                let learn = link(ui, "Learn more", app.page());
                a11y::for_what(learn, app.name());
            });
        }
    });
}

/// A small secondary button that opens `url` in the browser.
fn link(ui: &mut Ui, text: &str, url: &str) -> egui::Response {
    let response = ui.add(Button::secondary(text).small()).on_hover_text(url);
    if response.clicked()
        && let Err(err) = family::open(url)
    {
        eprintln!("[vino-ui] couldn't open {url}: {err}");
    }
    response
}
