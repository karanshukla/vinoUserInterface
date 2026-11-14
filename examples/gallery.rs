//! Every widget in a settings window laid out the way the vino apps lay theirs
//! out: a sidebar when the window is wide, underline tabs when it isn't, and
//! groups of rows in a capped column. `cargo run --example gallery`, then
//! drag the window narrower. tests/screenshots.rs draws the README's
//! pictures from it.

use eframe::egui;
use vino_user_interface::egui::{RichText, Ui};
use vino_user_interface::widgets::{self, Edge};
use vino_user_interface::{
    Button, ButtonKind, ChipKind, Column, Format, Icon, Row, Tone, a11y, file_tile, icons, logo,
    theme, window,
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
    /// The window buttons are the app's, in its header, not the desktop's.
    pub app_title_bar: bool,
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
            app_title_bar: false,
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
        let window = ctx.content_rect().width();
        let wide = window >= widgets::SIDEBAR_FROM;

        if wide {
            egui::Panel::left("pages")
                .resizable(false)
                .show_separator_line(false)
                .exact_size(widgets::sidebar_width(window))
                .frame(widgets::sidebar_frame(p))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    let brand = ui
                        .scope(|ui| {
                            widgets::sidebar_brand(
                                ui,
                                Some(logo::search),
                                "vino gallery",
                                "Settings",
                            );
                        })
                        .response
                        .rect;
                    if self.app_title_bar {
                        // After the brand, so its name drags the window rather than selecting.
                        let clip = ui.clip_rect();
                        window::drag(
                            ui,
                            egui::Rect::from_x_y_ranges(
                                clip.x_range(),
                                clip.top()..=brand.bottom(),
                            ),
                            "brand",
                        );
                    }
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
                    widgets::sidebar_foot(ui, |ui| {
                        a11y::live(&widgets::chip(ui, ChipKind::Ok, "Index up to date"));
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
                    if self.app_title_bar {
                        window::drag(ui, ui.clip_rect(), "header");
                    }
                    ui.horizontal(|ui| {
                        logo::search(ui, 22.0);
                        ui.add(
                            egui::Label::new(
                                RichText::new("vino gallery")
                                    .size(15.0)
                                    .family(theme::bold())
                                    .color(header.text),
                            )
                            .selectable(false),
                        );
                        if self.app_title_bar {
                            window::controls(ui);
                        }
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

        // With the app's title bar, a wide window's page title moves up out of
        // the scroll area into a bar of its own, with the window buttons at
        // its end: that bar is what the window is dragged by.
        let bar = wide && self.app_title_bar;
        if bar {
            window::bar(
                ui,
                widgets::page_inset(window),
                |ui| self.page_title(ui),
                |_| {},
            );
        }

        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(p.bg)
                    // No right margin: the scroll bar belongs on the window's edge.
                    .inner_margin(egui::Margin {
                        left: if wide {
                            widgets::page_inset(window) as i8
                        } else {
                            16
                        },
                        right: 0,
                        // The same 15 from title to content as in the scroll area.
                        top: if bar {
                            2
                        } else if wide {
                            20
                        } else {
                            12
                        },
                        bottom: 0,
                    }),
            )
            .show(ui, |ui| {
                // Pages of two columns get the room for both; the rest stop at one.
                let max = if self.page == Page::Shortcuts {
                    widgets::CONTENT_MAX
                } else {
                    widgets::WIDE_MAX
                };
                widgets::page_scroll(ui, self.page.title(), max, |ui| {
                    if !bar {
                        ui.horizontal(|ui| self.page_title(ui));
                    }
                    match self.page {
                        Page::General => self.general(ui),
                        Page::Models => self.models(ui),
                        Page::Shortcuts => shortcuts(ui),
                        Page::Widgets => self.widgets(ui),
                    }
                });
            });
    }

    fn page_title(&self, ui: &mut Ui) {
        widgets::title(ui, self.page.title());
        if self.page == Page::Widgets {
            let tag = widgets::tag(ui, Tone::Neutral, "Beta");
            a11y::decorative(ui, tag.id);
        }
    }

    fn general(&mut self, ui: &mut Ui) {
        widgets::search_field(
            ui,
            &mut self.query,
            "Search your files",
            "Search",
            ui.available_width(),
        );
        widgets::columns(ui, |ui, column| match column {
            Column::Left => self.what_to_index(ui),
            Column::Right => self.index(ui),
        });
    }

    fn what_to_index(&mut self, ui: &mut Ui) {
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
    }

    fn index(&mut self, ui: &mut Ui) {
        let p = theme::palette(ui.ctx());
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
        widgets::columns(ui, |ui, column| match column {
            Column::Left => self.vision_model(ui),
            Column::Right => text_model(ui),
        });
    }

    fn vision_model(&mut self, ui: &mut Ui) {
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
        });
    }

    fn widgets(&mut self, ui: &mut Ui) {
        widgets::columns(ui, |ui, column| match column {
            Column::Left => self.controls(ui),
            Column::Right => notices_and_art(ui),
        });
    }

    fn controls(&mut self, ui: &mut Ui) {
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
    }
}

fn text_model(ui: &mut Ui) {
    widgets::group_label(ui, "Text model");
    widgets::group(ui, |ui| {
        Row::new("Text model")
            .help("Downloading, 62 of 140 MB.")
            .show(ui, |ui, _| {
                widgets::progress(ui, 62.0 / 140.0, 120.0);
            });
    });
}

fn notices_and_art(ui: &mut Ui) {
    let p = theme::palette(ui.ctx());
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
                logo::authface(ui, 40.0);
            });
        });
        // At 30, the size the lists draw them.
        widgets::group_section(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(12.0, 12.0);
                for format in Format::ALL {
                    file_tile::file(ui, format, 30.0).on_hover_text(format.name());
                }
                logo::whisper(ui, 30.0).on_hover_text("Session");
            });
        });
    });
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
    let app_title_bar = window::app_title_bar();
    let options = eframe::NativeOptions {
        viewport: window::viewport(
            egui::ViewportBuilder::default()
                .with_title("vino gallery")
                .with_inner_size([960.0, 680.0])
                .with_min_inner_size([340.0, 320.0]),
            app_title_bar,
        ),
        ..Default::default()
    };
    eframe::run_native(
        "vino gallery",
        options,
        Box::new(|cc| {
            theme::install(&cc.egui_ctx);
            let mut app = App::default();
            app.gallery.app_title_bar = app_title_bar;
            Ok(Box::new(app))
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
        if self.gallery.app_title_bar {
            window::edges(ui);
        }
    }
}
