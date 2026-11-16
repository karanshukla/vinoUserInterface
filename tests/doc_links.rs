//! Every test a comment cites must be a `#[test]` fn, so a rule that names
//! its test fails CI when the test is renamed or deleted. rustdoc can't check
//! these: `#[test]` fns are compiled out of the docs. See
//! docs/comment-style.md.

use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn every_cited_test_exists() {
    let mut broken = Vec::new();
    for file in ["src", "tests", "examples"]
        .iter()
        .flat_map(|dir| rust_files(Path::new(dir)))
        .filter(|file| !file.ends_with("doc_links.rs"))
    {
        let text = fs::read_to_string(&file).unwrap();
        for (line_no, line) in text.lines().enumerate() {
            if !line.trim_start().starts_with("//") {
                continue;
            }
            for cited in cited_tests(line) {
                if !resolves(&file, &text, cited) {
                    broken.push(format!("{}:{}: `{cited}`", file.display(), line_no + 1));
                }
            }
        }
    }
    assert!(broken.is_empty(), "no such test:\n{}", broken.join("\n"));
}

#[test]
fn the_check_finds_a_cited_test_and_rejects_a_missing_one() {
    let text = "/// (pinned by `tests::real`)\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn real() {}\n    fn helper() {}\n}\n";
    let here = Path::new("src/here.rs");
    assert_eq!(cited_tests(text.lines().next().unwrap()), ["tests::real"]);
    assert!(resolves(here, text, "tests::real"));
    assert!(!resolves(here, text, "tests::helper"), "not a #[test]");
    assert!(!resolves(here, text, "tests::gone"));
}

#[test]
fn the_check_follows_module_paths_and_integration_tests() {
    let here = Path::new("src/lib.rs");
    assert!(resolves(
        here,
        "",
        "theme::desktop::tests::a_key_is_read_from_its_own_group_only"
    ));
    assert!(resolves(
        here,
        "",
        "widgets::tests::the_sidebar_breakpoint_leaves_a_row_exactly_narrow"
    ));
    assert!(resolves(
        here,
        "",
        "tests/doc_links.rs::the_check_follows_module_paths_and_integration_tests"
    ));
    assert!(!resolves(here, "", "tests/doc_links.rs::rust_files"));
    assert!(!resolves(here, "", "tests/gone.rs::anything"));
}

fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            files.extend(rust_files(&path));
        } else if path.extension().is_some_and(|e| e == "rs") {
            files.push(path);
        }
    }
    files
}

/// Code spans like `` `tests::name` ``, `` `module::tests::name` `` or
/// `` `tests/file.rs::name` ``.
fn cited_tests(line: &str) -> Vec<&str> {
    line.split('`')
        .skip(1)
        .step_by(2)
        .filter(|span| {
            (span.contains("tests::") || (span.starts_with("tests/") && span.contains(".rs::")))
                && span
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "_:/.".contains(c))
        })
        .collect()
}

/// `tests::name` is in `text` (the file at `here`), or beside a `mod.rs` in
/// its `tests.rs`; `a::b::tests::name` is in `src/a/b.rs`, `src/a/b/mod.rs`
/// or `src/a/b/tests.rs`; `tests/file.rs::name` is in that file.
fn resolves(here: &Path, text: &str, cited: &str) -> bool {
    let has_test =
        |path: PathBuf| fs::read_to_string(path).is_ok_and(|t| is_test_fn(&t, name_of(cited)));
    if let Some((file, name)) = cited.split_once(".rs::") {
        return fs::read_to_string(format!("{file}.rs")).is_ok_and(|t| is_test_fn(&t, name));
    }
    let (module, name) = cited.rsplit_once("tests::").unwrap();
    let module = module.trim_end_matches("::");
    if module.is_empty() {
        return is_test_fn(text, name)
            || (here.ends_with("mod.rs") && has_test(here.with_file_name("tests.rs")));
    }
    let base = Path::new("src").join(module.replace("::", "/"));
    [
        base.with_extension("rs"),
        base.join("mod.rs"),
        base.join("tests.rs"),
    ]
    .into_iter()
    .any(has_test)
}

fn name_of(cited: &str) -> &str {
    cited.rsplit("::").next().unwrap()
}

/// A `fn name(` whose attributes, above it, include `#[test]`.
fn is_test_fn(text: &str, name: &str) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    let signature = format!("fn {name}(");
    lines.iter().enumerate().any(|(i, line)| {
        line.trim_start().starts_with(&signature)
            && lines[..i]
                .iter()
                .rev()
                .map(|l| l.trim())
                .take_while(|l| l.starts_with("#[") || l.starts_with("///"))
                .any(|l| l == "#[test]")
    })
}
