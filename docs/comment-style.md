# Comment style

A comment is the last resort. Work down this ladder, and write prose only
when all four rungs fail.

1. **Extract it.** Logic that a comment explains belongs in a function with
   a domain name. Arithmetic spelled out in a comment belongs in named
   constants. A coercion that is explained twice belongs in one named helper.

   ```rust
   // Before
   // Sized up front: a row lays its controls out from the right.
   let total = galleys.iter().map(|g| g.size().x + 20.0).sum::<f32>()
       + 2.0 * options.len().saturating_sub(1) as f32;

   // After
   let total = segments_width(&galleys);
   ```

2. **Name it.** A comment that labels a block (`// Menus and popups: ...`)
   means the block wants to be a function (`popups(&mut v, p)`). Magic
   numbers get names too: `KITTEST_MARGIN`, not `+ 16.0 // kittest puts 8 px
   round the app`.

3. **Pin it with a test pair.** A comment that states a rule is a rule
   nothing enforces. Write one test inside the boundary and one outside,
   named after the rule (`a_failure_line_of_the_limit_is_kept_whole` /
   `a_longer_failure_line_is_cut_with_an_ellipsis`), instead of writing
   `// cut to 160 characters`. The pair is the point: it fails the day
   someone moves the limit. In a test, a comment that labels the next few
   asserts is the same smell. Split those asserts into their own test, named
   after what the comment said.

4. **Pin cross-boundary rules with an end-to-end test.** How the title bar
   answers a pointer, what the AccessKit tree says, what a window looks
   like: `tests/window.rs`, `tests/a11y.rs` and `tests/screenshots.rs`, not a
   paragraph.

5. **Only then, prose.** Hidden constraints (egui's `Frame` puts its stroke
   outside the margin), upstream gaps (Breeze has no small software-update
   icon), protocol requirements. State the constraint, not its biography.
   Measured claims carry a date.

## Linking the test from the code

When rung 3 or 4 replaced a comment, the code it governs says which test
pins the rule, in its doc comment, always with the same words:

```rust
/// Cut to one line of at most `FAILURE_LINE_MAX` characters (pinned by
/// `tests::a_longer_failure_line_is_cut_with_an_ellipsis`).
```

The forms are `` `tests::name` `` (the file's own tests, or `tests.rs` beside
a `mod.rs`), `` `module::tests::name` `` (`widgets::tests::name` is
`src/widgets/tests.rs`), and `` `tests/file.rs::name` `` for an integration
test.

rustdoc can't link to `#[test]` functions (they are compiled out even
under `--cfg test`), so these are code spans, not intra-doc links.
`tests/doc_links.rs` (CI job "Doc Links") checks them instead: every such
span in a comment must name a `#[test]` fn in that file.

## Exempt

- `///` and `//!` docs on items, which say what an item is and how to call
  it. They follow the same "constraint, not biography" rule, but aren't
  replaced by code. Every `pub` item's name and signature stay as they are
  (see [development.md](development.md#public-api)); its doc can be tightened.
- `// SAFETY:` comments on `unsafe`, `#[allow]` reasons, and coverage
  pragmas.
