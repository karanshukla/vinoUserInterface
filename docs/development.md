# Development

Test invariants, source layout, visual verification, and the release workflow.
For UI layout and component guidelines, see [DESIGN.md](../DESIGN.md).

## Public API

Every `pub` item reachable from the crate root is consumed by downstream
vinoPilot repositories, including the settings windows of vinoSearch,
vinoWhisper, vinoSpeak, vinoAuthFace, and vinoPilot, as well as tray processes
built with `default-features = false`. Renaming or removing a public item is a
breaking change that requires updates across those apps. Deprecate items first,
as was done for `widgets::chip_busy`.

Visual tokens, sizes, and spacing have the same reach: changing a token here
affects every downstream window.

## Test invariants

| Invariant | Verified by |
|---|---|
| Text contrast is at least 4.5:1 and control edges at least 3:1 against page, surface, field, and tint backgrounds in both themes | `cargo test --lib contrast`, checked against installed visuals |
| Every control has a role and accessible name, is at least 24 px (or adequately spaced), and is reachable via Tab | `tests/a11y.rs` over the AccessKit tree in both themes, wide and narrow |
| A row stacks its control below its label only below `NARROW`, and keeps small controls beside the label when at least 160 px remains | `a_row_stacks_under_narrow_and_not_at_it`, `a_narrow_row_keeps_a_control_that_leaves_the_label_its_minimum` (`widgets/rows.rs`) |
| `SIDEBAR_FROM` and `TWO_COLUMNS_FROM` leave each row at least `NARROW` wide | `the_sidebar_breakpoint_leaves_a_row_exactly_narrow`, `columns_split_only_where_both_keep_wide_rows` |
| Stacked rows keep controls in the same order as horizontal rows | `a_stacked_row_keeps_the_order_it_has_beside_the_label` |
| Plasma config keys are read only from their expected group | `theme::desktop::tests::a_key_is_read_from_its_own_group_only` |
| Invalid Plasma config lines are skipped without aborting the rest of the file | `theme::desktop::tests::a_line_that_is_no_entry_is_skipped_and_the_rest_is_read` |
| Update tags must contain only `v`, ASCII alphanumerics, dots, and dashes | `update::tests::only_a_plausible_tag_is_taken` |
| Client-side title bar buttons, drag region, and resize edges | `tests/window.rs` |

CI runs the contrast and AccessKit checks in a dedicated `a11y` job so
accessibility regressions stand out clearly.

Three conventions are enforced by review rather than automated tests:

- Rows lay out controls from right to left. Composite widgets placed inside a
  row (`segmented`, `keys`) compute their total width up front and lay out their
  internal items left to right.
- `widgets::switch` and `Button` are custom widgets rather than restyled egui
  defaults so they expose accurate screen-reader roles and names (for example,
  a switch reports the `Switch` role with the row's title as its label).
- Bundled fonts are Latin subsets
  ([assets/fonts/README.md](../assets/fonts/README.md)). egui falls back to its
  built-in fonts for other glyphs, and any glyph missing from all fonts panics
  in tests, so verify non-Latin symbols before adding them.

## Source layout

```
src/
  lib.rs          module list and root re-exports
  theme/
    mod.rs        dark/light tokens, bundled fonts, egui visuals and spacing
    desktop.rs    desktop preference detection (Settings portal, Plasma config files)
  widgets/
    mod.rs        layout constants and shared widget helpers
    text.rs       titles, labels, and body text
    rows.rs       group, Row, and responsive row layout
    choices.rs    switch, segmented control, filter chip, combo box, text fields
    buttons.rs    primary, secondary, ghost, and destructive buttons
    status.rs     status text, tags, key caps, progress/share bars, spinner
    banners.rs    banner and notice widgets
    navigation.rs sidebar entries, tabs, header and sidebar frames
    lists.rs      selected row and tile styling
    page.rs       page scroll area, multi-column layout, fluid sidebar and inset
    tests.rs      widget unit tests over the AccessKit tree
  icons.rs        stroke icons and SVG path painter
  logo.rs         vector app logos
  file_tile.rs    format-specific file tiles
  a11y.rs         accessibility roles, names, headings, and live regions
  input.rs        touchpad scrolling helpers
  window.rs       client-side title bar and resize handles
  about.rs        shared About page
  family.rs       vinoPilot app catalog and links    (no egui)
  tray.rs         tray menu symbolic icon names      (no egui)
  update.rs       update check and install runner    (no egui)
```

Files under `widgets/` and `theme/` are private modules re-exported through
their `mod.rs` files (`widgets::Button`, `theme::follow_desktop`, etc.).

Runtime dependencies are intentionally minimal ([SECURITY.md](../SECURITY.md)):
`egui` and `svgtypes` behind the `egui` feature, plus `serde_json` for GitHub
release JSON (also used by tray builds). Version comparison and INI parsing are
implemented directly in a few lines of code. `configparser` 3.3.0 was evaluated
for INI parsing (2026-11-15), but it rejects an entire file on a single
malformed line (`=5` or an unclosed `[group`), whereas `ini_entries` skips
invalid lines and parses the rest
(`a_line_that_is_no_entry_is_skipped_and_the_rest_is_read`).

## Comments

Prefer extracting a function, naming a constant, or writing a test before adding
an inline comment. When a comment documents an invariant, cite its test as
``(pinned by `tests::name`)``. See [comment-style.md](comment-style.md);
`tests/doc_links.rs` (CI job Doc Links) fails if a cited `#[test]` does not
exist.

## CI checks

Commands run by `.github/workflows/ci.yml`:

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked                                                        # includes --test doc_links
cargo clippy --locked --no-default-features --all-targets -- -D warnings   # tray builds
cargo test --locked --no-default-features --lib
cargo publish --dry-run --locked
cargo +1.95 check --locked --all-targets                                   # rust-version
cargo deny check                                                           # non-blocking check
```

## Visual verification

The screenshots in `README.md` are rendered offscreen from the gallery example
using wgpu. After making a visual change, regenerate and commit `docs/images`:

```sh
cargo test --test screenshots -- --ignored
```

After a change that should not affect appearance, rerun the same command and
verify that `git status` shows no changes under `docs/images`. Byte-for-byte
comparison against committed PNGs only works on the same renderer that produced
them. On Mesa's software renderer, two consecutive runs match each other byte
for byte, but neither matches GPU-rendered baselines (verified 2026-11-15 in a
cloud container without a GPU); in that environment, render the base commit in a
separate checkout and compare against it.

Set `VINO_UI_THEME=light` or `dark` to override the theme on desktops without a
Settings portal. `scripts/shot.sh OUT THEME W H CMD...` launches a floating
`W x H` window under headless sway (such as the linuxlab VM) and captures a
screenshot with `grim`.

## Releasing

Update `version` in `Cargo.toml`, merge to `main`, and push a matching tag:

```sh
git tag -a v0.2.2 -m "vino-user-interface 0.2.2" && git push origin v0.2.2
```

The release workflow reruns all checks, publishes to crates.io, and creates a
GitHub Release with auto-generated notes from merged PR titles since the
previous tag. After publishing, update the dependency version in downstream
apps.
