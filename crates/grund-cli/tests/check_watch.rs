//! Real CLI acceptance for §FS-check.6, using bounded output-driven subprocesses.
//! Signal scenarios use Unix SIGINT; private observer/PTY seams are specified in
//! §AR-bindings.3 and the handoff matrix, not faked by these production tests.

#![cfg(unix)]

#[path = "support/watch_process.rs"]
mod watch_process;

use std::fs;
use watch_process::{BAD, CLEAN, CONFIG, Fixture, Watch};

/// §FS-check.6.2.1, §FS-check.6.3.3: immediate text, save, repair and status 0.
#[test]
fn watch_text_checks_immediately_then_reports_save_and_repair() {
    let f = Fixture::new();
    let clean = f.check(&[]);
    assert_eq!(clean.status.code(), Some(0));
    assert_eq!(clean.stdout, b"success\n");
    assert!(clean.stderr.is_empty());
    let mut watch = Watch::start(&f, &[]);
    watch.report(&clean);
    f.write("src/main.rs", BAD);
    let bad = f.check(&[]);
    assert_eq!(bad.status.code(), Some(1));
    watch.report(&bad);
    f.write("src/main.rs", CLEAN);
    watch.report(&f.check(&[]));
    watch.interrupt(0);
}

/// §FS-check.6.2: options and narrowed scope preserve ordinary reports.
fn option_parity(args: &[&str]) {
    let f = Fixture::new();
    f.write("src/main.rs", BAD);
    f.write("docs/example.md", "Example `<\u{a7}>FS-live`.\n");
    if args.contains(&"--format=text") {
        f.write(
            "grund.toml",
            &format!("{CONFIG}\n[output]\nformat = \"json\"\n"),
        );
    }
    if args.contains(&"--ignore=unused") {
        f.write("docs/functional-spec/FS-unused.md", "# FS-unused: Unused\n");
        f.write("docs/functional-spec/README.md",
            "# FS index\n\n- [\u{a7}FS-live](FS-live.md#fs-live-live)\n- [\u{a7}FS-unused](FS-unused.md#fs-unused-unused)\n");
    }
    if args.contains(&"--require-grounding") {
        f.write("src/ungrounded.rs", "fn ungrounded() {}\n");
    }
    if args.contains(&"--full") {
        f.write(
            "grund.toml",
            &format!("{CONFIG}\n[scan]\ninclude = [\"docs/functional-spec\", \"src\"]\n"),
        );
        f.write("outside/note.md", "\u{a7}FS-outside-missing\n");
    }
    if args.contains(&"src/main.rs") {
        f.write("src/other.rs", "// \u{a7}FS-other-missing\n");
    }
    let ordinary = f.check(args);
    assert_ne!(
        ordinary.status.code(),
        Some(2),
        "invalid baseline {args:?}: {ordinary:?}"
    );
    let defaults = f.check(&[]);
    assert!(
        ordinary.stdout != defaults.stdout || ordinary.stderr != defaults.stderr,
        "option fixture does not distinguish {args:?} from default checking"
    );
    let mut watch = Watch::start(&f, args);
    watch.report(&ordinary);
    watch.interrupt(ordinary.status.code().unwrap());
}

macro_rules! parity_test {
    ($name:ident, $args:expr) => {
        #[test]
        fn $name() {
            option_parity($args);
        }
    };
}

parity_test!(watch_explicit_text_parity, &["--format=text"]);
parity_test!(
    watch_selector_composition_parity,
    &["--only=dangling", "--only", "unused", "--ignore=unused"]
);
parity_test!(watch_selected_empty_text_parity, &["--ignore=dangling"]);
parity_test!(
    watch_suggestions_parity,
    &["--suggestions", "--only=escaped-citation-resolves"]
);
parity_test!(watch_grounding_parity, &["--require-grounding"]);
parity_test!(watch_full_scope_parity, &["--full"]);
parity_test!(
    watch_trial_rule_selection_parity,
    &[
        "--rule",
        "Each FS must cite at least one FS.",
        "--only-rule"
    ]
);
parity_test!(
    watch_narrowed_path_parity,
    &["src/main.rs", "--only=dangling"]
);

/// §FS-check.6.2.1: finding NDJSON, no framing/clearing, and status 1.
#[test]
fn watch_json_finding_bytes_match_one_shot_and_interrupt_returns_one() {
    let f = Fixture::new();
    f.write("src/main.rs", BAD);
    let args = ["--format=json", "--only=dangling"];
    let expected = f.check(&args);
    assert_eq!(expected.status.code(), Some(1));
    for line in String::from_utf8_lossy(&expected.stdout).lines() {
        let row: serde_json::Value = serde_json::from_str(line).unwrap();
        assert_eq!(row["code"], "dangling");
    }
    let mut watch = Watch::start(&f, &args);
    watch.report(&expected);
    f.write(
        "src/main.rs",
        "// \u{a7}FS-live\n// \u{a7}FS-another-missing\n",
    );
    watch.report(&f.check(&args));
    watch.interrupt(1);
}

/// §FS-check.6.1.3, §FS-check.6.3.1: invalid initial config retains repair coverage.
#[test]
fn watch_invalid_initial_config_remains_resident_and_repairs() {
    let f = Fixture::new();
    f.write("grund.toml", "grund_config_version = \"invalid\"\n");
    let invalid = f.check(&[]);
    assert_eq!(invalid.status.code(), Some(2));
    assert!(invalid.stdout.is_empty());
    let mut watch = Watch::start(&f, &[]);
    watch.report(&invalid);
    f.write("grund.toml", CONFIG);
    watch.report(&f.check(&[]));
    watch.interrupt(0);
}

