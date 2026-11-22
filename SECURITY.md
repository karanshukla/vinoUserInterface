# Security

vinoUserInterface is a theme and widget library. It does not write files to
disk, and it makes network requests in only one case: when the user explicitly
checks for updates, it runs `curl` to send an anonymous GET request to GitHub's
releases API (`api.github.com/repos/<owner>/<repo>/releases/latest`). No network
requests occur automatically. If the user chooses to install an update, the
library runs the update command supplied by the host application with
`VINO_UPDATE_TAG` set to the release tag; it does not download or install
binaries itself.

To follow the desktop's light/dark, reduced-motion, and text-scale preferences,
the library reads the following under the current user account:

- The XDG Desktop Portal `Settings` interface over D-Bus via
  `busctl --user --timeout=1`, falling back to `gsettings` when no portal
  responds (both resolved from `PATH`).
- `kdeglobals` and `kcmfontsrc` in `$XDG_CONFIG_HOME` (or `~/.config`) on KDE
  Plasma.
- Environment variables: `VINO_UI_THEME`, `XDG_CURRENT_DESKTOP`,
  `XDG_CONFIG_HOME`, and `HOME`.

The primary security risk is supply-chain exposure: multiple vinoPilot apps
(vinoSearch, vinoWhisper, vinoSpeak, and others) compile this crate in, so a
compromised release affects all of them.

## Reporting

Use [private vulnerability reporting](https://github.com/karanshukla/vinoUserInterface/security/advisories/new)
instead of opening a public issue for exploitable vulnerabilities. If an issue
also affects specific applications, mention which ones so fixes can be released
here and bumped across downstream apps.

## Security controls

| Threat | Mitigation |
|---|---|
| Memory-safety bug in this crate | `#![forbid(unsafe_code)]` disallows `unsafe` blocks at compile time. |
| Malformed desktop configuration | Output from `busctl`, `gsettings`, and KDE config files is parsed as data and never executed. Unparseable values are ignored, and text scaling is clamped to a fixed range. A same-user attacker who can modify `PATH` or `~/.config` already has code execution as the user. |
| Compromised or yanked dependency | Only three runtime dependencies: `egui`, `svgtypes` (for icon SVG path data), and `serde_json` (for GitHub release JSON). `cargo-deny` checks advisories, licenses, and crate sources (`deny.toml`, crates.io only; dev-dependencies are excluded because they do not ship). `Cargo.lock` is enforced in CI with `--locked`. |
| Newly compromised dependency release | Dependabot enforces a 7-day cooldown before proposing updates (security fixes bypass the cooldown), and pull requests never auto-merge. |
| Compromised CI action | Every GitHub Action is pinned to a commit SHA. Workflows use a read-only token and do not persist checkout credentials. |
| Release mismatch with repository source | Releases publish only from `v*` tags using crates.io trusted publishing with short-lived per-run tokens (no stored API secrets). The `crates-io` environment accepts only `v*` tags, reruns all checks, and rejects tags that do not match `Cargo.toml`. |
| Unreviewed changes on `main` | `main` is protected against deletion and force-pushes, and changes require a pull request with passing checks. |

## Out of scope

- **Compromised maintainer account:** A maintainer with push access can tag a
  release. Branch rulesets reduce accidental or unreviewed changes but cannot
  prevent abuse of a compromised account.
- **Upstream bugs in egui:** Issues in egui itself affect apps using this crate
  and are reported upstream.
