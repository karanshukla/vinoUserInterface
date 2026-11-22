//! "Is there a newer release?" and "install it", for the About page's
//! update row and for the trays. No egui, and no HTTP stack: one anonymous
//! GET to GitHub's releases API through `curl`, which every app's installer
//! already needs. Nothing goes online until the user asks: [`Updater::check`]
//! is the only call that does.
//!
//! The app says how it installs a release with [`Updater::with_command`]: a
//! program and its arguments, run with `VINO_UPDATE_TAG` set to the tag. An
//! app without one still finds out about the release, and the row offers
//! its release page instead.

use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, TryRecvError};

use crate::family::App;

/// What the update row says and offers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// Not checked yet.
    Idle,
    /// Asking GitHub for the newest release.
    Checking,
    /// The running version is the newest release, or newer.
    UpToDate,
    /// The running version isn't a release (`dev`, a checkout past a tag);
    /// the newest release, by its tag. Never offered as an update.
    Development(String),
    /// A newer release, by its tag.
    Available(String),
    /// The app's update command is running for this tag.
    Installing(String),
    /// The command finished for this tag; the app needs a restart to run it.
    Installed(String),
    /// What failed, in a sentence for the row's help.
    Failed(String),
}

/// The update state of one app. Keep one for the life of the window and
/// call [`Updater::poll`] each frame; [`crate::about::page_with_updates`]
/// does both the polling and the drawing.
pub struct Updater {
    app: App,
    current: String,
    command: Option<Vec<String>>,
    status: Status,
    /// The tag `Failed` came from: retrying installs it again rather than
    /// checking from scratch.
    retry_tag: Option<String>,
    pending: Option<Receiver<Status>>,
}

impl Updater {
    /// `current` is the running version, usually `env!("CARGO_PKG_VERSION")`.
    pub fn new(app: App, current: impl Into<String>) -> Self {
        Updater {
            app,
            current: current.into(),
            command: None,
            status: Status::Idle,
            retry_tag: None,
            pending: None,
        }
    }

    /// How this app installs a release: a program and its arguments, run
    /// with `VINO_UPDATE_TAG` set to the tag, stdin closed. Exit 0 means
    /// installed; otherwise the last line it wrote to stderr is shown.
    pub fn with_command<I, S>(mut self, command: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let command: Vec<String> = command.into_iter().map(Into::into).collect();
        self.command = (!command.is_empty()).then_some(command);
        self
    }

    /// Start at `status` rather than `Idle`: for galleries and tests, which
    /// can't wait on GitHub.
    pub fn showing(mut self, status: Status) -> Self {
        self.status = status;
        self
    }

    pub fn app(&self) -> App {
        self.app
    }

    pub fn current(&self) -> &str {
        &self.current
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    /// Whether this app can install a release itself, rather than only
    /// point at it.
    pub fn can_install(&self) -> bool {
        self.command.is_some()
    }

    /// Something is running in the background: keep repainting.
    pub fn busy(&self) -> bool {
        self.pending.is_some()
    }

    /// Ask GitHub for the newest release, in the background.
    pub fn check(&mut self) {
        if self.busy() {
            return;
        }
        let (tx, rx) = mpsc::channel();
        let app = self.app;
        let current = self.current.clone();
        std::thread::spawn(move || {
            let status = match fetch_latest(app) {
                Ok(latest) if is_newer(&current, &latest) => Status::Available(latest),
                Ok(latest) if version_parts(&current).is_none() => Status::Development(latest),
                Ok(_) => Status::UpToDate,
                Err(e) => Status::Failed(e),
            };
            let _ = tx.send(status);
        });
        self.status = Status::Checking;
        self.pending = Some(rx);
    }

    /// Run the app's update command for the release `check` found.
    pub fn install(&mut self) {
        let Status::Available(tag) = &self.status else {
            return;
        };
        let Some(command) = self.command.clone() else {
            return;
        };
        if self.busy() {
            return;
        }
        let tag = tag.clone();
        let (tx, rx) = mpsc::channel();
        let for_thread = tag.clone();
        std::thread::spawn(move || {
            let _ = tx.send(run_update(&command, &for_thread));
        });
        self.retry_tag = Some(tag.clone());
        self.status = Status::Installing(tag);
        self.pending = Some(rx);
    }

    /// After a failure: install the same release again if that was what
    /// failed, otherwise check again.
    pub fn retry(&mut self) {
        match self.retry_tag.take() {
            Some(tag) if self.command.is_some() => {
                self.status = Status::Available(tag);
                self.install();
            }
            _ => self.check(),
        }
    }

    /// Take the background result if it has arrived. True when the status
    /// changed.
    pub fn poll(&mut self) -> bool {
        let Some(rx) = &self.pending else {
            return false;
        };
        let status = match rx.try_recv() {
            Ok(status) => status,
            Err(TryRecvError::Empty) => return false,
            Err(TryRecvError::Disconnected) => {
                Status::Failed("The update process exited unexpectedly.".into())
            }
        };
        self.pending = None;
        if !matches!(status, Status::Failed(_)) {
            self.retry_tag = None;
        }
        self.status = status;
        true
    }

    /// The release page, for an app with no update command, or for the notes.
    pub fn releases_url(&self) -> String {
        format!("https://github.com/{}/releases", self.app.repo())
    }
}

fn run_update(command: &[String], tag: &str) -> Status {
    let output = Command::new(&command[0])
        .args(&command[1..])
        .env("VINO_UPDATE_TAG", tag)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output();
    match output {
        Ok(out) if out.status.success() => Status::Installed(tag.to_string()),
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            Status::Failed(
                failure_line(&stderr).unwrap_or_else(|| format!("The update to {tag} failed.")),
            )
        }
        Err(e) => Status::Failed(format!("Couldn't run {}: {e}.", command[0])),
    }
}

