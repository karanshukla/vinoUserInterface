//! Tokens, fonts and egui's visuals, and what the desktop asks for.
//!
//! The colours are vinopilot.ca's site.css plus states (vinoSearch#148):
//! green only ever means "this one" (on, selected, done), amber means "needs
//! you" and red means "failed". Text is 4.5:1 and the edges of controls 3:1
//! against both the page and a field, in both themes; `tests` checks that
//! over the visuals as installed.

use std::sync::Arc;

use egui::{
    Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Shadow, Stroke,
    TextStyle, Theme, Visuals,
};

mod desktop;

pub use desktop::{
    KdeHeader, TEXT_SCALE, desktop_preference, desktop_text_scale, follow_desktop, kde_header,
    reduced_motion, text_scale,
};

/// One theme's colours.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    /// The page canvas.
    pub bg: Color32,
    /// The sidebar, headers, and neutral strips.
    pub surface: Color32,
    /// Text fields, combo boxes, secondary buttons, menus.
    pub field: Color32,
    /// A pressed control; the track of a switch that is off.
    pub pressed: Color32,
    /// Hairlines between rows and bounding groups.
    pub rule: Color32,
    /// The outline of a button or a menu.
    pub rule_strong: Color32,
    /// The outline of a field or an unchecked box, and the rails and handles
    /// of sliders and scroll bars: 3:1 against the page and a field, as WCAG
    /// asks of the edges that make a control findable.
    pub edge: Color32,
    pub text: Color32,
    pub muted: Color32,
    /// Links, the selected nav item's icon, focus rings.
    pub accent: Color32,
    /// The primary button's fill: ink on light, near-white on dark. Never
    /// green: green is kept for on, selected and done.
    pub primary_fill: Color32,
    /// Text on `primary_fill`.
    pub primary_text: Color32,
    /// The track of a switch that is on.
    pub on: Color32,
    /// The knob of a switch that is on, on `on`.
    pub switch_knob: Color32,
    /// A destructive button's fill, and its text.
    pub danger: Color32,
    pub danger_text: Color32,
    /// A selected row, nav item or chip: the accent at 10–14 %.
    pub tint: Color32,
    pub warn: Color32,
    pub warn_tint: Color32,
    /// The hairline round a warning banner.
    pub warn_rule: Color32,
    pub error: Color32,
    pub error_tint: Color32,
    /// The hairline round an error banner.
    pub error_rule: Color32,
    /// The index's kinds, for share bars: text, documents, images, code.
    pub kinds: [Color32; 4],
}

const fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

/// `hex` at full opacity, faded to `alpha` and stored premultiplied, as
/// `Color32` keeps it (pinned by `tests::rgba_premultiplies_the_full_opacity_hex`).
const fn rgba(hex: u32, alpha: u8) -> Color32 {
    const fn channel(c: u32, alpha: u8) -> u8 {
        (c * alpha as u32 / 255) as u8
    }
    Color32::from_rgba_premultiplied(
        channel((hex >> 16) & 0xff, alpha),
        channel((hex >> 8) & 0xff, alpha),
        channel(hex & 0xff, alpha),
        alpha,
    )
}

pub const DARK: Palette = Palette {
    bg: rgb(0x0f1012),
    surface: rgb(0x17181b),
    field: rgb(0x1f2124),
    pressed: rgb(0x2b2d31),
    rule: rgb(0x26282c),
    rule_strong: rgb(0x3a3d43),
    edge: rgb(0x6e727a),
    text: rgb(0xe8e9ec),
    muted: rgb(0x9ea2aa),
    accent: rgb(0x6fd496),
    primary_fill: rgb(0xecedef),
    primary_text: rgb(0x111214),
    on: rgb(0x4cc47a),
    switch_knob: rgb(0x0f1012),
    danger: rgb(0xc4362c),
    danger_text: rgb(0xffffff),
    tint: rgba(0x4cc47a, 33),
    warn: rgb(0xf0b45e),
    warn_tint: rgba(0xf0b45e, 26),
    warn_rule: rgba(0xf0b45e, 71),
    error: rgb(0xf2877e),
    error_tint: rgba(0xf2877e, 26),
    error_rule: rgba(0xf2877e, 71),
    kinds: [rgb(0x56cc78), rgb(0x6ea8fe), rgb(0xb38cf0), rgb(0x9a9ea6)],
};

