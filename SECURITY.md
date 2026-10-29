# Security

vino UI is a theme and widget library. It makes no network calls and writes
nothing. To follow the desktop's light/dark, reduced-motion and text-size
settings it reads, as the user running the app:

- the portal's Settings over D-Bus, by running `busctl --user --timeout=1`,
  and `gsettings` when no portal answers (both found on `PATH`);
- `kdeglobals` and `kcmfontsrc` in `$XDG_CONFIG_HOME` (or `~/.config`) on KDE;
- `VINO_UI_THEME`, `XDG_CURRENT_DESKTOP`, `XDG_CONFIG_HOME` and `HOME`.

Its main risk is the supply chain: every vino app (vinoSearch, vinoWhisper,
vinoSpeak) compiles it in, so a bad release reaches all of them at once.

## Reporting

Use [private vulnerability reporting](https://github.com/karanshukla/vinoUserInterface/security/advisories/new).
Don't open a public issue for anything exploitable. If it affects an app too,
say which; the fix is a release here, then a version bump in each app.

## What's in place

| Threat | Defence |
|---|---|
| Memory-safety bug in this crate | `#![forbid(unsafe_code)]`: there is no `unsafe` here, and the compiler refuses any. |
| Odd or hostile desktop settings | Whatever busctl, gsettings or the KDE files say is parsed, never executed; anything that doesn't parse is ignored, and the text scale is clamped to a fixed range. A same-user attacker who can change `PATH` or `~/.config` already controls the app, so that isn't a boundary. |
| Compromised or yanked dependency | One runtime dependency, egui. cargo-deny checks advisories, licences and sources for what ships (`deny.toml`, crates.io only, dev-dependencies excluded since a user never gets them). `Cargo.lock` is enforced in CI (`--locked`). |
| Freshly compromised release pulled in | Dependabot waits 7 days before proposing an update (security fixes skip the wait), and nothing auto-merges. |
| Compromised CI action | Every action is pinned to a commit SHA; workflows get a read-only token and don't keep checkout credentials. |
| A release that isn't what's in the repo | Releases publish only from a `v*` tag, through crates.io trusted publishing: a short-lived token minted for that run, no stored secret. The workflow reruns every check and refuses a tag that doesn't match `Cargo.toml`'s version. |
| Unreviewed change on main | `main` can't be deleted or force-pushed, and changes land through a pull request with the checks passing. |

## What it doesn't defend against

- **A compromised maintainer account.** The same account can tag a release.
  Rulesets slow that down; they don't stop someone holding the keys.
- **Bugs in egui.** They reach the apps through this crate as through any
  other egui app; they're reported upstream.
