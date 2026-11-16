//! What the desktop asks for: light or dark, less motion, bigger text, and
//! KDE's header colours. Read from the Settings portal and Plasma's config
//! files; whatever doesn't parse is ignored.

use egui::{Color32, Theme};

use super::Header;

const PORTAL: &str = "org.freedesktop.portal.Desktop";
const PORTAL_PATH: &str = "/org/freedesktop/portal/desktop";
/// A portal that is still starting would otherwise hold the frame for
/// busctl's and gdbus's default 25 s.
const PORTAL_TIMEOUT: &str = "--timeout=1";

/// One value from the portal's Settings, as busctl prints it ("v u 1").
/// Through gdbus where there is no busctl, as in a Flatpak's runtime.
fn read_setting(namespace: &str, key: &str) -> Option<String> {
    let reply = match read_with_busctl(namespace, key) {
        Some(reply) => reply,
        None => from_gdbus(&read_with_gdbus(namespace, key)?),
    };
    (!reply.trim().is_empty()).then_some(reply)
}

fn read_with_busctl(namespace: &str, key: &str) -> Option<String> {
    run(
        "busctl",
        &[
            "--user",
            PORTAL_TIMEOUT,
            "call",
            PORTAL,
            PORTAL_PATH,
            "org.freedesktop.portal.Settings",
            "ReadOne",
            "ss",
            namespace,
            key,
        ],
    )
}

fn read_with_gdbus(namespace: &str, key: &str) -> Option<String> {
    run(
        "gdbus",
        &[
            "call",
            "--session",
            PORTAL_TIMEOUT,
            "--dest",
            PORTAL,
            "--object-path",
            PORTAL_PATH,
            "--method",
            "org.freedesktop.portal.Settings.ReadOne",
            namespace,
            key,
        ],
    )
}

/// `program`'s stdout, or empty when it refused; None when it isn't there.
fn run(program: &str, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new(program)
        .args(args)
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    Some(if output.status.success() {
        String::from_utf8_lossy(&output.stdout).into_owned()
    } else {
        String::new()
    })
}

/// gdbus's "(<uint32 1>,)" with the brackets off, so its value is the last
/// word as in busctl's "v u 1" (pinned by `tests::gdbus_answers_read_as_busctls_do`).
fn from_gdbus(reply: &str) -> String {
    reply
        .trim()
        .trim_start_matches("(<")
        .trim_end_matches(">,)")
        .to_owned()
}

/// Light or dark. winit on Wayland only knows the theme of decorations it
/// draws itself, so egui would fall back to dark on a light desktop: ask the
/// portal instead. `VINO_UI_THEME=light|dark` overrides it, for screenshots
/// on a desktop without a portal.
pub fn desktop_preference() -> Option<Theme> {
    match std::env::var("VINO_UI_THEME").as_deref() {
        Ok("light") => return Some(Theme::Light),
        Ok("dark") => return Some(Theme::Dark),
        _ => {}
    }
    parse_color_scheme(&read_setting("org.freedesktop.appearance", "color-scheme")?)
}

/// The portal's `color-scheme` for "prefer dark". Anything else is light: 2
/// is "prefer light" and 0 is no preference, which desktops answer for their
/// light default (pinned by `tests::prefer_dark_picks_dark`,
/// `tests::prefer_light_and_no_preference_pick_light`).
const PREFERS_DARK: u32 = 1;

fn parse_color_scheme(reply: &str) -> Option<Theme> {
    match reply.split_whitespace().last()?.parse::<u32>().ok()? {
        PREFERS_DARK => Some(Theme::Dark),
        _ => Some(Theme::Light),
    }
}

/// Whether the desktop asks for less motion: GNOME's "Reduce animation", or
/// Plasma's animation speed slid to instant.
pub fn reduced_motion() -> bool {
    let gnome = read_setting("org.gnome.desktop.interface", "enable-animations");
    let kde = read_setting("org.kde.kdeglobals.KDE", "AnimationDurationFactor");
    gnome.as_deref().is_some_and(animations_off) || kde.as_deref().is_some_and(animations_off)
}

