//! Every widget in a settings window laid out the way the vino apps lay theirs
//! out: a sidebar when the window is wide, underline tabs when it isn't, and
//! groups of rows in a capped column. `cargo run --example gallery`, then
//! drag the window narrower. tests/screenshots.rs draws the README's
//! pictures from it.

use eframe::egui;
use vino_user_interface::egui::{Align, Layout, RichText, Ui};
use vino_user_interface::widgets::{self, Edge};
use vino_user_interface::{
    Button, ButtonKind, ChipKind, Icon, Row, Tone, a11y, icons, logo, theme,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    General,
    Models,
    Shortcuts,
    Widgets,
}

impl Page {
    const ALL: [Page; 4] = [Page::General, Page::Models, Page::Shortcuts, Page::Widgets];

    fn title(self) -> &'static str {
        match self {
            Page::General => "General",
            Page::Models => "Models",
            Page::Shortcuts => "Shortcuts",
            Page::Widgets => "Widgets",
        }
    }

    fn icon(self) -> Icon {
        match self {
            Page::General => Icon::Settings,
            Page::Models => Icon::Model,
            Page::Shortcuts => Icon::Keyboard,
            Page::Widgets => Icon::Flask,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Device {
    Npu,
    Gpu,
    Cpu,
}

pub struct Gallery {
    pub page: Page,
    images: bool,
    documents: bool,
    minutes: u32,
    folder: String,
    query: String,
    confirm_delete: bool,
    device: Device,
    model: &'static str,
    only_images: bool,
    only_documents: bool,
}

impl Default for Gallery {
    fn default() -> Self {
        Gallery {
            page: Page::General,
            images: true,
            documents: true,
            minutes: 15,
            folder: "~/Pictures".to_owned(),
            query: String::new(),
            confirm_delete: false,
            device: Device::Npu,
            model: "SigLIP 2, base",
            only_images: true,
            only_documents: false,
        }
    }
}

impl Gallery {
    pub fn on(page: Page) -> Self {
        Gallery {
            page,
            ..Default::default()
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let p = theme::palette(&ctx);
        let header = theme::header(&ctx, None, true);
        let wide = ctx.content_rect().width() >= widgets::SIDEBAR_FROM;

        if wide {
            egui::Panel::left("pages")
                .resizable(false)
                .show_separator_line(false)
                .exact_size(widgets::SIDEBAR_WIDTH)
                .frame(widgets::sidebar_frame(p))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        logo::search(ui, 28.0);
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            ui.label(
                                RichText::new("vino gallery")
                                    .size(15.0)
                                    .family(theme::bold()),
                            );
                            widgets::small(ui, "Settings");
                        });
                    });
                    ui.add_space(14.0);
                    a11y::tab_list(ui, "Pages");
                    for page in Page::ALL {
                        let badge = (page == Page::Widgets).then_some("Beta");
                        if widgets::nav_item(
                            ui,
                            page.icon(),
                            page.title(),
                            page.title(),
                            self.page == page,
                            badge,
                        )
                        .clicked()
                        {
                            self.page = page;
                        }
                    }
                    ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                        ui.add_space(2.0);
                        ui.horizontal(|ui| {
                            ui.add_space(4.0);
                            a11y::live(&widgets::chip(ui, ChipKind::Ok, "Index up to date"));
                        });
                    });
                    widgets::panel_rule(ui, Edge::Left);
                });
        } else {
            egui::Panel::top("header")
                .show_separator_line(false)
                .frame(widgets::header_frame(header, p).inner_margin(egui::Margin {
                    left: 16,
                    right: 16,
                    top: 8,
                    bottom: 0,
                }))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        logo::search(ui, 22.0);
                        ui.label(
                            RichText::new("vino gallery")
                                .size(15.0)
                                .family(theme::bold())
                                .color(header.text),
                        );
                    });
                    // Wrapped: at the desktop's largest text the tabs don't fit on one line.
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(18.0, 0.0);
                        a11y::tab_list(ui, "Pages");
                        for page in Page::ALL {
                            if widgets::tab(ui, page.title(), self.page == page, true, header)
                                .clicked()
                            {
                                self.page = page;
                            }
                        }
                    });
                });
        }

        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(p.bg)
                    // No right margin: the scroll bar belongs on the window's edge.
                    .inner_margin(egui::Margin {
                        left: if wide { widgets::PAGE_INSET as i8 } else { 16 },
                        right: 0,
                        top: if wide { 20 } else { 12 },
                        bottom: 0,
                    }),
            )
            .show(ui, |ui| {
                widgets::page_scroll(ui, self.page.title(), widgets::CONTENT_MAX, |ui| {
                    ui.horizontal(|ui| {
                        widgets::title(ui, self.page.title());
                        if self.page == Page::Widgets {
                            let tag = widgets::tag(ui, Tone::Neutral, "Beta");
                            a11y::decorative(ui, tag.id);
                        }
                    });
                    match self.page {
                        Page::General => self.general(ui),
                        Page::Models => self.models(ui),
                        Page::Shortcuts => shortcuts(ui),
                        Page::Widgets => self.widgets(ui),
                    }
                });
            });
    }

    fn general(&mut self, ui: &mut Ui) {
        let p = theme::palette(ui.ctx());
        widgets::search_field(
            ui,
            &mut self.query,
            "Search your files",
            "Search",
            ui.available_width(),
        );
        widgets::group_label(ui, "What to index");
        widgets::group(ui, |ui| {
            Row::new("Images")
                .help("By what's in them, not just the name.")
                .show(ui, |ui, _| {
                    widgets::switch(ui, &mut self.images, "Images");
                });
            Row::new("Documents")
                .help("PDFs, Office files and plain text.")
                .show(ui, |ui, _| {
                    widgets::switch(ui, &mut self.documents, "Documents");
                });
            Row::new("Folder").show(ui, |ui, named| {
                named(ui.add(widgets::field(&mut self.folder, "~/Pictures", 180.0)));
            });
            Row::new("Unload the model after")
                .help("Minutes idle. 0 keeps it loaded.")
                .show(ui, |ui, named| {
                    named(ui.add(egui::DragValue::new(&mut self.minutes).suffix(" min")));
                });
        });

        widgets::group_label(ui, "Index");
        widgets::group(ui, |ui| {
            widgets::group_section(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 32.0;
                    widgets::stat(ui, "48,213", "files");
                    widgets::stat(ui, "3.1 GB", "on disk");
                    widgets::stat(ui, "2 min", "since the last change");
                });
                widgets::share_bar(
                    ui,
                    &[
                        (0.52, p.kinds[0]),
                        (0.27, p.kinds[1]),
                        (0.15, p.kinds[2]),
                        (0.06, p.kinds[3]),
                    ],
                    8.0,
                );
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 16.0;
                    for (i, kind) in ["Text", "Documents", "Images", "Code"]
                        .into_iter()
                        .enumerate()
                    {
                        widgets::legend(ui, p.kinds[i], kind);
                    }
                });
            });
            let delete = Row::new("Delete the index").help(if self.confirm_delete {
                "Every file is read again from scratch. This can take an hour."
            } else {
                "Starts again from nothing."
            });
            let delete = if self.confirm_delete {
                delete.tint(p.error_tint).tone(Tone::Error)
            } else {
                delete
            };
            delete.show(ui, |ui, _| {
                if self.confirm_delete {
                    if ui.add(Button::destructive("Delete")).clicked() {
                        self.confirm_delete = false;
                    }
                    if ui.add(Button::ghost("Cancel")).clicked() {
                        self.confirm_delete = false;
                    }
                } else if ui.add(Button::secondary("Delete…")).clicked() {
                    self.confirm_delete = true;
                }
            });
        });
    }

    fn models(&mut self, ui: &mut Ui) {
        widgets::banner(
            ui,
            Tone::Warn,
            "The vision model isn't downloaded yet, so images are found by name only.",
            |ui| {
                ui.add(Button::primary("Download").icon(Icon::Download).small());
            },
        );
        widgets::group_label(ui, "Vision model");
        widgets::group(ui, |ui| {
            Row::new("Runs on")
                .help("The NPU is the quietest on battery.")
                .show(ui, |ui, _| {
                    widgets::segmented(
                        ui,
                        "Runs on",
                        &mut self.device,
                        &[
                            (Device::Npu, "NPU"),
                            (Device::Gpu, "GPU"),
                            (Device::Cpu, "CPU"),
                        ],
                    );
                });
            Row::new("Model").show(ui, |ui, named| {
                let picked =
                    widgets::combo(ui, "model", self.model.to_owned(), 180.0).show_ui(ui, |ui| {
                        for model in ["SigLIP 2, base", "SigLIP 2, large"] {
                            ui.selectable_value(&mut self.model, model, model);
                        }
                    });
                named(picked.response);
            });
            Row::new("Download")
                .help("Checked against pinned sha256s.")
                .show(ui, |ui, _| {
                    ui.add(Button::secondary("Verify").small());
                    widgets::chip(ui, ChipKind::Warn, "Not downloaded");
                });
            Row::new("Text model")
                .help("Downloading, 62 of 140 MB.")
                .show(ui, |ui, _| {
                    widgets::progress(ui, 62.0 / 140.0, 120.0);
                });
        });
    }

    fn widgets(&mut self, ui: &mut Ui) {
        let p = theme::palette(ui.ctx());
        widgets::group_label(ui, "Buttons");
        widgets::group(ui, |ui| {
            widgets::group_section(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.add(Button::primary("Save"));
                    ui.add(Button::secondary("Open folder").icon(Icon::Folder));
                    ui.add(Button::ghost("Cancel"));
                    ui.add(Button::destructive("Delete"));
                    widgets::icon_button(ui, ButtonKind::Secondary, Icon::Refresh, "Refresh");
                    widgets::icon_button(ui, ButtonKind::Ghost, Icon::More, "More");
                });
            });
        });
        widgets::group_label(ui, "Status");
        widgets::group(ui, |ui| {
            widgets::group_section(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    widgets::chip(ui, ChipKind::Ok, "Running");
                    widgets::chip(ui, ChipKind::Warn, "Needs a restart");
                    widgets::chip(ui, ChipKind::Err, "Failed");
                    widgets::chip(ui, ChipKind::Neutral, "Paused");
                });
                // A wait nothing else shows: the spinner, beside what it waits for.
                ui.horizontal(|ui| {
                    widgets::spinner(ui);
                    widgets::small(ui, "Connecting to the service");
                });
                ui.horizontal_wrapped(|ui| {
                    widgets::toggle_chip(ui, &mut self.only_images, "Images", Some(128));
                    widgets::toggle_chip(ui, &mut self.only_documents, "Documents", Some(42));
                    widgets::tag(ui, Tone::Neutral, "Beta");
                    widgets::tag(ui, Tone::Neutral, "PDF");
                    widgets::keys(ui, "Ctrl+Shift+F");
                });
            });
        });
        widgets::group_label(ui, "Notices");
        widgets::banner(
            ui,
            Tone::Info,
            "Indexing pauses on battery below 20 %.",
            |_| {},
        );
        ui.add_space(8.0);
        widgets::notice(ui, Tone::Error, "The index couldn't be opened: disk full.");
        widgets::group_label(ui, "Icons and logos");
        widgets::group(ui, |ui| {
            widgets::group_section(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(14.0, 12.0);
                    for icon in Icon::ALL {
                        icons::icon(ui, icon, 18.0, p.text).on_hover_text(format!("{icon:?}"));
                    }
                });
            });
            widgets::group_section(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 16.0;
                    logo::search(ui, 40.0);
                    logo::whisper(ui, 40.0);
                    logo::speak(ui, 40.0);
                });
            });
        });
    }
}

