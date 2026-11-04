//! Icons for tray menu items: one icon per meaning, so Settings, Quit or a
//! link look the same in every vinoPilot tray. No egui here: the trays
//! don't link it.
//!
//! A tray menu is drawn by the desktop, not the app, so these are names from
//! the desktop's icon theme rather than the painted [`crate::icons`]. Each
//! is a `-symbolic` name that both Breeze (KDE) and Adwaita (GNOME) ship:
//! the host recolours a symbolic icon to the menu's text, light or dark, the
//! way the painted set follows the window's theme. Breeze draws the status
//! icons (information, warning) in colour only; a host falls back to those
//! by dropping the suffix.
//!
//! Which items get one (DESIGN.md, "Tray menus"): every action and submenu
//! at the top level; not a checkmark or radio item, whose mark is its icon;
//! not the entries of a list of like things (recent dictations, the other
//! apps). A row of plain text gets one only when it carries a state.

/// What a menu item is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MenuIcon {
    /// Opens the settings window, or a submenu of settings.
    Settings,
    Quit,
    /// Opens the documentation.
    Help,
    /// What's new, the version, release notes.
    About,
    /// A link to vinopilot.ca or another web page that isn't help.
    Website,
    /// The other vinoPilot apps.
    Apps,
    /// Do it again: retrain, reset, reload.
    Refresh,
    /// Look for something: check a driver, find a device.
    Search,
    Info,
    Warning,
    /// Add something new: enrol a face.
    Add,
    /// Delete or uninstall.
    Remove,
    Cancel,
    Camera,
    /// Users and the login screen.
    Users,
    /// Download and install an update.
    Update,
    Copy,
    /// Empty a list.
    Clear,
    /// A list of recent things.
    Recent,
    /// Show a popup or a toolbar.
    Popup,
    /// A shortcut.
    Keyboard,
    Microphone,
    /// What is playing: system audio, a speaker.
    Audio,
    Plugins,
}

impl MenuIcon {
    pub const ALL: [MenuIcon; 24] = [
        MenuIcon::Settings,
        MenuIcon::Quit,
        MenuIcon::Help,
        MenuIcon::About,
        MenuIcon::Website,
        MenuIcon::Apps,
        MenuIcon::Refresh,
        MenuIcon::Search,
        MenuIcon::Info,
        MenuIcon::Warning,
        MenuIcon::Add,
        MenuIcon::Remove,
        MenuIcon::Cancel,
        MenuIcon::Camera,
        MenuIcon::Users,
        MenuIcon::Update,
        MenuIcon::Copy,
        MenuIcon::Clear,
        MenuIcon::Recent,
        MenuIcon::Popup,
        MenuIcon::Keyboard,
        MenuIcon::Microphone,
        MenuIcon::Audio,
        MenuIcon::Plugins,
    ];

    /// The icon theme name, for ksni's `icon_name` or Qt's
    /// `QIcon.fromTheme`.
    pub fn name(self) -> &'static str {
        match self {
            MenuIcon::Settings => "preferences-system-symbolic",
            MenuIcon::Quit => "application-exit-symbolic",
            MenuIcon::Help => "help-contents-symbolic",
            MenuIcon::About => "help-about-symbolic",
            MenuIcon::Website => "insert-link-symbolic",
            MenuIcon::Apps => "view-app-grid-symbolic",
            MenuIcon::Refresh => "view-refresh-symbolic",
            MenuIcon::Search => "system-search-symbolic",
            MenuIcon::Info => "dialog-information-symbolic",
            MenuIcon::Warning => "dialog-warning-symbolic",
            MenuIcon::Add => "list-add-symbolic",
            MenuIcon::Remove => "edit-delete-symbolic",
            MenuIcon::Cancel => "window-close-symbolic",
            MenuIcon::Camera => "camera-web-symbolic",
            MenuIcon::Users => "system-users-symbolic",
            // Breeze has no small software-update icon.
            MenuIcon::Update => "folder-download-symbolic",
            MenuIcon::Copy => "edit-copy-symbolic",
            MenuIcon::Clear => "edit-clear-all-symbolic",
            MenuIcon::Recent => "document-open-recent-symbolic",
            MenuIcon::Popup => "open-menu-symbolic",
            MenuIcon::Keyboard => "input-keyboard-symbolic",
            MenuIcon::Microphone => "audio-input-microphone-symbolic",
            MenuIcon::Audio => "audio-speakers-symbolic",
            // Breeze's "plugins" and Adwaita's "application-x-addon" have no
            // name in common.
            MenuIcon::Plugins => "preferences-other-symbolic",
        }
    }
}

/// The label of the submenu that links to vinopilot.ca and the other apps.
pub const APPS_MENU: &str = "vinoPilot apps";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_meaning_has_its_own_symbolic_icon() {
        let mut seen = std::collections::HashSet::new();
        for icon in MenuIcon::ALL {
            let name = icon.name();
            assert!(name.ends_with("-symbolic"), "{name}");
            assert!(seen.insert(name), "{name} is used twice");
        }
    }
}