/// GNOME's "v b false", or Plasma's duration factor "v d 0" (pinned by
/// `tests::reduced_motion_reads_both_desktops`).
fn animations_off(reply: &str) -> bool {
    match reply.split_whitespace().last() {
        Some("false") => true,
        Some(n) => n.parse::<f64>().is_ok_and(|f| f == 0.0),
        None => false,
    }
}

/// The desktop's theme and motion preference, applied. Call again when the
/// window regains focus: both may have changed in the background.
pub fn follow_desktop(ctx: &egui::Context) {
    if let Some(theme) = desktop_preference() {
        ctx.set_theme(theme);
    }
    let animation_time = if reduced_motion() {
        0.0
    } else {
        egui::Style::default().animation_time
    };
    ctx.all_styles_mut(|style| style.animation_time = animation_time);
}

/// The range a desktop's text scale is kept to. An app applies it as egui
/// zoom, as Ctrl + does (DESIGN.md, "Text size counts as width").
pub const TEXT_SCALE: std::ops::RangeInclusive<f32> = 1.0..=3.0;

/// The desktop's text scaling, clamped to `TEXT_SCALE`; None when it doesn't
/// say. Plasma has two knobs, a forced font DPI (kcmfontsrc) and the general
/// font's point size (kdeglobals); GNOME answers through the portal, or
/// gsettings when no portal runs.
pub fn desktop_text_scale() -> Option<f32> {
    let factor = if on_kde() {
        let read = |name: &str| read_kde_config(name).unwrap_or_default();
        kde_text_scale(&read("kcmfontsrc"), &read("kdeglobals"))
    } else {
        gnome_text_scale()
    }?;
    Some(factor.clamp(*TEXT_SCALE.start(), *TEXT_SCALE.end()))
}

/// `desktop_text_scale`, or 1 when the desktop doesn't say.
pub fn text_scale() -> f32 {
    desktop_text_scale().unwrap_or(1.0)
}

fn gnome_text_scale() -> Option<f32> {
    if let Some(reply) = read_setting("org.gnome.desktop.interface", "text-scaling-factor") {
        return parse_scale(&reply);
    }
    let output = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "text-scaling-factor"])
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    parse_scale(&String::from_utf8_lossy(&output.stdout))
}

/// The portal's "v d 1.25", or gsettings' "1.25" (pinned by
/// `tests::the_text_scale_reads_the_portal_and_gsettings`).
fn parse_scale(reply: &str) -> Option<f32> {
    let factor = reply.split_whitespace().last()?.parse::<f32>().ok()?;
    (factor.is_finite() && factor > 0.0).then_some(factor)
}

/// The font DPI that is 1x in Qt.
const QT_BASE_DPI: f32 = 96.0;
/// Breeze's default general font, in points: 1x.
const BREEZE_FONT_POINTS: f32 = 10.0;

/// A forced DPI wins over the font's size, as it does in Qt (pinned by
/// `tests::a_forced_dpi_wins_over_the_font_size`,
/// `tests::a_zero_forced_dpi_falls_back_to_the_font_size`).
fn kde_text_scale(kcmfontsrc: &str, kdeglobals: &str) -> Option<f32> {
    let dpi = ini_value(kcmfontsrc, "[General]", "forceFontDPI")
        .and_then(|dpi| dpi.parse::<f32>().ok())
        .filter(|dpi| *dpi > 0.0);
    if let Some(dpi) = dpi {
        return Some(dpi / QT_BASE_DPI);
    }
    kde_font_scale(&ini_value(kdeglobals, "[General]", "font")?)
}

/// The point size from a Qt font string, "Noto Sans,12,-1,5,50,0,0,0,0,0",
/// over Breeze's (pinned by `tests::the_font_scale_is_the_qt_font_strings_point_size`).
fn kde_font_scale(font: &str) -> Option<f32> {
    let points = font
        .trim_matches('"')
        .split(',')
        .nth(1)?
        .trim()
        .parse::<f32>()
        .ok()?;
    (points > 0.0).then_some(points / BREEZE_FONT_POINTS)
}