pub const LIGHT: Palette = Palette {
    bg: rgb(0xffffff),
    surface: rgb(0xf6f7f8),
    field: rgb(0xf0f1f3),
    pressed: rgb(0xe5e7ea),
    rule: rgb(0xe5e7eb),
    rule_strong: rgb(0xcdd0d5),
    edge: rgb(0x858a92),
    text: rgb(0x17181a),
    muted: rgb(0x5b5f67),
    accent: rgb(0x166f44),
    primary_fill: rgb(0x17181a),
    primary_text: rgb(0xffffff),
    on: rgb(0x1e8a50),
    switch_knob: rgb(0xffffff),
    danger: rgb(0xb3261e),
    danger_text: rgb(0xffffff),
    tint: rgba(0x166f44, 24),
    warn: rgb(0x8a5300),
    warn_tint: rgba(0xb26e00, 24),
    warn_rule: rgba(0xb26e00, 64),
    error: rgb(0xb3261e),
    error_tint: rgba(0xb3261e, 20),
    error_rule: rgba(0xb3261e, 56),
    kinds: [rgb(0x1f8a4c), rgb(0x2f6fdb), rgb(0x7b4fd0), rgb(0x6b6f77)],
};

impl Palette {
    /// A hovered control: halfway from `field` to `pressed`.
    pub fn hovered(&self) -> Color32 {
        self.field.lerp_to_gamma(self.pressed, 0.5)
    }

    /// The colour of the shadow under a menu or popup, the only shadowed
    /// surface: 0·12·32 at 45 % (dark) or 16 % (light).
    pub fn shadow(&self) -> Shadow {
        let color = if self.bg == DARK.bg {
            Color32::from_black_alpha(115)
        } else {
            Color32::from_rgba_unmultiplied(20, 20, 30, 41)
        };
        Shadow {
            offset: [0, 12],
            blur: 32,
            spread: 0,
            color,
        }
    }
}

pub fn palette(ctx: &egui::Context) -> &'static Palette {
    match ctx.theme() {
        Theme::Dark => &DARK,
        Theme::Light => &LIGHT,
    }
}

/// Inter at 500: row titles, button text.
pub fn medium() -> FontFamily {
    FontFamily::Name("Inter-Medium".into())
}

/// Inter at 600: window titles, group labels, selected nav items, key caps.
pub fn semibold() -> FontFamily {
    FontFamily::Name("Inter-SemiBold".into())
}

/// Inter at 700: page titles and stats.
pub fn bold() -> FontFamily {
    FontFamily::Name("Inter-Bold".into())
}

/// JetBrains Mono: paths and commands only.
pub fn mono() -> FontFamily {
    FontFamily::Monospace
}

/// Each family's font first, then Inter Regular, then egui's stock fonts for
/// the glyphs the Latin subsets lack (pinned by
/// `tests::every_family_is_installed_with_the_stock_fonts_behind_it`).
fn fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    for (name, bytes) in [
        (
            "Inter-Regular",
            &include_bytes!("../../assets/fonts/Inter-Regular.ttf")[..],
        ),
        (
            "Inter-Medium",
            &include_bytes!("../../assets/fonts/Inter-Medium.ttf")[..],
        ),
        (
            "Inter-SemiBold",
            &include_bytes!("../../assets/fonts/Inter-SemiBold.ttf")[..],
        ),
        (
            "Inter-Bold",
            &include_bytes!("../../assets/fonts/Inter-Bold.ttf")[..],
        ),
        (
            "JetBrainsMono-Regular",
            &include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf")[..],
        ),
    ] {
        fonts
            .font_data
            .insert(name.to_owned(), Arc::new(FontData::from_static(bytes)));
    }
    let stock: Vec<String> = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let family = |first: &str| fallback_chain(first, &stock);
    fonts
        .families
        .insert(FontFamily::Proportional, family("Inter-Regular"));
    fonts.families.insert(medium(), family("Inter-Medium"));
    fonts.families.insert(semibold(), family("Inter-SemiBold"));
    fonts.families.insert(bold(), family("Inter-Bold"));
    let mono = fonts.families.entry(FontFamily::Monospace).or_default();
    mono.insert(0, "JetBrainsMono-Regular".to_owned());
    mono.push("Inter-Regular".to_owned());
    fonts
}