/// The marks the vino installers start a terminal line with: `✗` a failure.
const TERMINAL_MARKS: [char; 3] = ['✗', '!', '·'];
/// The longest failure line shown, in characters, before an ellipsis.
const FAILURE_LINE_MAX: usize = 160;

/// Why a failed command stopped, as one line of help: the last line the vino
/// installers mark as the failure (`✗ ...`), else its last line, without
/// its mark (pinned by `tests::a_failure_shows_the_commands_last_words`).
fn failure_line(stderr: &str) -> Option<String> {
    let mut lines = stderr
        .lines()
        .rev()
        .map(str::trim)
        .filter(|l| !l.is_empty());
    let last = lines.clone().next()?;
    let line = lines.find(|l| l.starts_with('✗')).unwrap_or(last);
    Some(cut_to_one_line(
        line.trim_start_matches(TERMINAL_MARKS).trim(),
    ))
}

/// At most `FAILURE_LINE_MAX` characters, then `…` (pinned by
/// `tests::a_failure_line_of_the_limit_is_kept_whole`,
/// `tests::a_longer_failure_line_is_cut_with_an_ellipsis`).
fn cut_to_one_line(line: &str) -> String {
    let mut out: String = line.chars().take(FAILURE_LINE_MAX).collect();
    if line.chars().count() > FAILURE_LINE_MAX {
        out.push('…');
    }
    out
}

/// A version or tag as numbers: `v6` is `[6]`, `1.2.0` and `v1.2.0` are
/// `[1, 2, 0]`. Anything else (a `dev` build, a pre-release suffix) is `None`,
/// and is never offered an update (pinned by
/// `tests::a_dev_build_or_a_tag_that_is_no_version_is_never_told_to_update`).
pub fn version_parts(version: &str) -> Option<Vec<u64>> {
    let rest = version.strip_prefix('v').unwrap_or(version);
    rest.split('.').map(|part| part.parse().ok()).collect()
}

/// Whether `latest` is a newer release than `current`. Missing parts count
/// as 0, so the tag `v6` is the version 6.0.0.
pub fn is_newer(current: &str, latest: &str) -> bool {
    let (Some(mut current), Some(mut latest)) = (version_parts(current), version_parts(latest))
    else {
        return false;
    };
    let len = current.len().max(latest.len());
    current.resize(len, 0);
    latest.resize(len, 0);
    latest > current
}

/// The `"tag_name"` of a releases API response, if it looks like a tag. It
/// ends up in the window and in the update command's environment, so
/// anything but `v`, digits, letters, dots and dashes is refused (pinned by
/// `tests::only_a_plausible_tag_is_taken`).
pub fn latest_tag(json: &str) -> Option<String> {
    let release: serde_json::Value = serde_json::from_str(json).ok()?;
    let tag = release.get("tag_name")?.as_str()?;
    plausible_tag(tag).then(|| tag.to_string())
}