/// The `key = value` lines of one `[group]` of a KDE config file, trimmed.
fn ini_entries<'a>(file: &'a str, group: &'a str) -> impl Iterator<Item = (&'a str, &'a str)> {
    let mut inside = false;
    file.lines().map(str::trim).filter_map(move |line| {
        if line.starts_with('[') {
            inside = line == group;
            return None;
        }
        if !inside {
            return None;
        }
        let (key, value) = line.split_once('=')?;
        Some((key.trim(), value.trim()))
    })
}

fn ini_value(file: &str, group: &str, key: &str) -> Option<String> {
    ini_entries(file, group)
        .find(|(name, _)| *name == key)
        .map(|(_, value)| value.to_owned())
}

fn on_kde() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .split(':')
        .any(|name| name.eq_ignore_ascii_case("KDE"))
}

/// Plasma's `name` (`kdeglobals`, `kcmfontsrc`), from the first folder
/// that has it.
fn read_kde_config(name: &str) -> Option<String> {
    let sandboxed = std::path::Path::new("/.flatpak-info").exists();
    config_dirs(
        sandboxed,
        std::env::var_os("XDG_CONFIG_HOME"),
        std::env::var_os("HOME"),
    )
    .iter()
    .find_map(|dir| std::fs::read_to_string(dir.join(name)).ok())
}