fn fallback_chain(first: &str, stock: &[String]) -> Vec<String> {
    let mut list = vec![first.to_owned()];
    if first != "Inter-Regular" {
        list.push("Inter-Regular".to_owned());
    }
    list.extend(stock.iter().cloned());
    list
}

/// A pressed control's, and a keyboard focus's, accent ring.
const FOCUS_RING: f32 = 2.0;
const CONTROL_CORNER: CornerRadius = CornerRadius::same(8);
const POPUP_CORNER: CornerRadius = CornerRadius::same(10);

fn visuals(p: &Palette, base: Visuals) -> Visuals {
    let mut v = base;
    v.panel_fill = p.bg;
    popups(&mut v, p);
    v.extreme_bg_color = p.field;
    v.faint_bg_color = p.field;
    v.code_bg_color = p.field;
    v.hyperlink_color = p.accent;
    v.warn_fg_color = p.warn;
    v.error_fg_color = p.error;
    v.selection.bg_fill = p.tint;
    v.selection.stroke = Stroke::new(1.0_f32, p.accent);
    v.text_cursor.stroke.color = p.text;
    widget_states(&mut v.widgets, p);
    v
}

/// Menus and popups: a field-coloured surface with a strong rule, and the
/// only shadow anywhere.
fn popups(v: &mut Visuals, p: &Palette) {
    v.window_fill = p.field;
    v.window_stroke = Stroke::new(1.0_f32, p.rule_strong);
    v.window_shadow = p.shadow();
    v.popup_shadow = p.shadow();
    v.menu_corner_radius = POPUP_CORNER;
    v.window_corner_radius = POPUP_CORNER;
}

/// Outlines in `edge` at rest and hovered, so a field or an unchecked box
/// shows against the page (pinned by
/// `tests::control_edges_have_enough_contrast`); `FOCUS_RING` in the accent
/// when pressed or focused.
fn widget_states(w: &mut egui::style::Widgets, p: &Palette) {
    w.noninteractive.bg_fill = p.bg;
    w.noninteractive.weak_bg_fill = p.surface;
    w.noninteractive.bg_stroke = Stroke::new(1.0_f32, p.rule);
    w.noninteractive.fg_stroke = Stroke::new(1.0_f32, p.text);
    w.noninteractive.corner_radius = CONTROL_CORNER;
    for widget in [&mut w.inactive, &mut w.hovered, &mut w.active, &mut w.open] {
        widget.corner_radius = CONTROL_CORNER;
        widget.fg_stroke = Stroke::new(1.0_f32, p.text);
    }
    w.inactive.bg_fill = p.field;
    w.inactive.weak_bg_fill = p.field;
    w.inactive.bg_stroke = Stroke::new(1.0_f32, p.edge);
    w.hovered.bg_fill = p.hovered();
    w.hovered.weak_bg_fill = p.hovered();
    w.hovered.bg_stroke = Stroke::new(1.0_f32, p.edge);
    w.hovered.expansion = 0.0;
    w.active.bg_fill = p.pressed;
    w.active.weak_bg_fill = p.pressed;
    w.active.bg_stroke = Stroke::new(FOCUS_RING, p.accent);
    w.active.expansion = 0.0;
    w.open.bg_fill = p.pressed;
    w.open.weak_bg_fill = p.pressed;
    w.open.bg_stroke = Stroke::new(1.0_f32, p.accent);
}

/// A solid scroll bar paints its handle in `bg_fill` on an `extreme_bg_color`
/// track, and the visuals above make both the field colour, so the handle
/// vanished. Nothing else here uses `bg_fill` (buttons take `weak_bg_fill`),
/// so give it the edge grey, lighter while hovered and green while dragged.
/// Checkboxes and slider rails paint in `bg_fill` too, so they take the same
/// grey.
pub fn visible_scroll_handles(ctx: &egui::Context) {
    for (theme, p) in [(Theme::Dark, &DARK), (Theme::Light, &LIGHT)] {
        ctx.style_mut_of(theme, |style| {
            let w = &mut style.visuals.widgets;
            w.inactive.bg_fill = p.edge;
            w.hovered.bg_fill = p.muted;
            w.active.bg_fill = p.accent;
        });
    }
}

