//! A file's tile in a list, drawn as the logos are: an ink tile with a faint
//! inner edge, a white mark and one accent, so a file sits level with the
//! vinoWhisper logo a session shows. The accent says which format the file
//! is; the file's name is written beside it. vinoSpeak drew the first seven
//! in its window (vinoSpeak#55); here they're shared with vinoSearch.
//!
//! The accents are the one place outside the logos where colour isn't state:
//! DESIGN.md's Colour section names them and their one job. Each is 3:1
//! against the ink (`cargo test contrast`).

use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, Ui};

use crate::logo::{self, GREEN, WHITE};
use crate::{a11y, icons, theme};

const RED: Color32 = Color32::from_rgb(0xe5, 0x48, 0x4d);
const BLUE: Color32 = Color32::from_rgb(0x6c, 0xa8, 0xff);
const ORANGE: Color32 = Color32::from_rgb(0xf5, 0xa0, 0x4a);
const PURPLE: Color32 = Color32::from_rgb(0xb9, 0x97, 0xff);
const YELLOW: Color32 = Color32::from_rgb(0xf2, 0xd2, 0x4b);
const TEAL: Color32 = Color32::from_rgb(0x3d, 0xcb, 0xb8);
const PINK: Color32 = Color32::from_rgb(0xf0, 0x7f, 0xc4);
const CYAN: Color32 = Color32::from_rgb(0x58, 0xd0, 0xf2);
const GREY: Color32 = Color32::from_rgb(0xa0, 0xa4, 0xac);

// The marks, on the tile's 56-unit grid from its top-left corner.
const PAGE: &str = "M34 10H18a4 4 0 0 0-4 4v28a4 4 0 0 0 4 4h20a4 4 0 0 0 4-4V18z";
/// Drawn without round ends: a cap at the page's corner would poke out of it.
const FOLD: &str = "M34 10v8h8";
const LINES: &str = "M21 28h14 M21 36h14";
const HASH: &str = "M26 24l-2 15 M34 24l-2 15 M21 29h15 M20 35h15";
const CHEVRONS: &str = "M22 26l-4 4 4 4 M34 26l4 4-4 4 M30 24l-4 12";
const SCREEN: &str = "M14 13h28a3 3 0 0 1 3 3v18a3 3 0 0 1-3 3H14a3 3 0 0 1-3-3V16a3 3 0 0 1 3-3z M28 37v7 M20 44h16";
const BARS: &str = "M21 30v-4 M28 30v-10 M35 30v-7";
const BOOK: &str = "M28 18C24 14 18 13 11 13v28c7 0 13 1 17 5c4-4 10-5 17-5V13c-7 0-13 1-17 5z";
const SPINE: &str = "M28 19v26";
const FRAME: &str = "M14 14h28a3 3 0 0 1 3 3v22a3 3 0 0 1-3 3H14a3 3 0 0 1-3-3V17a3 3 0 0 1 3-3z";
const CAPTION_LINES: &str = "M19 28h8 M33 28h4 M19 35h18";
/// Two rules and a column line, inside the frame.
const GRID: &str = "M15 26h26 M15 34h26 M24 17v22";
/// Hills inside the frame; the sun is a dot.
const HILLS: &str = "M15 37l8-9 6 6 4-4 8 7";

/// What a file is, as far as its tile goes. Which files an app lists is the
/// app's business: vinoSpeak leaves out sheets, code and images, vinoSearch
/// lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    Pdf,
    /// Word, OpenDocument and RTF.
    Writing,
    Slides,
    /// EPUB.
    Book,
    /// SRT and WebVTT.
    Subtitles,
    Markdown,
    /// Spreadsheets and CSV.
    Sheet,
    Code,
    Image,
    /// Any other text, or no extension at all.
    Plain,
}

impl Format {
    pub const ALL: [Format; 10] = [
        Format::Pdf,
        Format::Writing,
        Format::Slides,
        Format::Book,
        Format::Subtitles,
        Format::Markdown,
        Format::Sheet,
        Format::Code,
        Format::Image,
        Format::Plain,
    ];

