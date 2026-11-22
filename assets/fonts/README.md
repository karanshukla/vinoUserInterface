`Inter-{Regular,Medium,SemiBold,Bold}.ttf` are static instances of
vinowhisper-landing's `inter-latin.woff2` (Inter 4, Latin subset) at weights
400, 500, 600, and 700, generated with `fontTools.varLib.instancer`. Regular
and Bold replace the duplicate copies that downstream apps previously bundled
individually.

`JetBrainsMono-Regular.ttf` is JetBrains Mono 2.304 Regular, subset to the same
Latin range with `pyftsubset`. Monospace is used for file paths and commands
only; all other UI text uses Inter.

Neither subset includes arrows or other symbols outside the Latin range. egui
falls back to its built-in fonts for missing glyphs, and a glyph missing from
all loaded fonts will panic in tests, so verify symbols before using them in UI
text. `"…"`, `"•"`, `"·"`, and `"×"` are safe to use.
