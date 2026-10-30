//! §REQ-remote-equivalence.4, source against projection on the command line: a tree a
//! consumer imports answers every command as the tree's own standalone run does
//! (§FS-imports.mount, §FS-remote-projects.mount.rule), and a command that writes
//! writes only the consumer's files (§FS-imports.writes). The inventory is the command
//! list `grund --help` prints, so a command added without a row fails
//! [`every_command_has_an_inventory_row`] rather than passing unproven.
//!
//! The relocation is the support module's. The tree holds a nested member, its own
//! dangling citation and rewrites of its own, so suppression and read-only writes have
//! something to hide.
//! The projection against a locally mounted member is owed with `mounted_members`.

#[path = "support/mount_fixture.rs"]
mod mount_fixture;

use mount_fixture::{Fixture, compare_bytes, compare_json, qualified, rows, text};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::process::Command;

/// A row's comparison over the shared fixture.
type Row = fn(&Fixture);

/// One row per command: `quiet` imports `lib` and cites nothing of it, so every count
/// is the tree's own; `app` imports it and cites it, one coordinate per line.
const ROWS: &[(&str, Row)] = &[
    ("show", show),
    ("list", list),
    ("refs", refs),
    ("cover", cover),
    ("check", check),
    ("fmt", fmt),
    ("id", id),
    ("fetch", fetch),
    ("init", init),
    ("complete", complete),
    ("config", reads_no_import),
    ("schema", reads_no_import),
    ("guide", reads_no_import),
    ("integrations", reads_no_import),
    ("completions", reads_no_import),
];

/// The completion helper `complete` is dispatched but kept out of help (§FS-completions.2).
const HIDDEN: &[&str] = &["complete"];

fn show(fixture: &Fixture) {
    for (source, projected) in [
        ("FS-x", "dep/FS-x"),
        ("FS-x.1", "dep/FS-x.1"),
        ("core/FS-y", "dep/core/FS-y"),
    ] {
        compare_json(
            fixture,
            &["show", source, "--format", "json"],
            &["show", projected, "--format", "json"],
            false,
        );
        for form in ["--brief", "--toc", "--full"] {
            compare_bytes(fixture, &["show", source, form], &["show", projected, form]);
        }
    }
    // A coordinate the tree does not declare is refused alike.
    compare_bytes(fixture, &["show", "FS-gone"], &["show", "dep/FS-gone"]);
}

fn list(fixture: &Fixture) {
    compare_json(
        fixture,
        &["list", "--format", "json"],
        &["list", "--format", "json"],
        true,
    );
    let unused = ["list", "--unused", "--format", "json"];
    let (native, through) = (
        fixture.grund("lib", &unused),
        fixture.grund("quiet", &unused),
    );
    assert_eq!(
        (native.status.code(), &native.stdout),
        (through.status.code(), &through.stdout)
    );
}

fn refs(fixture: &Fixture) {
    for (source, projected) in [("FS-x", "dep/FS-x"), ("core/FS-y", "dep/core/FS-y")] {
        compare_json(
            fixture,
            &["refs", source, "--format", "json"],
            &["refs", projected, "--format", "json"],
            false,
        );
    }
    for (source, projected) in [("FS-x.1", "dep/FS-x.1"), ("core/FS-y", "dep/core/FS-y")] {
        compare_json(
            fixture,
            &["refs", "--from", source, "--format", "json"],
            &["refs", "--from", projected, "--format", "json"],
            false,
        );
    }
    compare_json(
        fixture,
        &["refs", "--glob", "FS-*", "--format", "json"],
        &["refs", "--glob", "dep/FS-*", "--format", "json"],
        false,
    );
}

fn cover(fixture: &Fixture) {
    compare_json(
        fixture,
        &["cover", "--format", "json"],
        &["cover", "--format", "json"],
        false,
    );
    compare_json(
        fixture,
        &["cover", "--glob", "FS-*", "--format", "json"],
        &["cover", "--glob", "dep/FS-*", "--format", "json"],
        false,
    );
    // A file of the tree is its own run's file, as §FS-imports.mount reads a path into it.
    let lines = |file: &'static str| ["cover", file, "--lines", "5", "--format", "json"];
    compare_bytes(
        fixture,
        &lines("docs/fs/FS-x.md"),
        &lines("../lib/docs/fs/FS-x.md"),
    );
}

