//! The vinoPilot apps and where each one lives on vinopilot.ca, so every
//! app's About page links to the same places with the same words. No egui
//! here: the tray processes use it too, for their own app's repository.

use std::io;
use std::process::{Command, Stdio};

/// The home page of the whole family.
pub const SITE: &str = "https://vinopilot.ca/";

/// How a link to the site reads: the domain, which is what the user would
/// type.
pub const SITE_LABEL: &str = "vinopilot.ca";

/// One vinoPilot app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum App {
    Pilot,
    Whisper,
    AuthFace,
    OpenEffects,
    Pop,
    Speak,
    Search,
    Watch,
}

impl App {
    /// Every app, in the order vinopilot.ca lists them.
    pub const ALL: [App; 8] = [
        App::Pilot,
        App::Whisper,
        App::AuthFace,
        App::OpenEffects,
        App::Pop,
        App::Speak,
        App::Search,
        App::Watch,
    ];

    pub fn name(self) -> &'static str {
        match self {
            App::Pilot => "vinoPilot",
            App::Whisper => "vinoWhisper",
            App::AuthFace => "vinoAuthFace",
            App::OpenEffects => "vinoOpenEffects",
            App::Pop => "vinoPop",
            App::Speak => "vinoSpeak",
            App::Search => "vinoSearch",
            App::Watch => "vinoWatch",
        }
    }

    /// What it does, in the site's words: one line, sentence case, no full
    /// stop.
    pub fn blurb(self) -> &'static str {
        match self {
            App::Pilot => "Install and update vinoPilot apps",
            App::Whisper => "Live captions and voice typing",
            App::AuthFace => "Unlock with your face",
            App::OpenEffects => "Webcam effects for video calls",
            App::Pop => "AI actions for selected text",
            App::Speak => "Natural-sounding text-to-speech",
            App::Search => "Search local files by meaning",
            App::Watch => "Monitor NPU usage and activity",
        }
    }

    /// Its section of vinopilot.ca/apps.
    pub fn page(self) -> &'static str {
        match self {
            App::Pilot => "https://vinopilot.ca/apps/#vinopilot",
            App::Whisper => "https://vinopilot.ca/apps/#vinowhisper",
            App::AuthFace => "https://vinopilot.ca/apps/#vinoauthface",
            App::OpenEffects => "https://vinopilot.ca/apps/#vinoopeneffects",
            App::Pop => "https://vinopilot.ca/apps/#vinopop",
            App::Speak => "https://vinopilot.ca/apps/#vinospeak",
            App::Search => "https://vinopilot.ca/apps/#vinosearch",
            App::Watch => "https://vinopilot.ca/apps/#vinowatch",
        }
    }

    /// Its GitHub repository, `owner/name`: where its releases are.
    pub fn repo(self) -> &'static str {
        match self {
            App::Pilot => "karanshukla/vinoPilot",
            App::Whisper => "karanshukla/vinoWhisper",
            App::AuthFace => "karanshukla/vinoAuthFace",
            App::OpenEffects => "karanshukla/vinoOpenEffects",
            App::Pop => "karanshukla/vinoPop",
            App::Speak => "karanshukla/vinoSpeak",
            App::Search => "karanshukla/vinoSearch",
            App::Watch => "karanshukla/vinoWatch",
        }
    }

    /// Every app but this one, in the site's order.
    pub fn others(self) -> impl Iterator<Item = App> {
        App::ALL.into_iter().filter(move |app| *app != self)
    }
}

/// An `http(s)://` URL with no spaces or control characters, so `xdg-open`
/// can't take it as a flag, a local file or a second argument.
fn is_valid_http_url(url: &str) -> bool {
    (url.starts_with("https://") || url.starts_with("http://"))
        && url.bytes().all(|b| b > 0x20 && b != 0x7f)
}

/// Opens `url` in the default browser with `xdg-open`, which goes through
/// the OpenURI portal inside a Flatpak. Returns once it has started; a
/// thread reaps it so it doesn't linger as a zombie. Anything but an
/// `http(s)://` URL is refused with `InvalidInput`.
pub fn open(url: &str) -> io::Result<()> {
    if !is_valid_http_url(url) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid URL"));
    }
    let mut child = Command::new("xdg-open")
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .spawn()?;
    std::thread::spawn(move || child.wait());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_rejects_non_http_or_control_char_urls() {
        assert!(is_valid_http_url(
            "https://github.com/karanshukla/vinoPilot"
        ));
        assert!(is_valid_http_url("http://example.com/a"));
        for bad in [
            "",
            "--help",
            "-v",
            "file:///etc/passwd",
            "javascript:alert(1)",
            "https://example.com/ space",
            "https://example.com/\nnewline",
            "https://example.com/\x7fdel",
        ] {
            assert!(!is_valid_http_url(bad), "{bad}");
            let err = open(bad).unwrap_err();
            assert_eq!(err.kind(), io::ErrorKind::InvalidInput, "{bad}");
        }
    }

    #[test]
    fn every_app_has_its_own_name_and_page_on_the_site() {
        let mut names = std::collections::HashSet::new();
        let mut pages = std::collections::HashSet::new();
        for app in App::ALL {
            assert!(names.insert(app.name()));
            assert!(pages.insert(app.page()));
            assert!(app.page().starts_with(SITE), "{}", app.page());
            assert_eq!(app.repo(), format!("karanshukla/{}", app.name()));
            assert_eq!(
                app.page().rsplit_once('#').map(|(_, anchor)| anchor),
                Some(app.name().to_lowercase().as_str())
            );
        }
    }

    #[test]
    fn blurbs_are_one_short_line() {
        for app in App::ALL {
            let blurb = app.blurb();
            assert!(!blurb.is_empty() && blurb.len() <= 40, "{blurb}");
            assert!(!blurb.ends_with('.') && !blurb.contains('\n'), "{blurb}");
            assert!(blurb.starts_with(char::is_uppercase), "{blurb}");
        }
    }

    #[test]
    fn others_leaves_out_only_this_app() {
        for app in App::ALL {
            let others: Vec<App> = app.others().collect();
            assert_eq!(others.len(), App::ALL.len() - 1);
            assert!(!others.contains(&app));
        }
    }
}