fn shortcuts(ui: &mut Ui) {
    widgets::group_help(
        ui,
        "Set in your desktop's keyboard settings; shown here as they are bound.",
    );
    widgets::group(ui, |ui| {
        for (what, help, keys) in [
            ("Search", "From anywhere.", "Meta+Space"),
            ("Dictate", "Hold to talk, let go to type.", "Meta+Alt+D"),
            ("Read the selection aloud", "", "Meta+Alt+R"),
            ("Captions", "Live, over whatever is playing.", "Meta+Alt+C"),
        ] {
            Row::new(what).help(help).show(ui, |ui, _| {
                widgets::keys(ui, keys);
            });
        }
    });
}

#[allow(dead_code)]
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("vino gallery")
            .with_inner_size([960.0, 680.0])
            .with_min_inner_size([340.0, 320.0]),
        ..Default::default()
    };
    eframe::run_native(
        "vino gallery",
        options,
        Box::new(|cc| {
            theme::install(&cc.egui_ctx);
            Ok(Box::new(App::default()))
        }),
    )
}

#[derive(Default)]
struct App {
    gallery: Gallery,
    focused: bool,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // The desktop's light/dark switch may have flipped while the window was in the background.
        let focused = ui.input(|input| input.focused);
        if focused && !self.focused {
            theme::follow_desktop(ui.ctx());
        }
        self.focused = focused;
        self.gallery.ui(ui);
    }
}