const TAG_MAX_LEN: usize = 32;

fn plausible_tag(tag: &str) -> bool {
    !tag.is_empty()
        && tag.len() <= TAG_MAX_LEN
        && tag.starts_with(|c: char| c == 'v' || c.is_ascii_digit())
        && tag
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
}

/// curl's limits on the check: GitHub's answer is a few KB.
const CHECK_TIMEOUT_SECS: &str = "10";
const CHECK_MAX_BYTES: &str = "1048576";

/// The newest full release's tag (GitHub's "latest": not a draft, not a
/// pre-release). The error is a sentence for the row's help.
pub fn fetch_latest(app: App) -> Result<String, String> {
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        app.repo()
    );
    let out = Command::new("curl")
        .args([
            "-fsSL",
            "--max-time",
            CHECK_TIMEOUT_SECS,
            "--max-filesize",
            CHECK_MAX_BYTES,
            "-H",
            "Accept: application/vnd.github+json",
            "--proto",
            "=https",
            "--proto-redir",
            "=https",
            "--tlsv1.2",
            "--",
            &url,
        ])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map_err(|_| "Could not check for updates: curl is not installed.".to_string())?;
    if !out.status.success() {
        return Err("Could not reach GitHub. Check your internet connection and try again.".into());
    }
    latest_tag(&String::from_utf8_lossy(&out.stdout))
        .ok_or_else(|| "Could not read the latest release from GitHub.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_and_tags_compare_as_numbers() {
        assert_eq!(version_parts("v6"), Some(vec![6]));
        assert_eq!(version_parts("1.2.0"), Some(vec![1, 2, 0]));
        assert_eq!(version_parts("dev"), None);
        assert_eq!(version_parts("v1.2.0-rc1"), None);
        assert!(is_newer("6.0.0", "v7"));
        assert!(!is_newer("6.0.0", "v6"));
        assert!(is_newer("1.2.0", "v1.10.0"));
        assert!(!is_newer("1.10.0", "v1.9.9"));
        assert!(is_newer("0.4.2", "v0.4.3"));
    }

    #[test]
    fn a_dev_build_or_a_tag_that_is_no_version_is_never_told_to_update() {
        assert!(!is_newer("dev", "v9"));
        assert!(!is_newer("1.0.0", "nightly"));
    }

    #[test]
    fn only_a_plausible_tag_is_taken() {
        assert_eq!(
            latest_tag(r#"{"url":"x","tag_name": "v1.3.0","name":"v1.3.0"}"#),
            Some("v1.3.0".into())
        );
        assert_eq!(latest_tag(r#"{"tag_name":"v6"}"#), Some("v6".into()));
        assert_eq!(latest_tag(r#"{"tag_name":"v1; rm -rf ~"}"#), None);
        assert_eq!(latest_tag(r#"{"tag_name":""}"#), None);
        assert_eq!(latest_tag(r#"{"message":"Not Found"}"#), None);
    }

    #[test]
    fn the_tag_is_read_from_githubs_answer() {
        let body = r#"{
  "url": "https://api.github.com/repos/o/r/releases/1",
  "id": 1,
  "author": {"login": "o", "id": 2},
  "tag_name": "v1.3.0",
  "name": "v1.3.0",
  "draft": false,
  "assets": [{"name": "x.tar.gz", "size": 3}],
  "body": "notes"
}"#;
        assert_eq!(latest_tag(body), Some("v1.3.0".into()));
        assert_eq!(latest_tag("{\"tag_name\"\n:\t\"v2\"}"), Some("v2".into()));
    }

    #[test]
    fn quotes_escaped_in_other_strings_do_not_end_them() {
        assert_eq!(
            latest_tag(r#"{"name":"the \"big\" one","tag_name":"v4","body":"say \"hi\""}"#),
            Some("v4".into())
        );
        assert_eq!(latest_tag(r#"{"tag_name":"v1\"x"}"#), None);
    }

    #[test]
    fn a_tag_name_inside_another_string_is_not_the_tag() {
        assert_eq!(
            latest_tag(r#"{"body":"set \"tag_name\": \"v9\" to release","tag_name":"v2"}"#),
            Some("v2".into())
        );
        assert_eq!(
            latest_tag(r#"{"body":"\"tag_name\": \"v9\""}"#),
            None,
            "and it is not taken when there is no tag"
        );
        assert_eq!(
            latest_tag(r#"{"body":"tag_name: v9","tag_name":"v2"}"#),
            Some("v2".into())
        );
    }

    #[test]
    fn only_the_releases_own_tag_name_counts() {
        assert_eq!(
            latest_tag(r#"{"author":{"tag_name":"v9"},"tag_name":"v2"}"#),
            Some("v2".into())
        );
        assert_eq!(latest_tag(r#"{"author":{"tag_name":"v9"}}"#), None);
        assert_eq!(latest_tag(r#"{"tag_name":"v2","na"#), None, "cut short");
    }

    #[test]
    fn a_tag_that_is_not_a_string_is_no_tag() {
        assert_eq!(latest_tag(r#"{"tag_name":null}"#), None);
        assert_eq!(latest_tag(r#"{"tag_name":6}"#), None);
        assert_eq!(latest_tag(""), None);
        assert_eq!(latest_tag("<html>rate limited</html>"), None);
    }

    #[test]
    fn a_failure_shows_the_commands_last_words() {
        assert_eq!(
            failure_line(
                "downloading\n✗ Checksum verification failed for x.tar.gz\n  Nothing was installed.\n\n"
            ),
            Some("Checksum verification failed for x.tar.gz".into())
        );
        assert_eq!(
            failure_line("✗ Download failed\n"),
            Some("Download failed".into())
        );
        assert_eq!(failure_line("one\nthe end\n"), Some("the end".into()));
        assert_eq!(failure_line("\n \n"), None);
    }

    #[test]
    fn a_failure_line_of_the_limit_is_kept_whole() {
        let line = "x".repeat(FAILURE_LINE_MAX);
        assert_eq!(failure_line(&line), Some(line));
    }

    #[test]
    fn a_longer_failure_line_is_cut_with_an_ellipsis() {
        let line = "é".repeat(FAILURE_LINE_MAX + 1);
        let shown = failure_line(&line).unwrap();
        assert_eq!(shown.chars().count(), FAILURE_LINE_MAX + 1);
        assert!(shown.ends_with("é…"), "{shown}");
    }

    fn available(command: &[&str]) -> Updater {
        let mut updater = Updater::new(App::Search, "6.0.0").with_command(command.iter().copied());
        updater.status = Status::Available("v7".into());
        updater
    }

    fn settle(updater: &mut Updater) {
        for _ in 0..500 {
            if updater.poll() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("no result");
    }

    #[test]
    fn install_runs_the_command_with_the_tag() {
        let mut updater = available(&["sh", "-c", "test \"$VINO_UPDATE_TAG\" = v7"]);
        updater.install();
        assert_eq!(updater.status(), &Status::Installing("v7".into()));
        settle(&mut updater);
        assert_eq!(updater.status(), &Status::Installed("v7".into()));
    }

    #[test]
    fn a_failed_install_says_why_and_retries_the_same_release() {
        let mut updater = available(&["sh", "-c", "echo '✗ No space left' >&2; exit 1"]);
        updater.install();
        settle(&mut updater);
        assert_eq!(updater.status(), &Status::Failed("No space left".into()));
        updater.command = Some(vec!["true".into()]);
        updater.retry();
        assert_eq!(updater.status(), &Status::Installing("v7".into()));
        settle(&mut updater);
        assert_eq!(updater.status(), &Status::Installed("v7".into()));
    }

    #[test]
    fn a_dev_build_is_told_what_the_release_is_but_never_offered_it() {
        let mut updater = Updater::new(App::AuthFace, "dev").with_command(["true"]);
        updater.status = Status::Development("v7".into());
        updater.install();
        assert_eq!(updater.status(), &Status::Development("v7".into()));
    }

    #[test]
    fn without_a_command_nothing_installs() {
        let mut updater = Updater::new(App::Pop, "1.3.0");
        updater.status = Status::Available("v1.4.0".into());
        assert!(!updater.can_install());
        updater.install();
        assert_eq!(updater.status(), &Status::Available("v1.4.0".into()));
        assert_eq!(
            updater.releases_url(),
            "https://github.com/karanshukla/vinoPop/releases"
        );
    }
}