    /// By extension, in any case, without its dot: the formats vinoSearch
    /// indexes (its `src/extract.rs`, `src/code.rs` and the images it
    /// reads). Anything else is `Plain`.
    pub fn from_extension(extension: &str) -> Self {
        match extension.to_ascii_lowercase().as_str() {
            "pdf" => Self::Pdf,
            "docx" | "docm" | "dotx" | "dotm" | "odt" | "ott" | "fodt" | "doc" | "dot" | "rtf" => {
                Self::Writing
            }
            "pptx" | "pptm" | "potx" | "ppsx" | "ppsm" | "odp" | "otp" | "fodp" | "ppt" | "pot"
            | "pps" => Self::Slides,
            "epub" => Self::Book,
            "srt" | "vtt" => Self::Subtitles,
            "md" | "markdown" | "mdx" => Self::Markdown,
            "xlsx" | "xlsm" | "xlsb" | "xltx" | "xltm" | "xls" | "ods" | "ots" | "csv" | "tsv" => {
                Self::Sheet
            }
            "rs" | "py" | "pyi" | "js" | "mjs" | "cjs" | "jsx" | "ts" | "mts" | "cts" | "tsx"
            | "vue" | "svelte" | "go" | "c" | "h" | "cc" | "cpp" | "cxx" | "hh" | "hpp" | "hxx"
            | "m" | "mm" | "java" | "kt" | "kts" | "scala" | "swift" | "dart" | "rb" | "php"
            | "pl" | "pm" | "lua" | "zig" | "nix" | "cs" | "fs" | "ex" | "exs" | "erl" | "hs"
            | "ml" | "mli" | "clj" | "r" | "jl" | "sh" | "bash" | "zsh" | "fish" | "ps1"
            | "css" | "scss" | "sass" | "less" | "sql" | "qml" | "cmake" | "gd" | "glsl"
            | "wgsl" | "proto" => Self::Code,
            "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp" | "tif" | "tiff" | "heic" | "heif"
            | "avif" => Self::Image,
            _ => Self::Plain,
        }
    }

    /// The format of a path, by its extension; `Plain` for none.
    pub fn of(path: &std::path::Path) -> Self {
        path.extension()
            .and_then(|e| e.to_str())
            .map_or(Self::Plain, Self::from_extension)
    }

    /// The accent, for the contrast test and the gallery's labels.
    pub fn accent(self) -> Color32 {
        match self {
            Format::Pdf => RED,
            Format::Writing => BLUE,
            Format::Slides => ORANGE,
            Format::Book => PURPLE,
            Format::Subtitles => YELLOW,
            Format::Markdown => GREEN,
            Format::Sheet => TEAL,
            Format::Code => PINK,
            Format::Image => CYAN,
            Format::Plain => GREY,
        }
    }

    /// What to call it, as in the gallery's hover text.
    pub fn name(self) -> &'static str {
        match self {
            Format::Pdf => "PDF",
            Format::Writing => "Writing",
            Format::Slides => "Slides",
            Format::Book => "Book",
            Format::Subtitles => "Subtitles",
            Format::Markdown => "Markdown",
            Format::Sheet => "Sheet",
            Format::Code => "Code",
            Format::Image => "Image",
            Format::Plain => "Plain text",
        }
    }
}

/// The tile in a `size` square, hidden from screen readers: the file's name
/// is written beside it. Lists draw it at 30.
pub fn file(ui: &mut Ui, format: Format, size: f32) -> Response {
    let (rect, response) = ui.allocate_exact_size(logo::square(size), Sense::hover());
    a11y::decorative(ui, response.id);
    if ui.is_rect_visible(rect) {
        paint(ui.painter(), rect, format);
    }
    response
}

pub fn paint(painter: &Painter, rect: Rect, format: Format) {
    let (k, _) = logo::tile(painter, rect);
    let origin = rect.min;
    let draw = |path: &str, color: Color32| stroke(painter, origin, k, path, color, true);
    let accent = format.accent();
    match format {
        Format::Pdf => {
            draw(PAGE, WHITE);
            stroke(painter, origin, k, FOLD, WHITE, false);
            // Across the page and past its edges, as a PDF's icon has it.
            let at = |x: f32, y: f32| origin + egui::vec2(x, y) * k;
            painter.rect_filled(
                Rect::from_min_max(at(8.0, 25.0), at(48.0, 40.0)),
                3.0 * k,
                accent,
            );
            painter.text(
                at(28.0, 32.5),
                Align2::CENTER_CENTER,
                "PDF",
                FontId::new(13.0 * k, theme::bold()),
                WHITE,
            );
        }
        Format::Writing | Format::Plain => {
            draw(PAGE, WHITE);
            stroke(painter, origin, k, FOLD, WHITE, false);
            draw(LINES, accent);
        }
        Format::Markdown => {
            draw(PAGE, WHITE);
            stroke(painter, origin, k, FOLD, WHITE, false);
            draw(HASH, accent);
        }
        Format::Code => {
            draw(PAGE, WHITE);
            stroke(painter, origin, k, FOLD, WHITE, false);
            draw(CHEVRONS, accent);
        }
        Format::Slides => {
            draw(SCREEN, WHITE);
            draw(BARS, accent);
        }
        Format::Book => {
            draw(BOOK, WHITE);
            draw(SPINE, accent);
        }
        Format::Subtitles => {
            draw(FRAME, WHITE);
            draw(CAPTION_LINES, accent);
        }
        // The frame goes on last, over the ends of the accent.
        Format::Sheet => {
            draw(GRID, accent);
            draw(FRAME, WHITE);
        }
        Format::Image => {
            draw(HILLS, accent);
            painter.circle_filled(origin + egui::vec2(36.0, 22.0) * k, 3.0 * k, accent);
            draw(FRAME, WHITE);
        }
    }
}