fn complete(fixture: &Fixture) {
    for (source, projected) in [("FS-", "dep/FS-"), ("core/", "dep/core/")] {
        let ask = |side: &str, prefix: &str| {
            text(
                &fixture
                    .grund(side, &["complete", "ids", "--prefix", prefix])
                    .stdout,
            )
        };
        let native = ask("lib", source)
            .lines()
            .map(qualified)
            .collect::<Vec<_>>();
        let through = ask("quiet", projected)
            .lines()
            .map(str::to_string)
            .collect::<Vec<_>>();
        assert!(!native.is_empty(), "the tree completes `{source}`");
        assert_eq!(
            native, through,
            "completion of `{source}` against `{projected}`"
        );
    }
}

/// §FS-imports.findings: the tree's own findings are its standalone run's, suppressed in
/// the consumer's; a consumer citation fails exactly where the tree declares nothing.
fn check(fixture: &Fixture) {
    let native = fixture.grund("lib", &["check", "--format", "json"]);
    assert_eq!(
        native.status.code(),
        Some(1),
        "the tree's own run has findings to suppress"
    );
    let target = fixture.grund("app", &["check", "../lib", "--format", "json"]);
    assert_eq!(
        (target.status.code(), &target.stdout),
        (native.status.code(), &native.stdout)
    );
    let consumer = fixture.grund("app", &["check", "--format", "json"]);
    let lib = fixture.dir("lib");
    let findings = rows(&consumer, &fixture.dir("app"), false, false);
    assert!(
        findings
            .iter()
            .all(|row| !row.contains(&*lib.display().to_string())),
        "{findings:?}"
    );
    let notes = fs::read_to_string(fixture.0.join("app/docs/notes.md")).expect("notes");
    for (index, line) in notes.lines().enumerate() {
        let cited = line
            .trim_start_matches("See \u{a7}dep/")
            .trim_end_matches('.');
        let declared = fixture.grund("lib", &["show", cited]).status.success();
        let site = format!(
            "\"path\":\"docs/notes.md\",\"line\":{},\"code\":\"dangling\"",
            index + 1
        );
        let dangles = text(&consumer.stdout).contains(&site);
        assert_eq!(
            dangles, !declared,
            "the consumer's verdict on `{cited}` is the tree's"
        );
    }
}

/// §FS-imports.writes, §FS-fmt.7.3: the preview names exactly the lines the write
/// changes, all the consumer's, and each link written into a consumer file names the
/// file and anchor the tree's own run gives the coordinate. The tree's own pending
/// rewrites stay its own.
fn fmt(fixture: &Fixture) {
    assert_eq!(
        fixture.grund("lib", &["fmt", "--check"]).status.code(),
        Some(1)
    );
    let (lib, app) = (fixture.snapshot("lib"), fixture.snapshot("app"));
    let preview = fixture.grund("app", &["fmt", "--check"]);
    let named = text(&preview.stdout)
        .lines()
        .map(|line| {
            let mut parts = line.splitn(3, ':');
            let path = fixture.dir("app").join(parts.next().expect("path"));
            (
                path,
                parts
                    .next()
                    .expect("line")
                    .parse::<usize>()
                    .expect("number"),
            )
        })
        .collect::<BTreeSet<_>>();
    assert!(fixture.grund("app", &["fmt", "--write"]).status.success());
    let before = app.into_iter().collect::<BTreeMap<_, _>>();
    let mut changed = BTreeSet::new();
    for (path, after) in fixture.snapshot("app") {
        let old = before
            .get(&path)
            .map(|bytes| text(bytes))
            .unwrap_or_default();
        let new = text(&after);
        let (old, new) = (
            old.lines().collect::<Vec<_>>(),
            new.lines().collect::<Vec<_>>(),
        );
        for index in 0..old.len().max(new.len()) {
            if old.get(index) != new.get(index) {
                changed.insert((path.canonicalize().expect("canonical"), index + 1));
            }
        }
    }
    assert!(
        !changed.is_empty() && named == changed,
        "previewed {named:?}, wrote {changed:?}"
    );
    assert_eq!(
        fixture.snapshot("lib"),
        lib,
        "the tree's bytes are read-only"
    );
    let notes = fs::read_to_string(fixture.0.join("app/docs/notes.md")).expect("notes");
    for link in notes.split("[\u{a7}dep/").skip(1) {
        let (cited, rest) = link.split_once("](").expect("a written link");
        let (file, anchor) = rest[..rest.find(')').expect("link end")]
            .split_once('#')
            .expect("anchor");
        let shown = fixture.grund("lib", &["show", cited, "--format", "json"]);
        let shown: Value = serde_json::from_slice(&shown.stdout).expect("show JSON");
        let declared = fixture
            .dir("lib")
            .join(shown["path"].as_str().expect("path"));
        let written = fixture
            .dir("app")
            .join("docs")
            .join(file)
            .canonicalize()
            .expect("link file");
        assert_eq!(
            (written, anchor),
            (declared, shown["anchor"].as_str().expect("anchor"))
        );
    }
    assert!(fixture.grund("app", &["fmt", "--check"]).status.success());
}

