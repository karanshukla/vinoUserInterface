# Working on vino UI

What the tests keep, how the source is organised, and how a release goes out.
How a window is put together from the parts is in [DESIGN.md](../DESIGN.md).

## Public API

Every `pub` item reachable from the crate root is used by another repository:
the settings windows of vinoSearch, vinoWhisper, vinoSpeak, vinoAuthFace and
vinoPilot, and the trays, which build with `default-features = false`. Renaming
or removing one is a breaking release and a change in each app. Deprecate
first, as `widgets::chip_busy` was.

The same goes for how things render. A token, a size or a spacing that changes
here changes five windows.

## Rules the tests keep

| Rule | Where it is checked |
|---|---|
| Text is 4.5:1 and the edges of controls 3:1 against the page, a surface, a field and the three tints, in both themes | `cargo test --lib contrast`, over the visuals as installed |
| Every control has a role and a name, is 24 px or has room around it, and Tab reaches it | `tests/a11y.rs`, over the AccessKit tree, both themes, wide and narrow |
| A row's control goes under its label only in a row under `NARROW`, and not when it is small enough to leave the label 160 | `a_row_stacks_under_narrow_and_not_at_it`, `a_narrow_row_keeps_a_control_that_leaves_the_label_its_minimum` (`widgets/rows.rs`) |
| `SIDEBAR_FROM` and `TWO_COLUMNS_FROM` leave a row exactly `NARROW` wide | `the_sidebar_breakpoint_leaves_a_row_exactly_narrow`, `columns_split_only_where_both_keep_wide_rows` |
| A stacked row keeps its controls in the order they have beside the label | `a_stacked_row_keeps_the_order_it_has_beside_the_label` |
| A Plasma config key is read from its own group only | `theme::desktop::tests::a_key_is_read_from_its_own_group_only` |
| A Plasma config line that is no entry is skipped, and the rest of the file is still read | `theme::desktop::tests::a_line_that_is_no_entry_is_skipped_and_the_rest_is_read` |
| An update tag is `v`, digits, letters, dots and dashes, or it is refused | `update::tests::only_a_plausible_tag_is_taken` |
| The app's own title bar buttons, drag area and resize edges | `tests/window.rs` |

CI runs the contrast and AccessKit checks as their own `a11y` job, so a
regression there reads as one.

Three rules have no test:

- A row lays its controls out from the right. A composite widget inside one
  sizes itself up front and lays out left to right (`segmented`, `keys`).
- `widgets::switch` and the buttons are their own widgets, not styled egui
  ones, so they can say what they are to a screen reader: a switch is a
  `Switch` with the row's title for its name.
- The bundled fonts are Latin subsets
  ([assets/fonts/README.md](../assets/fonts/README.md)). egui falls back to
  its own fonts for other glyphs, and a glyph no font has panics in a test, so
  check before putting a symbol in a window.

## Where things live

```
src/
  lib.rs          the module list and the root re-exports
  theme/
    mod.rs        tokens for dark and light, fonts, egui's visuals and spacing
    desktop.rs    what the desktop asks for: the Settings portal, Plasma's files
  widgets/
    mod.rs        the layout constants and what every widget shares
    text.rs       titles, labels, body text
    rows.rs       group, Row and how a row lays itself out
    choices.rs    switch, segmented control, filter chip, combo box, fields
    buttons.rs    the four kinds of button
    status.rs     status text, tags, key caps, bars, the spinner
    banners.rs    banner and notice
    navigation.rs sidebar entries, tabs, the header's and the sidebar's frames
    lists.rs      a selected row's and tile's look
    page.rs       the page scroll area, columns, the fluid sidebar and inset
    tests.rs      the widgets' tests, over the AccessKit tree
  icons.rs        the stroke icons and the SVG path painter
  logo.rs         the app logos
  file_tile.rs    a file's tile, per format
  a11y.rs         roles, names, headings and live regions
  input.rs        touchpad scrolling
  window.rs       the app's own title bar
  about.rs        the About page
  family.rs       the apps and their links           (no egui)
  tray.rs         tray menu icon names               (no egui)
  update.rs       the update check and install       (no egui)
```

The files under `widgets/` and `theme/` are private. `widgets::Button` and
`theme::follow_desktop` are still the paths: each `mod.rs` re-exports what its
files hold.

Three runtime dependencies, and few on purpose
([SECURITY.md](../SECURITY.md)): egui and svgtypes (the icons' path data)
behind the `egui` feature, and serde_json (GitHub's release JSON), which the
trays link too. The version and INI reading here is a few lines each, written
by hand. configparser 3.3.0 was tried for the INI (2026-11-15): it refuses a
whole file for one line with no key (`=5`) or an unclosed `[group`, where
`ini_entries` skips the line and reads the rest
(`a_line_that_is_no_entry_is_skipped_and_the_rest_is_read`).

## Comments

A comment is the last resort: extract a function, name a constant, or pin a
rule with a test pair first, and cite the test from the code it governs as
``(pinned by `tests::name`)``. See [comment-style.md](comment-style.md);
`tests/doc_links.rs` (CI job Doc Links) fails on a citation with no such
`#[test]`.

## The gate

What CI runs (`.github/workflows/ci.yml`):

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked                                                        # includes --test doc_links
cargo clippy --locked --no-default-features --all-targets -- -D warnings   # what the trays build
cargo test --locked --no-default-features --lib
cargo publish --dry-run --locked
cargo +1.95 check --locked --all-targets                                   # rust-version
cargo deny check                                                           # not a required check
```

## Checking a window by eye

The README's pictures are the gallery rendered offscreen with wgpu. After a
visual change, redraw them and commit `docs/images`:

```sh
cargo test --test screenshots -- --ignored
```

After a change that should not be visible, run the same thing and expect
`git status` to show nothing under `docs/images`. That only works on the
machine that drew the committed ones. On Mesa's software renderer two runs
matched each other byte for byte and neither matched the committed pictures
(2026-11-15, a cloud container with no GPU), so there, render the commit
before the change into a second checkout and compare against that.

`VINO_UI_THEME=light|dark` forces the theme on a desktop without a portal.
`scripts/shot.sh OUT THEME W H CMD...` runs a window as a floating `W x H`
window on a sway desktop (the linuxlab VM's headless one) and screenshots it
with grim.

## Releasing

Bump `version` in `Cargo.toml`, merge, then push a matching tag:

```sh
git tag -a v0.2.2 -m "vino-user-interface 0.2.2" && git push origin v0.2.2
```

The release workflow reruns the checks, publishes to crates.io, then creates
the GitHub Release with notes GitHub generates from the PRs merged since the
last tag, so PR titles are the changelog. Then bump each app.