/// `path` on the tile's grid from `origin`, 4 units wide. Open subpaths get
/// round ends like the logos' strokes, unless `capped` is off.
fn stroke(painter: &Painter, origin: Pos2, k: f32, path: &str, color: Color32, capped: bool) {
    let stroke = Stroke::new(4.0 * k, color);
    for sub in icons::parse(path) {
        let points: Vec<Pos2> = sub
            .points
            .iter()
            .map(|p| origin + p.to_vec2() * k)
            .collect();
        if sub.closed {
            painter.add(Shape::closed_line(points, stroke));
        } else if capped {
            logo::capped(painter, points, stroke);
        } else {
            painter.add(Shape::line(points, stroke));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::logo::INK;
    use crate::theme::tests::contrast;

    /// vinoSearch's document formats (its `src/extract.rs`).
    const DOCUMENTS: &[&str] = &[
        "pdf", "docx", "docm", "dotx", "dotm", "odt", "ott", "fodt", "pptx", "pptm", "potx",
        "ppsx", "ppsm", "odp", "otp", "fodp", "xlsx", "xlsm", "xlsb", "xltx", "xltm", "xls", "ods",
        "ots", "csv", "tsv", "epub", "srt", "vtt", "doc", "dot", "ppt", "pot", "pps", "rtf",
    ];
    /// Its code (`src/code.rs`).
    const CODE: &[&str] = &[
        "rs", "py", "pyi", "js", "mjs", "cjs", "jsx", "ts", "mts", "cts", "tsx", "vue", "svelte",
        "go", "c", "h", "cc", "cpp", "cxx", "hh", "hpp", "hxx", "m", "mm", "java", "kt", "kts",
        "scala", "swift", "dart", "rb", "php", "pl", "pm", "lua", "zig", "nix", "cs", "fs", "ex",
        "exs", "erl", "hs", "ml", "mli", "clj", "r", "jl", "sh", "bash", "zsh", "fish", "ps1",
        "css", "scss", "sass", "less", "sql", "qml", "cmake", "gd", "glsl", "wgsl", "proto",
    ];
    /// The images it reads by default (`crates/shared/src/config.rs`).
    const IMAGES: &[&str] = &[
        "jpg", "jpeg", "png", "webp", "gif", "bmp", "tif", "tiff", "heic", "heif", "avif",
    ];

    /// An accent is a 4-unit mark on the ink: a graphic, so 3:1 (WCAG 1.4.11).
    #[test]
    fn accents_have_enough_contrast_on_the_ink() {
        for format in Format::ALL {
            let ratio = contrast(format.accent(), INK);
            assert!(ratio >= 3.0, "{format:?}'s accent is {ratio:.2}:1 on ink");
        }
        let mark = contrast(WHITE, INK);
        assert!(mark >= 4.5, "the white mark is {mark:.2}:1 on ink");
        // PDF writes on its band.
        let band = contrast(WHITE, Format::Pdf.accent());
        assert!(band >= 3.0, "PDF on its band is {band:.2}:1");
    }

    #[test]
    fn every_format_vinosearch_indexes_has_a_tile_of_its_own() {
        for ext in DOCUMENTS.iter().chain(CODE).chain(IMAGES) {
            assert_ne!(Format::from_extension(ext), Format::Plain, "{ext}");
        }
        for ext in CODE {
            assert_eq!(Format::from_extension(ext), Format::Code, "{ext}");
        }
        for ext in IMAGES {
            assert_eq!(Format::from_extension(ext), Format::Image, "{ext}");
        }
    }

    #[test]
    fn formats_go_by_extension_in_any_case() {
        assert_eq!(Format::from_extension("PDF"), Format::Pdf);
        assert_eq!(Format::from_extension("Docx"), Format::Writing);
        assert_eq!(Format::from_extension("odp"), Format::Slides);
        assert_eq!(Format::from_extension("epub"), Format::Book);
        assert_eq!(Format::from_extension("vtt"), Format::Subtitles);
        assert_eq!(Format::from_extension("md"), Format::Markdown);
        assert_eq!(Format::from_extension("csv"), Format::Sheet);
        assert_eq!(Format::from_extension("txt"), Format::Plain);
        assert_eq!(Format::from_extension(""), Format::Plain);
        assert_eq!(Format::of(Path::new("/a/b.PNG")), Format::Image);
        assert_eq!(Format::of(Path::new("/a/README")), Format::Plain);
        assert_eq!(Format::of(Path::new("/a/.bashrc")), Format::Plain);
    }

    #[test]
    fn every_format_has_a_name_and_a_mark_that_draws() {
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(Default::default(), |ui| {
            for format in Format::ALL {
                assert!(!format.name().is_empty());
                let r = file(ui, format, 30.0);
                assert_eq!(r.rect.size(), egui::vec2(30.0, 30.0));
            }
        });
        // Nothing draws the font texture here.
        out.textures_delta.clear();
    }
}