/// XDG_CONFIG_HOME, else the home's `.config`. In a Flatpak the home's
/// first: XDG_CONFIG_HOME there is the app's own folder, which Plasma never
/// writes to. The sandbox sees the home's only when it is let
/// (`--filesystem=xdg-config/kdeglobals:ro`, or the home) (pinned by
/// `tests::plasmas_config_is_the_homes_first_in_a_flatpak`).
fn config_dirs(
    sandboxed: bool,
    xdg_config: Option<std::ffi::OsString>,
    home: Option<std::ffi::OsString>,
) -> Vec<std::path::PathBuf> {
    let xdg = xdg_config
        .filter(|dir| !dir.is_empty())
        .map(std::path::PathBuf::from);
    let home = home.map(|home| std::path::Path::new(&home).join(".config"));
    match (sandboxed, xdg, home) {
        (true, xdg, home) => home.into_iter().chain(xdg).collect(),
        (false, Some(xdg), _) => vec![xdg],
        (false, None, home) => home.into_iter().collect(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KdeHeader {
    pub active: Header,
    pub inactive: Header,
}

/// Breeze paints the title bar in the colour scheme's Header colours, so a
/// header painted in them runs into it, as Dolphin's toolbar does. KDE only:
/// elsewhere the file means nothing.
pub fn kde_header() -> Option<KdeHeader> {
    if !on_kde() {
        return None;
    }
    parse_kde_header(&read_kde_config("kdeglobals")?)
}

fn parse_kde_header(kdeglobals: &str) -> Option<KdeHeader> {
    let active = header_colours(kdeglobals, "[Colors:Header]")?;
    let inactive = header_colours(kdeglobals, "[Colors:Header][Inactive]").unwrap_or(active);
    Some(KdeHeader { active, inactive })
}

fn header_colours(kdeglobals: &str, group: &str) -> Option<Header> {
    let (mut bg, mut text, mut muted) = (None, None, None);
    for (key, value) in ini_entries(kdeglobals, group) {
        let slot = match key {
            "BackgroundNormal" => &mut bg,
            "ForegroundNormal" => &mut text,
            "ForegroundInactive" => &mut muted,
            _ => continue,
        };
        *slot = parse_rgb(value);
    }
    Some(Header {
        bg: bg?,
        text: text?,
        muted: muted?,
    })
}

fn parse_rgb(value: &str) -> Option<Color32> {
    let mut parts = value.split(',').map(|part| part.trim().parse::<u8>().ok());
    let color = Color32::from_rgb(parts.next()??, parts.next()??, parts.next()??);
    Some(color)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{DARK, header};

    #[test]
    fn prefer_dark_picks_dark() {
        assert_eq!(parse_color_scheme("v u 1\n"), Some(Theme::Dark));
    }

    #[test]
    fn prefer_light_and_no_preference_pick_light() {
        assert_eq!(parse_color_scheme("v u 2\n"), Some(Theme::Light));
        assert_eq!(parse_color_scheme("v u 0\n"), Some(Theme::Light));
    }

    #[test]
    fn no_answer_picks_no_theme() {
        assert_eq!(parse_color_scheme(""), None);
        assert_eq!(parse_color_scheme("garbage"), None);
    }

    #[test]
    fn gdbus_answers_read_as_busctls_do() {
        assert_eq!(
            parse_color_scheme(&from_gdbus("(<uint32 1>,)\n")),
            Some(Theme::Dark)
        );
        assert_eq!(
            parse_color_scheme(&from_gdbus("(<uint32 2>,)\n")),
            Some(Theme::Light)
        );
        assert_eq!(parse_scale(&from_gdbus("(<1.25>,)\n")), Some(1.25));
        assert!(animations_off(&from_gdbus("(<false>,)\n")));
        assert!(!animations_off(&from_gdbus("(<true>,)\n")));
        assert!(animations_off(&from_gdbus("(<0.0>,)\n")));
        assert_eq!(from_gdbus(""), "");
    }

    #[test]
    fn plasmas_config_is_the_homes_first_in_a_flatpak() {
        let dirs = |sandboxed, xdg: Option<&str>| {
            config_dirs(sandboxed, xdg.map(Into::into), Some("/home/u".into()))
        };
        let path = std::path::PathBuf::from;
        assert_eq!(dirs(false, Some("/c")), [path("/c")]);
        assert_eq!(dirs(false, None), [path("/home/u/.config")]);
        assert_eq!(dirs(false, Some("")), [path("/home/u/.config")]);
        let own = "/home/u/.var/app/a.b/config";
        assert_eq!(dirs(true, Some(own)), [path("/home/u/.config"), path(own)]);
    }

    #[test]
    fn the_text_scale_reads_the_portal_and_gsettings() {
        assert_eq!(parse_scale("v d 1.25\n"), Some(1.25));
        assert_eq!(parse_scale("1.5\n"), Some(1.5));
        assert_eq!(parse_scale("v d nan\n"), None);
        assert_eq!(parse_scale(""), None);
    }

    fn kdeglobals_font(points: &str) -> String {
        format!("[General]\nfont=Noto Sans,{points},-1,5,50,0,0,0,0,0\n")
    }

    #[test]
    fn the_font_scale_is_the_qt_font_strings_point_size() {
        assert_eq!(kde_text_scale("", &kdeglobals_font("10")), Some(1.0));
        assert_eq!(kde_text_scale("", &kdeglobals_font("15")), Some(1.5));
        assert_eq!(kde_text_scale("", &kdeglobals_font("0")), None);
        assert_eq!(kde_text_scale("", "[General]\nfont=Noto Sans\n"), None);
    }

    #[test]
    fn a_forced_dpi_wins_over_the_font_size() {
        assert_eq!(
            kde_text_scale("[General]\nforceFontDPI=144\n", &kdeglobals_font("10")),
            Some(1.5)
        );
    }

    #[test]
    fn a_zero_forced_dpi_falls_back_to_the_font_size() {
        assert_eq!(
            kde_text_scale("[General]\nforceFontDPI=0\n", &kdeglobals_font("12")),
            Some(1.2)
        );
    }

    #[test]
    fn reduced_motion_reads_both_desktops() {
        assert!(animations_off("v b false\n"));
        assert!(!animations_off("v b true\n"));
        assert!(animations_off("v d 0\n"));
        assert!(!animations_off("v d 1\n"));
        assert!(!animations_off(""));
    }

    #[test]
    fn a_key_is_read_from_its_own_group_only() {
        let file = "[Other]\nfont=1\n\n[General]\nother=x\n font = 2 \n[Later]\nfont=3\n";
        assert_eq!(ini_value(file, "[General]", "font"), Some("2".into()));
        assert_eq!(ini_value(file, "[Missing]", "font"), None);
        assert_eq!(ini_value(file, "[General]", "missing"), None);
    }

    #[test]
    fn a_group_is_its_whole_name_as_written() {
        let file = "[General][Sub]\nfont=1\n[general]\nfont=2\n[General]\nfont=3\n";
        assert_eq!(ini_value(file, "[General]", "font"), Some("3".into()));
        assert_eq!(ini_value(file, "[General][Sub]", "font"), Some("1".into()));
        assert_eq!(ini_value(file, "[General]", "Font"), None, "keys too");
        assert_eq!(
            ini_value("font=1\n[General]\n", "[General]", "font"),
            None,
            "a key before any group is in none"
        );
        assert_eq!(
            ini_value(
                "[General]\nfont=1\n[Other]\n[General]\ndpi=2\n",
                "[General]",
                "dpi"
            ),
            Some("2".into()),
            "a group written twice is one group"
        );
    }

    #[test]
    fn a_value_is_everything_after_the_first_equals() {
        let file = "[General]\nfont=Noto Sans;10 #x\nmenu = a=b:c \nempty=\nx:y=1\n";
        assert_eq!(
            ini_value(file, "[General]", "font"),
            Some("Noto Sans;10 #x".into()),
            "; and # are part of a value"
        );
        assert_eq!(ini_value(file, "[General]", "menu"), Some("a=b:c".into()));
        assert_eq!(ini_value(file, "[General]", "empty"), Some(String::new()));
        assert_eq!(ini_value(file, "[General]", "x:y"), Some("1".into()));
        assert_eq!(ini_value(file, "[General]", "x"), None);
    }

    #[test]
    fn a_line_that_is_no_entry_is_skipped_and_the_rest_is_read() {
        let file = "[General]\nno entry\n=5\n# note\nfont=1\n[unclosed\nfont=2\n[General]\ndpi=3\n";
        assert_eq!(ini_value(file, "[General]", "font"), Some("1".into()));
        assert_eq!(ini_value(file, "[General]", "dpi"), Some("3".into()));
        assert_eq!(ini_value(file, "[General]", "no entry"), None);
    }

    const KDEGLOBALS: &str = "\
[Colors:Button]
BackgroundNormal=1,1,1

[Colors:Header]
BackgroundNormal=222,224,226
ForegroundInactive=112,125,138
ForegroundNormal=35,38,41

[Colors:Header][Inactive]
BackgroundNormal=239,240,241
ForegroundInactive=112,125,138
ForegroundNormal=35,38,41
";

    #[test]
    fn the_header_colours_come_from_their_own_group() {
        let kde = parse_kde_header(KDEGLOBALS).unwrap();
        assert_eq!(kde.active.bg, Color32::from_rgb(222, 224, 226));
        assert_eq!(kde.active.text, Color32::from_rgb(35, 38, 41));
        assert_eq!(kde.active.muted, Color32::from_rgb(112, 125, 138));
        assert_eq!(kde.inactive.bg, Color32::from_rgb(239, 240, 241));
    }

    #[test]
    fn a_scheme_without_inactive_colours_reuses_the_active_ones() {
        let only_active = KDEGLOBALS
            .split("[Colors:Header][Inactive]")
            .next()
            .unwrap();
        let kde = parse_kde_header(only_active).unwrap();
        assert_eq!(kde.inactive, kde.active);
    }

    #[test]
    fn no_header_group_means_the_surface_colours() {
        assert_eq!(
            parse_kde_header("[Colors:Button]\nBackgroundNormal=1,1,1\n"),
            None
        );
        assert_eq!(
            parse_kde_header("[Colors:Header]\nBackgroundNormal=oops\n"),
            None
        );
        let ctx = egui::Context::default();
        ctx.set_theme(Theme::Dark);
        assert_eq!(header(&ctx, None, true).bg, DARK.surface);
    }
}