/// Fonts, colours, sizes and spacing, without asking the desktop anything.
pub fn apply(ctx: &egui::Context) {
    ctx.set_fonts(fonts());
    ctx.set_visuals_of(Theme::Dark, visuals(&DARK, Visuals::dark()));
    ctx.set_visuals_of(Theme::Light, visuals(&LIGHT, Visuals::light()));
    ctx.all_styles_mut(|style| {
        style
            .text_styles
            .insert(TextStyle::Body, FontId::proportional(14.0));
        style
            .text_styles
            .insert(TextStyle::Button, FontId::new(13.5, medium()));
        style
            .text_styles
            .insert(TextStyle::Small, FontId::proportional(12.5));
        style
            .text_styles
            .insert(TextStyle::Heading, FontId::new(22.0, bold()));
        style
            .text_styles
            .insert(TextStyle::Monospace, FontId::monospace(12.5));
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        style.spacing.interact_size.y = 30.0;
        style.spacing.menu_margin = egui::Margin::same(4);
        style.spacing.combo_width = 200.0;
        style.spacing.scroll = egui::style::ScrollStyle {
            bar_width: crate::widgets::SCROLL_BAR,
            ..egui::style::ScrollStyle::solid()
        };
    });
    visible_scroll_handles(ctx);
}

/// `apply`, then the desktop's theme and motion preference.
pub fn install(ctx: &egui::Context) {
    apply(ctx);
    follow_desktop(ctx);
}

/// No animation time is the signal: egui's own transitions go instant and
/// [`crate::widgets::spinner`] stops turning.
pub fn still(ctx: &egui::Context) -> bool {
    ctx.global_style().animation_time == 0.0
}

/// The colours of a window's top strip.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Header {
    pub bg: Color32,
    pub text: Color32,
    pub muted: Color32,
}