/// The target's proposal is its standalone run's, and the consumer proposes in its own folders.
fn id(fixture: &Fixture) {
    let ask = ["id", "FS", "new thing", "--format", "json"];
    let native = fixture.grund("lib", &ask);
    let target = fixture.grund(
        "quiet",
        &["id", "FS", "new thing", "../lib", "--format", "json"],
    );
    assert_eq!(
        (native.status.code(), &native.stdout),
        (target.status.code(), &target.stdout)
    );
    let consumer = fixture.grund("quiet", &["id", "FS", "x", "--format", "json"]);
    let consumer: Value = serde_json::from_slice(&consumer.stdout).expect("id JSON");
    assert_eq!(
        consumer["id"], "FS-x",
        "the tree's FS-x is in another namespace"
    );
}

/// §FS-imports.writes: a fetch into the tree is refused before its integration runs.
fn fetch(fixture: &Fixture) {
    let lib = fixture.snapshot("lib");
    let refused = fixture.grund("app", &["fetch", "dep/TICKET-1"]);
    assert_eq!(refused.status.code(), Some(2), "{}", text(&refused.stderr));
    assert!(text(&refused.stderr).contains("run grund fetch TICKET-1 in ../lib"));
    assert_eq!(fixture.snapshot("lib"), lib);
}

/// §FS-imports.writes: `init` in the consumer writes nothing of the tree.
fn init(fixture: &Fixture) {
    let lib = fixture.snapshot("lib");
    let git = Command::new("git")
        .args(["init", "-q"])
        .current_dir(fixture.0.join("quiet"))
        .status();
    assert!(git.expect("git").success());
    let init = fixture.grund("quiet", &["init"]);
    assert!(init.status.success(), "{}", text(&init.stderr));
    assert_eq!(fixture.snapshot("lib"), lib);
    let agents = fs::read_to_string(fixture.0.join("quiet/AGENTS.md")).expect("AGENTS.md");
    assert!(
        !agents.contains("dep/") && !agents.contains("../lib"),
        "the block lists no import"
    );
}

/// Reads nothing of the tree: the same bytes after the tree gains a kind and a declaration.
fn reads_no_import(fixture: &Fixture) {
    let commands: [&[&str]; 5] = [
        &["config", "show"],
        &["schema"],
        &["guide"],
        &["integrations"],
        &["completions", "bash"],
    ];
    let ask = || {
        commands
            .map(|args| fixture.grund("quiet", args))
            .map(|out| (out.status.code(), out.stdout))
    };
    let before = ask();
    let config = fs::read_to_string(fixture.0.join("lib/grund.toml")).expect("lib config");
    fixture.write(
        "lib/grund.toml",
        &format!("{config}\n[schema.kinds.ADR]\nfolders = [\"docs/adr\"]\n"),
    );
    fixture.write("lib/docs/adr/ADR-w.md", "# ADR-w: W\n");
    assert_eq!(ask(), before);
    fixture.write("lib/grund.toml", &config);
    fs::remove_dir_all(fixture.0.join("lib/docs/adr")).expect("restore the tree");
}

/// The commands `grund --help` lists, in its groups.
fn listed_commands() -> BTreeSet<String> {
    let help = Command::new(env!("CARGO_BIN_EXE_grund"))
        .arg("--help")
        .output()
        .expect("help");
    text(&help.stdout)
        .lines()
        .filter(|line| line.starts_with("  ") && !line.starts_with("   "))
        .filter_map(|line| line.split_whitespace().next().map(str::to_string))
        .collect()
}

#[test]
fn every_command_has_an_inventory_row() {
    let rows = ROWS
        .iter()
        .map(|(name, _)| name.to_string())
        .collect::<BTreeSet<_>>();
    let mut commands = listed_commands();
    assert!(
        commands.len() >= 10,
        "the help lists the commands: {commands:?}"
    );
    commands.extend(HIDDEN.iter().map(|name| name.to_string()));
    assert_eq!(
        rows, commands,
        "a command without a row, or a row without a command"
    );
}

#[test]
fn an_imported_tree_answers_every_command_as_its_standalone_run() {
    let fixture = Fixture::new("every-command");
    for (name, row) in ROWS {
        eprintln!("row {name}");
        row(&fixture);
    }
}
