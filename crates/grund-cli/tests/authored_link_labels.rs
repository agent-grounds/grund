//! §FS-citations.wrap.enclosing — a citation inside a link its author wrote is
//! left with its link by `fmt` and taken as the index entry by `check`, in every
//! configuration that reaches it: version 1 and version 2, the index carve-out
//! under `enabled = false` and in a suppressed scope (§FS-fmt.6.4), and the forced
//! `--cross-refs`. The e2e cases of grund#825 sit in `tests/e2e/cases-red-825/`
//! until the fix lands and moves them into `tests/e2e/cases/`.
//!
//! These run the real binary, because the defect is a write: `check`, then
//! `fmt --write`, then both again, on the bytes a user would hold.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[path = "support/case_runner.rs"]
#[allow(dead_code)]
mod case_runner;

use case_runner::{CaseKind, assert_every_case_passed, run_case};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run_cases(label: &str, dir: &str, names: &[&str]) {
    let root = manifest_dir();
    let outcomes: Vec<_> = names
        .iter()
        .map(|name| run_case(&root, &root.join(dir).join(name), CaseKind::E2e))
        .collect();
    assert_every_case_passed(label, &outcomes);
}

fn grund(args: &[&str], cwd: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("spawn grund")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// A fresh tree holding `files`, each `(path, text)`.
fn tree(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = manifest_dir()
        .join("target/authored-link-labels")
        .join(name);
    let _ = fs::remove_dir_all(&dir);
    for (path, text) in files {
        let file = dir.join(path);
        fs::create_dir_all(file.parent().expect("a parent")).expect("create parent");
        fs::write(file, text).expect("write fixture file");
    }
    dir
}

fn read(dir: &Path, path: &str) -> String {
    fs::read_to_string(dir.join(path)).expect("read fixture file")
}

const V2_TOML: &str = "grund_config_version = 2\nproject_name = \"records\"\n\n\
    [schema.sources]\nlanguages = [\"markdown\", \"rust\"]\n\n\
    [schema.kinds.ADR]\nfolders = [\"docs/adr\"]\n";
const POSTGRES: &str = "# ADR-use-postgres: Use Postgres for all data\n\nWe use Postgres.\n";
const REDIS: &str = "# ADR-use-redis: Use Redis for the cache\n\nWe use Redis.\n";
const V2_SRC: &str = "// Stores data in Postgres (§ADR-use-postgres) and caches in Redis (§ADR-use-redis).\n\
    pub fn store() {}\n";
const LABEL: &str = "- [§ADR-use-postgres: Use Postgres for all data](use-postgres.md)\n";

/// The v1 spelling of the same project: numbered IDs, a folder kind with its
/// default `README.md` index, and `extra` appended to the config.
fn v1_toml(extra: &str) -> String {
    format!(
        "grund_config_version = 1\n\n{extra}\
         [[kinds]]\nkind = \"ADR\"\nfolder = \"docs/adr\"\n\n\
         [scan]\ninclude = [\"docs\", \"src\"]\n"
    )
}
const V1_POSTGRES: &str = "# ADR-001-postgres: Use Postgres for all data\n\nWe use Postgres.\n";
const V1_REDIS: &str = "# ADR-002-redis: Use Redis for the cache\n\nWe use Redis.\n";
const V1_SRC: &str =
    "// Postgres (§ADR-001-postgres), Redis (§ADR-002-redis).\npub fn store() {}\n";
const V1_LABEL: &str = "- [§ADR-001-postgres: Use Postgres for all data](ADR-001-postgres.md)\n";

/// Asserts `check` is clean, `fmt --write` writes nothing, `check` is still clean,
/// and `fmt --check` lists nothing — and that the named files kept their bytes.
fn assert_round_trip_leaves(dir: &Path, files: &[(&str, &str)]) {
    let first = grund(&["check"], dir);
    assert!(
        first.status.success() && !stdout(&first).contains("is not a link"),
        "the author's link is the entry: {}",
        stdout(&first)
    );
    let write = grund(&["fmt", "--write"], dir);
    assert_eq!(
        stdout(&write),
        "rewrote 0 lines\n",
        "fmt --write leaves the author's link"
    );
    for (path, text) in files {
        assert_eq!(&read(dir, path), text, "{path} keeps every byte");
    }
    let second = grund(&["check"], dir);
    assert!(second.status.success(), "{}", stdout(&second));
    let dry = grund(&["fmt", "--check"], dir);
    assert!(
        dry.status.success() && stdout(&dry).is_empty(),
        "fmt --check lists nothing: {}",
        stdout(&dry)
    );
}

/// §FS-citations.wrap.enclosing, §FS-check.3.17.4, §FS-fmt.6.4: the reproducer of
/// grund#825, ported to golden cases — `check` passes with no "is not a link",
/// `fmt --write` writes nothing, `fmt --check` is clean, and a dangling or
/// missing-section citation in a label keeps its usual finding.
#[test]
#[ignore = "red until #825 leaves a citation in an enclosing link as written and takes it as the entry; implement removes this"]
fn authored_link_label_cases() {
    run_cases(
        "authored_link_label_cases",
        "tests/e2e/cases-red-825",
        &[
            "check-index-authored-link-label-is-an-entry",
            "fmt-check-authored-link-label-left-as-written",
            "fmt-write-authored-link-label-left-as-written",
            "check-index-authored-link-label-dangling-and-missing-section",
        ],
    );
}

/// §FS-citations.wrap.enclosing.images, §FS-citations.wrap.enclosing.wrap-wins,
/// §FS-check.3.18.5: what must not move — an image encloses nothing, a nested line
/// is a fixed point, and the missing-entry message keeps both its forms' bytes.
#[test]
fn authored_link_label_guard_cases() {
    run_cases(
        "authored_link_label_guard_cases",
        "tests/e2e/cases",
        &[
            "fmt-write-image-alt-is-not-an-enclosing-link",
            "check-index-image-is-not-an-entry",
            "fmt-write-already-nested-line-is-a-fixed-point",
            "fmt-check-already-nested-line-is-a-fixed-point",
            "check-index-mention-in-table-is-not-an-entry",
            "check-index-link-destination-is-not-an-entry",
        ],
    );
}

/// §FS-citations.wrap.enclosing: the same round trip on a version 1 config.
#[test]
#[ignore = "red until #825 leaves a citation in an enclosing link as written and takes it as the entry; implement removes this"]
fn a_v1_round_trip_leaves_authored_labels() {
    let notes = "See [the §ADR-001-postgres record](adr/ADR-001-postgres.md).\n";
    let index =
        format!("{V1_LABEL}- [§ADR-002-redis: Use Redis for the cache](ADR-002-redis.md)\n");
    let dir = tree(
        "a_v1_round_trip_leaves_authored_labels",
        &[
            ("grund.toml", &v1_toml("")),
            ("docs/adr/ADR-001-postgres.md", V1_POSTGRES),
            ("docs/adr/ADR-002-redis.md", V1_REDIS),
            ("docs/adr/README.md", &index),
            ("docs/notes.md", notes),
            ("src/lib.rs", V1_SRC),
        ],
    );
    assert_round_trip_leaves(
        &dir,
        &[("docs/adr/README.md", &index), ("docs/notes.md", notes)],
    );
}

/// §FS-fmt.6.4 over §FS-fmt.6.1.1: under `enabled = false` the index carve-out
/// still wraps a bare entry, and leaves the author's link as written.
#[test]
#[ignore = "red until #825 lets the enclosing link outrank the index carve-out; implement removes this"]
fn the_disabled_carve_out_wraps_a_bare_entry_and_leaves_an_authored_label() {
    let dir = tree(
        "the_disabled_carve_out_wraps_a_bare_entry_and_leaves_an_authored_label",
        &[
            (
                "grund.toml",
                &v1_toml("[fmt.cross_refs]\nenabled = false\n\n"),
            ),
            ("docs/adr/ADR-001-postgres.md", V1_POSTGRES),
            ("docs/adr/ADR-002-redis.md", V1_REDIS),
            (
                "docs/adr/README.md",
                &format!("{V1_LABEL}- §ADR-002-redis: Use Redis for the cache\n"),
            ),
            ("src/lib.rs", V1_SRC),
        ],
    );
    grund(&["fmt", "--write"], &dir);
    assert_eq!(
        read(&dir, "docs/adr/README.md"),
        format!(
            "{V1_LABEL}- [§ADR-002-redis](ADR-002-redis.md#adr-002-redis-use-redis-for-the-cache): \
             Use Redis for the cache\n"
        )
    );
}

/// §FS-fmt.6.4 over §FS-fmt.2.5.3: in a suppressed scope, the `[fmt] exclude`
/// file and the `grund:fmt off` region alike, the carve-out still wraps a bare
/// entry and leaves the author's link as written.
#[test]
#[ignore = "red until #825 lets the enclosing link outrank the index carve-out; implement removes this"]
fn a_suppressed_index_wraps_a_bare_entry_and_leaves_an_authored_label() {
    let bare = "- §ADR-use-redis: Use Redis for the cache\n";
    let wrapped = "- [§ADR-use-redis](use-redis.md#adr-use-redis-use-redis-for-the-cache): \
                   Use Redis for the cache\n";
    let excluded = format!("{V2_TOML}\n[presentation.fmt]\nexclude = [\"docs/adr/README.md\"]\n");
    for (name, toml, before, after) in [
        (
            "excluded_file",
            excluded.as_str(),
            format!("{LABEL}{bare}"),
            format!("{LABEL}{wrapped}"),
        ),
        (
            "off_region",
            V2_TOML,
            format!("<!-- grund:fmt off -->\n{LABEL}{bare}<!-- grund:fmt on -->\n"),
            format!("<!-- grund:fmt off -->\n{LABEL}{wrapped}<!-- grund:fmt on -->\n"),
        ),
    ] {
        let dir = tree(
            &format!("a_suppressed_index_wraps_a_bare_entry_and_leaves_an_authored_label_{name}"),
            &[
                ("grund.toml", toml),
                ("docs/adr/use-postgres.md", POSTGRES),
                ("docs/adr/use-redis.md", REDIS),
                ("docs/adr/README.md", &before),
                ("src/lib.rs", V2_SRC),
            ],
        );
        grund(&["fmt", "--write"], &dir);
        assert_eq!(read(&dir, "docs/adr/README.md"), after, "{name}");
    }
}

/// §FS-citations.wrap.enclosing: the forced `--cross-refs` leaves the author's
/// link as the default pass does.
#[test]
#[ignore = "red until #825 leaves a citation in an enclosing link as written; implement removes this"]
fn the_forced_cross_refs_pass_leaves_an_authored_label() {
    let notes = "See [the §ADR-001-postgres record](adr/ADR-001-postgres.md).\n";
    let dir = tree(
        "the_forced_cross_refs_pass_leaves_an_authored_label",
        &[
            (
                "grund.toml",
                &v1_toml("[fmt.cross_refs]\nenabled = false\n\n"),
            ),
            ("docs/adr/ADR-001-postgres.md", V1_POSTGRES),
            ("docs/adr/ADR-002-redis.md", V1_REDIS),
            (
                "docs/adr/README.md",
                &format!("{V1_LABEL}- [§ADR-002-redis: Redis](ADR-002-redis.md)\n"),
            ),
            ("docs/notes.md", notes),
            ("src/lib.rs", V1_SRC),
        ],
    );
    grund(&["fmt", "--cross-refs", "--write"], &dir);
    assert_eq!(read(&dir, "docs/notes.md"), notes);
}

/// §FS-citations.wrap.enclosing: only the link is protected, never the token in it,
/// so off strict mode `--marker` still marks the citation inside the author's link,
/// and the link pass then leaves that link alone.
#[test]
#[ignore = "red until #825 leaves a citation in an enclosing link as written; implement removes this"]
fn the_marker_pass_still_marks_a_citation_in_an_authored_label() {
    let dir = tree(
        "the_marker_pass_still_marks_a_citation_in_an_authored_label",
        &[
            ("grund.toml", &v1_toml("[reference]\nstrict = false\n\n")),
            ("docs/adr/ADR-001-postgres.md", V1_POSTGRES),
            ("docs/adr/ADR-002-redis.md", V1_REDIS),
            (
                "docs/adr/README.md",
                &format!("{V1_LABEL}- [§ADR-002-redis: Redis](ADR-002-redis.md)\n"),
            ),
            (
                "docs/notes.md",
                "See [the ADR-001-postgres record](adr/ADR-001-postgres.md).\n",
            ),
            ("src/lib.rs", V1_SRC),
        ],
    );
    grund(&["fmt", "--marker", "--write"], &dir);
    assert_eq!(
        read(&dir, "docs/notes.md"),
        "See [the §ADR-001-postgres record](adr/ADR-001-postgres.md).\n"
    );
}

/// §FS-check.3.18.3: a link the author aimed at a source declaration's home never
/// enrolls it, so `fmt --write` does not turn it into one that does.
#[test]
#[ignore = "red until #825 leaves a citation in an enclosing link as written; implement removes this"]
fn an_authored_label_to_a_source_home_does_not_enroll_it() {
    let index = format!("{LABEL}- [§ADR-inline: the inline record](../../src/lib.rs)\n");
    let dir = tree(
        "an_authored_label_to_a_source_home_does_not_enroll_it",
        &[
            ("grund.toml", V2_TOML),
            ("docs/adr/use-postgres.md", POSTGRES),
            ("docs/adr/README.md", &index),
            (
                "src/lib.rs",
                "/// ADR-inline: Keep the inline record\n///\n/// Stores in Postgres (§ADR-use-postgres).\n\
                 pub fn store() {}\n",
            ),
        ],
    );
    grund(&["fmt", "--write"], &dir);
    assert_eq!(
        read(&dir, "docs/adr/README.md"),
        index,
        "the author's link keeps its bytes"
    );
    let check = grund(&["check"], &dir);
    assert!(
        !stdout(&check).contains("declared but never cited: ADR-inline"),
        "the index row stays an ordinary citation, not an enrollment: {}",
        stdout(&check)
    );
}

/// §FS-citations.wrap.enclosing, §FS-check.3.18.5: an enclosing link with an empty
/// destination is left by `fmt` and is no entry, so the ID is missing from the index,
/// in the bytes that finding has shipped in.
#[test]
#[ignore = "red until #825 reads an empty-destination enclosing link as no entry; implement removes this"]
fn an_empty_destination_label_is_a_missing_entry() {
    let index = "- [§ADR-use-postgres: Use Postgres for all data]()\n";
    let dir = tree(
        "an_empty_destination_label_is_a_missing_entry",
        &[
            ("grund.toml", V2_TOML),
            ("docs/adr/use-postgres.md", POSTGRES),
            ("docs/adr/README.md", index),
            (
                "src/lib.rs",
                "// Stores data in Postgres (§ADR-use-postgres).\npub fn store() {}\n",
            ),
        ],
    );
    let check = grund(&["check"], &dir);
    assert_eq!(
        stdout(&check),
        "docs/adr/use-postgres.md:1: error: ADR-use-postgres appears in docs/adr/README.md but not \
         as an entry: an entry is a `§`-marked Markdown link to the declaration — became an error in \
         grund 0.13.0\n"
    );
    let write = grund(&["fmt", "--write"], &dir);
    assert_eq!(stdout(&write), "rewrote 0 lines\n");
    assert_eq!(read(&dir, "docs/adr/README.md"), index);
}

/// §FS-show.3.2.1: `show` collapses only a wrap — the author's link is printed as
/// written, and on a nested line only the inner wrap collapses.
#[test]
fn show_prints_an_authored_label_as_written() {
    let canon = "use-postgres.md#adr-use-postgres-use-postgres-for-all-data";
    let redis = format!(
        "# ADR-use-redis: Use Redis for the cache\n\n\
         We follow [the §ADR-use-postgres record](use-postgres.md).\n\n\
         Also [§ADR-use-postgres]({canon}).\n\n\
         And [[§ADR-use-postgres]({canon}): Use Postgres for all data](use-postgres.md).\n"
    );
    let dir = tree(
        "show_prints_an_authored_label_as_written",
        &[
            ("grund.toml", V2_TOML),
            ("docs/adr/use-postgres.md", POSTGRES),
            ("docs/adr/use-redis.md", &redis),
            ("src/lib.rs", V2_SRC),
        ],
    );
    let shown = stdout(&grund(&["ADR-use-redis"], &dir));
    assert!(
        shown.contains("We follow [the §ADR-use-postgres record](use-postgres.md).")
            && shown.contains("Also §ADR-use-postgres.")
            && shown
                .contains("And [§ADR-use-postgres: Use Postgres for all data](use-postgres.md)."),
        "{shown}"
    );
}