/// The header's colours: KDE's when it has them, else the surface.
pub fn header(ctx: &egui::Context, kde: Option<&KdeHeader>, focused: bool) -> Header {
    match kde {
        Some(kde) if focused => kde.active,
        Some(kde) => kde.inactive,
        None => {
            let p = palette(ctx);
            Header {
                bg: p.surface,
                text: p.text,
                muted: p.muted,
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// WCAG 2.x relative luminance of an sRGB colour.
    fn luminance(c: Color32) -> f64 {
        let channel = |v: u8| {
            let v = f64::from(v) / 255.0;
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(c.r()) + 0.7152 * channel(c.g()) + 0.0722 * channel(c.b())
    }

    pub fn contrast(a: Color32, b: Color32) -> f64 {
        let (a, b) = (luminance(a), luminance(b));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    /// A translucent tint as it lands on `over`.
    fn flatten(tint: Color32, over: Color32) -> Color32 {
        let a = f64::from(tint.a()) / 255.0;
        let mix = |t: u8, o: u8| (f64::from(t) * a + f64::from(o) * (1.0 - a)).round() as u8;
        Color32::from_rgb(
            mix(tint.r(), over.r()),
            mix(tint.g(), over.g()),
            mix(tint.b(), over.b()),
        )
    }

    #[test]
    fn contrast_ratio_matches_the_wcag_examples() {
        assert!((contrast(Color32::BLACK, Color32::WHITE) - 21.0).abs() < 0.01);
        assert!((contrast(rgb(0x777777), Color32::WHITE) - 4.48).abs() < 0.01);
    }

    /// The palette and the visuals as `apply` installs them, for each theme.
    fn installed() -> Vec<(Theme, &'static Palette, egui::Style)> {
        let ctx = egui::Context::default();
        apply(&ctx);
        [(Theme::Dark, &DARK), (Theme::Light, &LIGHT)]
            .into_iter()
            .map(|(theme, p)| (theme, p, (*ctx.style_of(theme)).clone()))
            .collect()
    }

    /// The page, a surface and a field.
    fn plain_grounds(p: &Palette) -> [(&'static str, Color32); 3] {
        [("page", p.bg), ("surface", p.surface), ("field", p.field)]
    }

    /// 4.5:1 for text (WCAG 1.4.3) on the page, a surface, a field and the
    /// three tints.
    #[test]
    fn text_has_enough_contrast() {
        for (theme, p, style) in installed() {
            let v = &style.visuals;
            let w = &v.widgets;
            let tints = [
                ("tint", flatten(p.tint, p.surface)),
                ("warn tint", flatten(p.warn_tint, p.surface)),
                ("error tint", flatten(p.error_tint, p.surface)),
            ];
            for (what, fg) in [
                ("text", p.text),
                ("muted", p.muted),
                ("link", v.hyperlink_color),
                ("warning", v.warn_fg_color),
                ("error", v.error_fg_color),
                ("button text", w.inactive.fg_stroke.color),
                ("noninteractive text", w.noninteractive.fg_stroke.color),
            ] {
                for (on, bg) in plain_grounds(p).into_iter().chain(tints) {
                    let ratio = contrast(fg, bg);
                    assert!(ratio >= 4.5, "{theme:?}: {what} on {on} is {ratio:.2}:1");
                }
            }
        }
    }

    #[test]
    fn primary_and_destructive_button_text_has_enough_contrast() {
        for (theme, p, _) in installed() {
            let primary = contrast(p.primary_text, p.primary_fill);
            assert!(
                primary >= 4.5,
                "{theme:?}: primary button text is {primary:.2}:1"
            );
            let destructive = contrast(p.danger_text, p.danger);
            assert!(
                destructive >= 4.5,
                "{theme:?}: destructive button text is {destructive:.2}:1"
            );
        }
    }

    /// 3:1 for the edges that make a control findable (WCAG 1.4.11), on the
    /// page, a surface and a field.
    #[test]
    fn control_edges_have_enough_contrast() {
        for (theme, p, style) in installed() {
            let v = &style.visuals;
            let w = &v.widgets;
            for (what, edge) in [
                ("checkbox and field outline", w.inactive.bg_stroke.color),
                ("checkbox box, rail, scroll handle", w.inactive.bg_fill),
                ("hovered outline", w.hovered.bg_stroke.color),
                ("focus outline", w.active.bg_stroke.color),
                ("text selection outline", v.selection.stroke.color),
                ("switch on", p.on),
            ] {
                for (on, bg) in plain_grounds(p) {
                    let ratio = contrast(edge, bg);
                    assert!(ratio >= 3.0, "{theme:?}: {what} on {on} is {ratio:.2}:1");
                }
            }
        }
    }

    #[test]
    fn a_checkbox_tick_and_a_switch_knob_have_enough_contrast() {
        for (theme, p, style) in installed() {
            let w = &style.visuals.widgets;
            let tick = contrast(w.inactive.fg_stroke.color, w.inactive.bg_fill);
            assert!(tick >= 3.0, "{theme:?}: checkbox tick is {tick:.2}:1");
            let knob = contrast(p.switch_knob, p.on);
            assert!(knob >= 3.0, "{theme:?}: switch knob is {knob:.2}:1");
        }
    }

    #[test]
    fn rgba_premultiplies_the_full_opacity_hex() {
        assert_eq!(
            rgba(0xff8000, 128),
            Color32::from_rgba_premultiplied(128, 64, 0, 128)
        );
        assert_eq!(rgba(0x4cc47a, 255), rgb(0x4cc47a));
    }

    #[test]
    fn every_family_is_installed_with_the_stock_fonts_behind_it() {
        let fonts = fonts();
        for family in [FontFamily::Proportional, medium(), semibold(), bold()] {
            let list = &fonts.families[&family];
            assert!(list[0].starts_with("Inter-"), "{family:?}: {list:?}");
            assert!(list.len() > 2, "{family:?} has no fallback: {list:?}");
        }
        assert_eq!(
            fonts.families[&FontFamily::Monospace][0],
            "JetBrainsMono-Regular"
        );
    }
}
