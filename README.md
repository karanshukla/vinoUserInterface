# vinoUserInterface

The egui theme and widgets shared by [vinoSearch](https://github.com/karanshukla/vinoSearch),
[vinoWhisper](https://github.com/karanshukla/vinoWhisper) and
[vinoSpeak](https://github.com/karanshukla/vinoSpeak): one look, built on
vinopilot.ca's site.css. Inter, flat surfaces separated by hairlines, ink and
green. The boards it implements are on the
[vino UI canvas](https://claude.ai/code/artifact/f79675d7-67e0-4a99-a84d-8a22d3b3a8f6).

```toml
[dependencies]
vino-user-interface = "0.1"
```

```rust
use vino_user_interface::{theme, widgets};
```

## Releasing

Bump `version` in `Cargo.toml`, merge, then push a matching tag:
`git tag v0.1.1 && git push origin v0.1.1`. The release workflow reruns the
checks and publishes to crates.io. Then bump each app.

## What is in it

| Module | Holds |
|---|---|
| `theme` | the tokens for dark and light, fonts, egui's visuals and spacing, and what the desktop asks for: light or dark (the Settings portal), text size, reduced motion, KDE's header colours |
| `widgets` | group + row, switch, segmented control, the four button kinds, status chip, tag, key caps, progress and share bars, banner, notice, nav item, underline tab, selection ring and tint, a 10 fps spinner, the page scroll area |
| `icons` | a 16 px stroke icon set painted from SVG path data (no icon font) |
| `logo` | the three app logos, painted at any size |
| `a11y` | headings, tabs, list boxes, live regions and naming for screen readers |

Colour has one job each: green means on, selected or done; amber means it
needs you; red means it failed. Menus and popups are the only shadowed surface.

## Rules the code keeps

- Text is 4.5:1 and the edges of controls 3:1 against the page, a surface, a
  field and the three tints, in both themes. `cargo test` checks it over the
  visuals as installed; a token that fails it does not ship.
- Every control has a role and a name, is 24 px or has room around it, and
  Tab reaches it. `tests/a11y.rs` checks that over the AccessKit tree, in both
  themes, wide and narrow; CI runs it as its own `a11y` check.
- A row lays its controls out from the right. A composite widget inside one
  sizes itself up front and lays out left to right (see `segmented`, `keys`).
- `widgets::switch` and the buttons are their own widgets, not styled egui
  ones, so they can say what they are to a screen reader: a switch is a
  `Switch` with the row's title for its name.
- The bundled fonts are Latin subsets (`assets/fonts/README.md`). egui falls
  back to its own fonts for other glyphs, and a glyph no font has panics in a
  test, so check before putting a symbol in a window.

## Checking a window by eye

`VINO_UI_THEME=light|dark` forces the theme on a desktop without a portal.
`scripts/shot.sh OUT THEME W H CMD...` runs a window as a floating `W x H`
window on a sway desktop (the linuxlab VM's headless one) and screenshots it
with grim.
