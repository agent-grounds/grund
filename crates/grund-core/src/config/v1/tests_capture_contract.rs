//! Migration capture is opt-in (§FS-config-migrate.4.3, §AR-config.8.1). The observer
//! counts records actually constructed, including those an ordinary read drops;
//! it is thread-local and compiled only into unit tests.

use std::cell::Cell;
use std::path::Path;

use super::{LineKind, LinePass, LineRecord, read_sections, read_sections_with};
use crate::config::discovery::{load_config_at, pending_config};
use crate::config::run::{Invocation, Selection};
use crate::testing::{test_root, write};

thread_local! {
    static RECORDS: Cell<usize> = const { Cell::new(0) };
}

pub(super) fn record_created() {
    RECORDS.set(RECORDS.get() + 1);
}

fn captured_during<T>(read: impl FnOnce() -> T) -> (T, usize) {
    RECORDS.set(0);
    let result = read();
    (result, RECORDS.get())
}

const TEXT: &str = "# lead\n\n\
    [reference] # header\n\
    marker = \"#\" # marker\n\n\
    [[kinds]] # row\n\
    kind = \"FS\" # kind\n\
    folder = \"docs\"\n";

/// §FS-config-migrate.4.3: the real ordinary entrypoint must construct no migration
/// records, for either spelling of the version selector.
#[test]
fn ordinary_section_reads_construct_no_line_records() {
    for text in [
        TEXT.to_string(),
        format!("grund_config_version = 1\n{TEXT}"),
    ] {
        let (read, captured) = captured_during(|| read_sections(&text, Path::new("grund.toml")));
        let read = read.expect("the ordinary v1 file reads");
        assert_eq!(read.project.schema.citation.marker, "#");
        assert_eq!(captured, 0, "ordinary reads constructed migration records");
    }
}

/// §FS-config-migrate.4.3: an unsuccessful ordinary read does not capture either.
/// §FS-config.4.3: it still reports the first spelling error at its own line.
#[test]
fn ordinary_refusals_construct_no_line_records() {
    let (read, captured) = captured_during(|| {
        read_sections(
            "# comment\nunknown = true\n[not_a_section]\n",
            Path::new("grund.toml"),
        )
    });
    let error = read.err().expect("the unknown key is refused");
    assert_eq!(
        error.to_string(),
        "grund.toml:2: unknown config key `unknown`"
    );
    assert_eq!(
        captured, 0,
        "a refused ordinary read captured migration records"
    );
}

/// §FS-config-migrate.4.3: discovery's shared path (also used for workspace members,
/// `config show` and `config validate`) must use the non-capturing reader.
#[test]
fn discovered_config_constructs_no_line_records() {
    let root = test_root("config-capture-contract-discovered");
    write(&root.join("grund.toml"), TEXT);
    let (loaded, captured) = captured_during(|| load_config_at(&root, &root));
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(loaded.expect("the config loads").marker, "#");
    assert_eq!(captured, 0, "discovery captured migration records");
}

/// §FS-config-migrate.4.3: text that has not reached disk takes the same ordinary path.
#[test]
fn pending_config_constructs_no_line_records() {
    let root = test_root("config-capture-contract-pending");
    let invocation = Invocation::new(&root, true, Selection::Project);
    let (loaded, captured) = captured_during(|| pending_config(TEXT, &root, &invocation));
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(loaded.expect("the pending config loads").marker, "#");
    assert_eq!(
        captured, 0,
        "pending config text captured migration records"
    );
}

/// §FS-config-migrate.4.1, §FS-config-migrate.4.2, §FS-config-migrate.4.3:
/// migration keeps every raw line, 1-based location, section and comment,
/// including a hash inside a quoted value and comments on both header forms.
#[test]
fn migration_keeps_complete_line_records() {
    let mut pass = LinePass::tolerant();
    let (read, captured) =
        captured_during(|| read_sections_with(TEXT, Path::new("grund.toml"), &mut pass));
    read.expect("migration reads");
    assert_eq!(captured, TEXT.lines().count(), "the observer sees capture");
    let kinds = [
        LineKind::Comment,
        LineKind::Blank,
        LineKind::Header {
            name: "reference".into(),
            array: false,
            trailing: Some("# header".into()),
        },
        LineKind::Key {
            section: "reference".into(),
            key: "marker".into(),
            value: "\"#\"".into(),
            trailing: Some("# marker".into()),
        },
        LineKind::Blank,
        LineKind::Header {
            name: "kinds".into(),
            array: true,
            trailing: Some("# row".into()),
        },
        LineKind::Key {
            section: "kinds".into(),
            key: "kind".into(),
            value: "\"FS\"".into(),
            trailing: Some("# kind".into()),
        },
        LineKind::Key {
            section: "kinds".into(),
            key: "folder".into(),
            value: "\"docs\"".into(),
            trailing: None,
        },
    ];
    let expected: Vec<_> = TEXT
        .lines()
        .zip(kinds)
        .enumerate()
        .map(|(index, (raw, kind))| LineRecord {
            line: index + 1,
            raw: raw.into(),
            kind,
        })
        .collect();
    assert_eq!(pass.lines, expected);
}

/// §AR-config.4, §FS-config-migrate.4.3: capture preserves lowering and the first
/// spelling error. This guards behavior while the separate capture tests fail.
#[test]
fn capture_preserves_lowering_and_first_spelling_error() {
    let path = Path::new("grund.toml");
    let ordinary = read_sections(TEXT, path)
        .unwrap()
        .lower_kinds(path, Path::new("."))
        .unwrap();
    let captured = read_sections_with(TEXT, path, &mut LinePass::tolerant())
        .unwrap()
        .lower_kinds(path, Path::new("."))
        .unwrap();
    assert_eq!(ordinary, captured);
    let broken = "[reference]\nstrict = 7\n[unknown]\n";
    let ordinary = read_sections(broken, path).err().expect("invalid boolean");
    let captured = read_sections_with(broken, path, &mut LinePass::tolerant())
        .err()
        .expect("migration refuses the same invalid boolean");
    assert_eq!(ordinary.to_string(), captured.to_string());
    assert!(ordinary.to_string().starts_with("grund.toml:2:"));
}

/// §FS-config.3.5.16, §FS-config-migrate.3.1.4: only migration accepts a
/// path-shaped exclusion for diagnosis, and its line still carries the comment.
#[test]
fn migration_tolerance_does_not_relax_ordinary_exclusions() {
    let path = Path::new("grund.toml");
    let text = "[scan]\nexclude = [\"docs/plans\"] # diagnose me\n";
    let error = read_sections(text, path).err().expect("ordinary refusal");
    assert_eq!(
        error.to_string(),
        "grund.toml:2: [scan] exclude entry `docs/plans` contains `/`, not a directory name; \
         remove it to keep the scan unchanged, or use `plans` to exclude that name at any depth"
    );
    let mut pass = LinePass::tolerant();
    let read = read_sections_with(text, path, &mut pass).expect("migration diagnoses it later");
    assert_eq!(read.project.schema.sources.exclude, ["docs/plans"]);
    assert_eq!(pass.lines.len(), 2);
    assert_eq!(pass.lines[1].line, 2);
    assert!(matches!(
        &pass.lines[1].kind,
        LineKind::Key { trailing: Some(comment), .. } if comment == "# diagnose me"
    ));
}
