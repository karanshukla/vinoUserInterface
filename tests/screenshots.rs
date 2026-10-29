//! The README's pictures, drawn from examples/gallery.rs with wgpu:
//! `cargo test --test screenshots -- --ignored`, then commit docs/images.

#[path = "../examples/gallery.rs"]
mod gallery;

use egui::Theme;
use egui_kittest::Harness;
use gallery::{Gallery, Page};
use image::RgbaImage;
use vino_user_interface::{egui, theme};

const SCALE: f32 = 2.0;
/// The outer margin kittest puts round a `build_ui` app, cut off again.
const MARGIN: f32 = 8.0;

fn render(theme: Theme, page: Page, width: f32, height: f32) -> RgbaImage {
    // The harness draws a frame as it is built, so the theme goes in on that
    // first frame, as tests/a11y.rs does it.
    let mut h = Harness::builder()
        .with_size((width + 2.0 * MARGIN, height + 2.0 * MARGIN))
        .with_pixels_per_point(SCALE)
        .wgpu()
        .build_ui_state(
            move |ui, (gallery, styled): &mut (Gallery, bool)| {
                if !*styled {
                    theme::apply(ui.ctx());
                    ui.ctx().set_theme(theme);
                    // Still: switches at rest and a spinner that asks for no more frames.
                    ui.ctx().all_styles_mut(|style| style.animation_time = 0.0);
                    *styled = true;
                    ui.ctx().request_repaint();
                    return;
                }
                gallery.ui(ui);
            },
            (Gallery::on(page), false),
        );
    h.run_steps(4);
    let full = h.render().expect("wgpu renders");
    let m = (MARGIN * SCALE) as u32;
    image::imageops::crop_imm(&full, m, m, full.width() - 2 * m, full.height() - 2 * m).to_image()
}

fn save(image: &RgbaImage, name: &str) {
    let path = format!("{}/docs/images/{name}.png", env!("CARGO_MANIFEST_DIR"));
    image.save(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
}

/// Two windows snapped side by side on a 1366 x 768 laptop, 8 px apart.
fn side_by_side(left: &RgbaImage, right: &RgbaImage) -> RgbaImage {
    let gap = (8.0 * SCALE) as u32;
    let mut out = RgbaImage::from_pixel(
        left.width() + gap + right.width(),
        left.height().max(right.height()),
        image::Rgba([0x3a, 0x3c, 0x42, 0xff]),
    );
    image::imageops::overlay(&mut out, left, 0, 0);
    image::imageops::overlay(&mut out, right, i64::from(left.width() + gap), 0);
    out
}

#[test]
#[ignore = "writes docs/images; run by hand"]
fn readme_pictures() {
    save(
        &render(Theme::Dark, Page::General, 960.0, 640.0),
        "settings-dark",
    );
    save(
        &render(Theme::Light, Page::Models, 960.0, 640.0),
        "settings-light",
    );
    save(
        &render(Theme::Dark, Page::Widgets, 960.0, 900.0),
        "widgets-dark",
    );
    save(
        &render(Theme::Light, Page::Widgets, 960.0, 900.0),
        "widgets-light",
    );
    let half = (1366.0 - 8.0) / 2.0;
    save(
        &side_by_side(
            &render(Theme::Dark, Page::General, half, 700.0),
            &render(Theme::Light, Page::Shortcuts, half, 700.0),
        ),
        "side-by-side",
    );
    save(&render(Theme::Dark, Page::General, 380.0, 700.0), "narrow");
}