/// §FS-check.6.3.1: read failures remain resident, with their ordinary error bytes.
#[test]
fn watch_read_error_remains_resident_and_repairs() {
    use std::os::unix::fs::symlink;

    let f = Fixture::new();
    let mut watch = Watch::start(&f, &[]);
    watch.report(&f.check(&[]));
    symlink("absent.rs", f.0.join("src/broken.rs")).unwrap();
    let error = f.check(&[]);
    assert_eq!(error.status.code(), Some(2));
    watch.report(&error);
    fs::remove_file(f.0.join("src/broken.rs")).unwrap();
    watch.report(&f.check(&[]));
    watch.interrupt(0);
}

/// §FS-check.6.1.3: followed file targets outside source roots are effective inputs.
#[test]
fn watch_followed_external_file_link_updates_report() {
    use std::os::unix::fs::symlink;

    let f = Fixture::new();
    let external = Fixture::new();
    fs::remove_file(f.0.join("src/main.rs")).unwrap();
    symlink(external.0.join("src/main.rs"), f.0.join("src/main.rs")).unwrap();
    let mut watch = Watch::start(&f, &[]);
    watch.report(&f.check(&[]));
    external.write("src/main.rs", BAD);
    let bad = f.check(&[]);
    assert_eq!(bad.status.code(), Some(1));
    watch.report(&bad);
    external.write("src/main.rs", CLEAN);
    watch.report(&f.check(&[]));
    watch.interrupt(0);
}

/// §FS-check.6.4: static invocation failures terminate with ordinary diagnostics.
#[test]
fn watch_static_validation_keeps_one_shot_error_bytes() {
    let f = Fixture::new();
    let cases: &[&[&str]] = &[
        &["--format=invalid"],
        &["--only=not-a-code"],
        &["--only-rule"],
        &["--rule"],
        &["src", "docs"],
    ];
    for args in cases {
        let ordinary = f.check(args);
        assert_eq!(ordinary.status.code(), Some(2));
        let mut watched = vec!["--watch"];
        watched.extend_from_slice(args);
        let actual = f.check(&watched);
        assert_eq!(actual.status.code(), Some(2));
        assert_eq!(actual.stdout, ordinary.stdout);
        assert_eq!(
            actual.stderr, ordinary.stderr,
            "static watch error differs for {args:?}"
        );
    }
}

/// §FS-check.6.3.3: an ordinary completed error run supplies SIGINT status 2.
#[test]
fn watch_interrupt_after_config_error_returns_two() {
    let f = Fixture::new();
    let mut watch = Watch::start(&f, &[]);
    watch.report(&f.check(&[]));
    f.write("grund.toml", "grund_config_version = \"invalid\"\n");
    watch.report(&f.check(&[]));
    watch.interrupt(2);
}

/// §FS-check.6.1.3: hidden discovery and atomic config precedence changes.
#[test]
fn watch_discovers_hidden_config_after_root_config_removal() {
    let f = Fixture::new();
    let mut watch = Watch::start(&f, &[]);
    watch.report(&f.check(&[]));
    f.write(".agents/grund.toml", "grund_config_version = \"invalid\"\n");
    fs::remove_file(f.0.join("grund.toml")).unwrap();
    watch.report(&f.check(&[]));
    f.write(".agents/next.toml", CONFIG);
    fs::rename(
        f.0.join(".agents/next.toml"),
        f.0.join(".agents/grund.toml"),
    )
    .unwrap();
    watch.report(&f.check(&[]));
    watch.interrupt(0);
}

/// §FS-check.6.1.3: effective ignore changes must refresh input discovery.
#[test]
fn watch_ignore_edits_change_the_report_and_repair() {
    let f = Fixture::new();
    f.write("src/main.rs", BAD);
    let mut watch = Watch::start(&f, &[]);
    watch.report(&f.check(&[]));
    f.write(".ignore", "src/main.rs\n");
    let ignored = f.check(&[]);
    assert_eq!(ignored.status.code(), Some(0));
    watch.report(&ignored);
    fs::remove_file(f.0.join(".ignore")).unwrap();
    watch.report(&f.check(&[]));
    watch.interrupt(1);
}

/// §FS-check.6.1.1, §FS-check.6.1.3: real saves and parent-anchor reattachment.
#[test]
fn watch_atomic_create_rename_delete_and_recreated_root() {
    let f = Fixture::new();
    let mut watch = Watch::start(&f, &[]);
    watch.report(&f.check(&[]));
    // Save via a non-source temporary file so only the final state is checked.
    f.write("src/save.tmp", BAD);
    fs::rename(f.0.join("src/save.tmp"), f.0.join("src/main.rs")).unwrap();
    watch.report(&f.check(&[]));
    fs::rename(f.0.join("src/main.rs"), f.0.join("src/renamed.rs")).unwrap();
    watch.report(&f.check(&[]));
    fs::remove_dir_all(f.0.join("src")).unwrap();
    watch.report(&f.check(&[]));
    f.write("src/recreated.rs", BAD);
    watch.report(&f.check(&[]));
    fs::remove_file(f.0.join("src/recreated.rs")).unwrap();
    watch.report(&f.check(&[]));
    watch.interrupt(0);
}

/// §FS-cli.3.3: terminal discovery must expose the resident mode and its guide.
#[test]
fn watch_check_help_describes_mode_and_links_guide() {
    let f = Fixture::new();
    let help = f.check(&["--help"]);
    assert_eq!(help.status.code(), Some(0));
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(text.contains("--watch"), "check help omits --watch");
    assert!(
        text.contains("docs/user-facing/watch.md"),
        "check help omits public watch guide"
    );
}
