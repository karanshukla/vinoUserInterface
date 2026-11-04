//! The About page every app's window ends with: which app and version this
//! is, a link to vinopilot.ca, and the other vinoPilot apps, each with a
//! link to its part of the site. One copy, so the family reads the same
//! from every window and a new app shows up in all of them at once. With
//! [`page_with_updates`], also a row that checks for a newer release and
//! installs it with the app's own command.

use egui::Ui;

use crate::a11y;
use crate::family::{self, App};
use crate::update::{Status, Updater};
use crate::widgets::{self, Button, ChipKind, Row, Tone};

/// The page's title, for the app's page list.
pub const TITLE: &str = "About";

/// The About page's body, under the page title the app draws. `version` is
/// the app's own, usually `env!("CARGO_PKG_VERSION")`.
pub fn page(ui: &mut Ui, this: App, version: &str) {
    body(ui, this, version, None);
}

/// [`page`], plus an Updates row under the version: "Check for updates",
/// then "Update to vN" when a newer release is out. Nothing goes online
/// until it's clicked. Keep the `Updater` for the window's life; this polls
/// it and keeps repainting while it works.
pub fn page_with_updates(ui: &mut Ui, updater: &mut Updater) {
    let (this, version) = (updater.app(), updater.current().to_owned());
    body(ui, this, &version, Some(updater));
}

fn body(ui: &mut Ui, this: App, version: &str, updater: Option<&mut Updater>) {
    let heading = format!("{} {version}", this.name());
    widgets::group(ui, |ui| {
        Row::new(&heading).help(this.blurb()).show(ui, |ui, _| {
            link(ui, family::SITE_LABEL, family::SITE);
        });
        if let Some(updater) = updater {
            update_row(ui, updater);
        }
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

fn update_row(ui: &mut Ui, updater: &mut Updater) {
    updater.poll();
    if updater.busy() {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(200));
    }
    let name = updater.app().name();
    let status = updater.status().clone();
    let (help, tone) = match &status {
        Status::Idle => (
            "Checks GitHub for a newer release. Nothing else is sent.".to_owned(),
            Tone::Neutral,
        ),
        Status::Checking => (
            "Checking GitHub for a newer release.".to_owned(),
            Tone::Neutral,
        ),
        Status::UpToDate => (
            format!("{name} {} is the newest release.", updater.current()),
            Tone::Neutral,
        ),
        Status::Development(tag) => (
            format!("This is a development build. The newest release is {tag}."),
            Tone::Neutral,
        ),
        Status::Available(tag) if updater.can_install() => {
            (format!("{tag} is out."), Tone::Neutral)
        }
        Status::Available(tag) => (
            format!("{tag} is out. Its release page says how to update."),
            Tone::Neutral,
        ),
        Status::Installing(tag) => (
            format!("Installing {tag}. This can take a few minutes."),
            Tone::Neutral,
        ),
        Status::Installed(tag) => (format!("Restart {name} to use {tag}."), Tone::Neutral),
        Status::Failed(why) => (why.clone(), Tone::Error),
    };
    Row::new("Updates")
        .help(&help)
        .tone(tone)
        .show(ui, |ui, _| match &status {
            Status::Idle => {
                if ui
                    .add(Button::secondary("Check for updates").small())
                    .clicked()
                {
                    updater.check();
                }
            }
            Status::UpToDate | Status::Development(_) => {
                if ui.add(Button::secondary("Check again").small()).clicked() {
                    updater.check();
                }
            }
            Status::Checking | Status::Installing(_) => {
                widgets::spinner(ui);
            }
            Status::Available(tag) if updater.can_install() => {
                if ui
                    .add(Button::primary(format!("Update to {tag}")).small())
                    .clicked()
                {
                    updater.install();
                }
            }
            Status::Available(tag) => {
                link(
                    ui,
                    "Open release page",
                    &format!("{}/tag/{tag}", updater.releases_url()),
                );
            }
            Status::Installed(_) => {
                widgets::chip(ui, ChipKind::Ok, "Updated");
            }
            Status::Failed(_) => {
                if ui.add(Button::secondary("Try again").small()).clicked() {
                    updater.retry();
                }
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
