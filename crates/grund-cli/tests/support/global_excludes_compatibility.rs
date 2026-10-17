//! Optional pathname and malformed-config compatibility for the bounded repair
//! in §FS-scan.scope.ignore-files.pathname. Saved baseline outcomes govern cases
//! Git rejects; no test assumes an installed Git prefix or changes shell HOME.

use super::{BOTH, Fixture, git_path, write};
use std::fs;

const DECLARATION: &str = "# FS-y: Y\n\nCites \u{a7}FS-gone.\n";
const MALFORMED: &str = "[alias]\n broken = \"unterminated\n";

fn assert_scanned(fixture: &Fixture) {
    assert_eq!(fixture.listed(), (BOTH.into(), Some(0)));
    let checked = fixture.grund(&["check"]);
    assert_eq!(checked.status.code(), Some(1), "{checked:?}");
    assert!(
        String::from_utf8_lossy(&checked.stdout).contains("unknown reference FS-gone"),
        "{checked:?}"
    );
}

fn assert_unresolved_optional_path(name: &str, value: &str) {
    let fixture = Fixture::new(name);
    fixture.configure(value);
    fs::rename(
        fixture.dir.join("xdg/git/config"),
        fixture.home.join(".gitconfig"),
    )
    .unwrap();
    // Neither a literal lookalike nor another config/default may stand in for
    // the selected pathname when its interpolation context is unavailable.
    write(&fixture.repo.join(value), "notes/gen/\n");
    fixture.excludes("fallback.ignore");
    write(&fixture.dir.join("xdg/git/ignore"), "notes/gen/\n");
    write(&fixture.repo.join("notes/gen/b.md"), "Ordinary text.\n");
    fixture.assert_filtered();
    // Pin the scan result too: silently falling back could pass the plain-text
    // case above, but must not hide the declaration or its existing error.
    write(&fixture.repo.join("notes/gen/b.md"), DECLARATION);
    assert_scanned(&fixture);
}

/// R2-01: valid prefix syntax without prefix context remains optional. The
/// absence of Git-prefix discovery must not introduce a fatal scan rejection.
#[test]
fn an_unresolved_installation_prefix_is_an_optional_file() {
    assert_unresolved_optional_path("optional-prefix", "%(prefix)/global.ignore");
}

/// R2-01: an unavailable named home, with or without a suffix, must preserve
/// baseline success for ordinary text and the existing dangling-citation error.
/// This also covers platforms where named-user interpolation is unsupported.
#[test]
fn an_unresolved_named_home_is_an_optional_file() {
    for (name, value) in [
        (
            "optional-user-suffix",
            "~grund788nonexistentuser/global.ignore",
        ),
        ("optional-user-bare", "~grund788nonexistentuser"),
    ] {
        assert_unresolved_optional_path(name, value);
    }
}

/// R2-02: the baseline found the setting on either side of malformed unrelated
/// syntax. Recovery must preserve both literal tildes and single home expansion.
#[test]
fn unrelated_malformed_syntax_keeps_the_selected_excludes_path() {
    let mut fixture = Fixture::new("malformed-with-setting");
    fixture.home = fixture.dir.join("h~me");
    fs::create_dir_all(&fixture.home).unwrap();
    let empty = fixture.dir.join("empty.ignore");
    write(&empty, "");
    fixture.configure(&git_path(&empty));
    let mut values = Vec::new();
    for relative in ["real.ignore", "a~b/global.ignore"] {
        let file = fixture.dir.join(relative);
        write(&file, "notes/gen/\n");
        values.push(git_path(&file));
    }
    write(&fixture.home.join("a~b/global.ignore"), "notes/gen/\n");
    values.push("~/a~b/global.ignore".into());
    values.push("~//a~b/global.ignore".into());
    for value in values {
        let setting = format!(
            "[core]\n excludesFile = \"{value}\"\n excludesFile = {}\n",
            git_path(&empty)
        );
        // Before introducing invalid syntax, Git validates the intended file.
        // Its last-value policy differs from grund's first-value compatibility.
        let valid = format!("[core]\n excludesFile = \"{value}\"\n");
        write(&fixture.home.join(".gitconfig"), &valid);
        // The fixture's Git child reads GIT_CONFIG_GLOBAL at the XDG path.
        write(&fixture.dir.join("xdg/git/config"), &valid);
        fixture.git_ignores();
        fixture.configure(&git_path(&empty));
        for contents in [
            format!("{setting}{MALFORMED}"),
            format!("{MALFORMED}{setting}"),
        ] {
            write(&fixture.home.join(".gitconfig"), &contents);
            fixture.assert_filtered();
        }
    }
}

/// R2-02: recovery in XDG is still reached after malformed HOME without a
/// setting. A valid empty HOME assignment must allow the same fallthrough.
#[test]
fn malformed_config_recovery_keeps_discovery_fallthrough() {
    let fixture = Fixture::new("malformed-setting-fallthrough");
    fixture.excludes("a~b/global.ignore");
    let setting = fs::read_to_string(fixture.dir.join("xdg/git/config")).unwrap();
    write(
        &fixture.dir.join("xdg/git/config"),
        &format!("{MALFORMED}{setting}"),
    );
    for home in [MALFORMED.to_owned(), "[core]\n excludesFile =\n".into()] {
        write(&fixture.home.join(".gitconfig"), &home);
        fixture.assert_filtered();
    }
    // With no nonempty assignment in either config, retain the XDG default.
    write(&fixture.dir.join("xdg/git/config"), MALFORMED);
    write(&fixture.dir.join("xdg/git/ignore"), "notes/gen/\n");
    fixture.assert_filtered();
}

/// The parse-error recovery must not replace Git normalization for valid files,
/// including quoted spaces, inline comments and a literal interior tilde.
#[test]
fn valid_quoted_paths_keep_git_normalization() {
    let fixture = Fixture::new("quoted-path-normalization");
    let file = fixture.dir.join("a~b space/global.ignore");
    write(&file, "notes/gen/\n");
    let setting = format!(
        "[core]\n excludesFile = \"{}\" # keep the quoted space\n",
        git_path(&file)
    );
    write(&fixture.dir.join("xdg/git/config"), &setting);
    fixture.git_ignores();
    fixture.assert_filtered();
}
