//! Test module: a string literal that spans lines is one literal on every
//! surface (§FS-languages.multi-line-string, §FS-fmt.2.3.1). `check` and
//! `fmt --check` must give each line of one fixture the same verdict: silent on
//! the literal's interior, live again after its closer.

use std::path::{Path, PathBuf};

use super::*;
use crate::testing::{check_run, test_root, write};

/// A repo declaring `FS-042-user-login`, with one Rust file holding `source`.
fn rust_repo(name: &str, source: &str) -> PathBuf {
    let root = test_root(name);
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"docs/functional-spec\"\nindex = false\n\n\
         [scan]\ninclude = [\"docs\", \"src\"]\nextensions = [\"md\", \"rs\"]\n",
    );
    write(
        &root.join("docs/functional-spec/FS-042-user-login.md"),
        "# FS-042-user-login: User login\n\nLead.\n",
    );
    write(&root.join("src/lib.rs"), source);
    root
}

/// The lines `check` reports a shorthand on.
fn reported_lines(root: &Path) -> Vec<usize> {
    let run = check_run(root, false);
    let mut lines: Vec<usize> = run
        .report
        .errors
        .iter()
        .chain(run.report.warnings.iter())
        .filter(|finding| finding.code == "shorthand-citation")
        .map(|finding| finding.line.unwrap_or(0))
        .collect();
    lines.sort();
    lines
}

/// The lines `fmt --check` offers a rewrite on.
fn rewrite_lines(root: &Path) -> Vec<usize> {
    let output = format_references(FmtOpts {
        path: root.to_path_buf(),
        path_provided: true,
        ..FmtOpts::default()
    })
    .expect("fmt --check");
    let mut lines: Vec<usize> = output
        .changes
        .into_iter()
        .map(|change| change.line)
        .collect();
    lines.sort();
    lines
}

/// §FS-languages.multi-line-string.forms: the interior lines of a raw string
/// and of an ordinary Rust string are inside the literal, so neither surface
/// acts there; the comment after each closer is source again, so both act.
#[test]
fn check_and_fmt_agree_across_a_rust_multi_line_string() {
    let source = "fn usage() -> &'static str {\n\
                  \x20   r#\"\n\
                  Interior cites \u{a7}FS-042.\n\
                  \"#\n\
                  }\n\
                  fn hint() -> &'static str {\n\
                  \x20   \"first\n\
                  Interior cites \u{a7}FS-042.\n\
                  last\" // Tail cites \u{a7}FS-042.\n\
                  }\n\
                  const X: &str = r#\"one\n\
                  \"#; // Closing tail cites \u{a7}FS-042.\n";
    let root = rust_repo(
        "check_and_fmt_agree_across_a_rust_multi_line_string",
        source,
    );
    assert_eq!(reported_lines(&root), vec![9, 12]);
    assert_eq!(rewrite_lines(&root), vec![9, 12]);
}

/// §FS-languages.multi-line-string.bound: an opener whose closer never comes
/// opens nothing, so the file reads exactly as it does line by line.
#[test]
fn an_unclosed_raw_string_opens_nothing() {
    let source = "fn broken() { let s = r#\"never closed; }\n\
                  // Later cites \u{a7}FS-042.\n";
    let root = rust_repo("an_unclosed_raw_string_opens_nothing", source);
    assert_eq!(reported_lines(&root), vec![2]);
    assert_eq!(rewrite_lines(&root), vec![2]);
}
