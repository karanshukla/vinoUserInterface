//! vino UI: the one look of the vinoPilot egui apps (vinoSearch, vinoWhisper,
//! vinoSpeak, vinoAuthFace), built on vinopilot.ca's site.css: Inter, flat surfaces
//! separated by hairlines, ink and green. One copy here replaces the
//! `theme.rs` each app carried (vinoSearch#148).
//!
//! - [`theme`]: the colour tokens for dark and light, fonts, egui's visuals
//!   and spacing, and what the desktop asks for (light or dark, text size,
//!   reduced motion, KDE's header colours).
//! - [`widgets`]: the parts every window is made of: groups of rows, switches,
//!   segmented controls, buttons, status text, filter chips, tags, key caps,
//!   banners, nav items and tabs, and a spinner for real waits that stands
//!   still when motion is reduced.
//! - [`icons`]: a 16 px stroke icon set painted from SVG paths.
//! - [`logo`]: the three app logos, painted so they stay sharp at any scale.
//! - [`file_tile`]: a file's tile for a list, drawn like the logos, with an
//!   accent that says which format it is.
//! - [`a11y`]: names, roles and headings for screen readers, which egui's
//!   painted widgets don't give by themselves.
//! - [`about`]: the About page each window ends with, linking to
//!   vinopilot.ca and the other apps.
//!
//! Two modules need no egui, for the tray processes, which build with
//! `default-features = false`:
//!
//! - [`family`]: the vinoPilot apps, their one-line blurbs and their pages
//!   on vinopilot.ca.
//! - [`tray`]: the icon theme name for each kind of tray menu item, so every
//!   tray uses the same ones.

#![forbid(unsafe_code)]

#[cfg(feature = "egui")]
pub use egui;

pub mod family;
pub mod tray;

#[cfg(feature = "egui")]
pub mod a11y;
#[cfg(feature = "egui")]
pub mod about;
#[cfg(feature = "egui")]
pub mod file_tile;
#[cfg(feature = "egui")]
pub mod icons;
#[cfg(feature = "egui")]
pub mod logo;
#[cfg(feature = "egui")]
pub mod theme;
#[cfg(feature = "egui")]
pub mod widgets;

pub use family::App;
#[cfg(feature = "egui")]
pub use file_tile::Format;
#[cfg(feature = "egui")]
pub use icons::Icon;
#[cfg(feature = "egui")]
pub use theme::{Palette, palette};
pub use tray::MenuIcon;
#[cfg(feature = "egui")]
pub use widgets::{Button, ButtonKind, ChipKind, Row, Tone};
