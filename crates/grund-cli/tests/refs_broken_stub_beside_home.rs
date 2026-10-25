//! Black-box: a broken stub beside a real home does not make `refs` refuse; it lists every
//! citation of the ID, exits `0`, and names the stub it set aside in one stderr warning
//! (§FS-refs.4.1.1). Ports the #1006 reproducer: a module moved from `src/cache.rs` to
//! `src/store/cache.rs`, its doc-comment still declaring `AR-cache`, its stub left pointing at
//! the old file. Two real homes and `show` still refuse (§FS-refs.4, §FS-show.2.2.1).

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const CONFIG: &str = "grund_config_version = 2

[schema]
id_format = \"{kind}-{slug}\"

[schema.kinds.AR]
folders = [\"docs/architecture\"]
";

/// The stub as the move left it: its target no longer exists.
const BROKEN_STUB: &str = "# AR-cache: [`../../src/cache.rs`](../../src/cache.rs)\n";

/// The control: the same stub pointed at where the module went.
const REPAIRED_STUB: &str = "# AR-cache: [`../../src/store/cache.rs`](../../src/store/cache.rs)\n";

/// The reproducer's tree beside the stub, which each fixture writes itself.
const TREE: &[(&str, &str)] = &[
    (
        "docs/architecture/AR-auth.md",
        "# AR-auth: Authentication\n\nA login checks the password,\n\
         then stores the new session in the cache (\u{a7}AR-cache).\n",
    ),
    (
        "docs/architecture/README.md",
        "# Architecture\n\n| ID | Summary |\n|----|---------|\n\
         | [\u{a7}AR-auth](AR-auth.md) | Authentication |\n\
         | [\u{a7}AR-cache](AR-cache.md) | The session cache |\n",
    ),
    (
        "src/store/cache.rs",
        "/// AR-cache: Sessions are cached in process\n///\n/// ## 1. Expiry\n///\n\
         /// A session expires after an hour.\npub struct Cache;\n",
    ),
    (
        "src/session.rs",
        "//! Session handling (\u{a7}AR-auth).\n\npub struct Session;\n\nimpl Session {\n    \
         // Expired sessions leave the cache like any other (\u{a7}AR-cache.1)\n    \
         pub fn expire(&self) {}\n}\n",
    ),
];

const SITES: &str = "docs/architecture/AR-auth.md:4: \u{a7}AR-cache
docs/architecture/README.md:6: \u{a7}AR-cache (index entry)
src/session.rs:6: \u{a7}AR-cache.1
";

const SUMMARY: &str = "docs/architecture/AR-auth.md: 1 (line 4)
docs/architecture/README.md: 1 (line 6) (index entries at line 6)
src/session.rs: 1 (line 6)
";

const TOTAL: &str = "cited at 3 sites across 3 files\n";

/// The one line the set-aside stub earns: `warning: ` and check's own `broken stub:` clause.
const WARNING: &str = "warning: broken stub: docs/architecture/AR-cache.md:1 links to missing \
                       ../../src/cache.rs; AR-cache is declared at src/store/cache.rs:1; point \
                       the stub link there\n";

/// A file left at the old path that declares no `AR-cache`: the stub's target exists and lacks
/// the ID. The refusal carries no `broken stub:` clause for that verdict, so the warning is the
/// line `show` refuses the stub alone with (§FS-show.2.3.4).
const LEFT_AT_OLD_PATH: &str = "//! What is left at the old path re-exports the moved cache.\n\
                                pub use crate::store::cache::Cache;\n";

const LACKS_ID_WARNING: &str = "warning: broken stub: AR-cache (stub at \
                                docs/architecture/AR-cache.md:1 points at ../../src/cache.rs, \
                                which contains no inline declaration of AR-cache)\n";

struct Fixture(PathBuf);

impl Fixture {
    /// A Git tree at a fresh temp path holding the reproducer's tree with `stub` as the stub.
    fn new(name: &str, stub: &str) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("grund-1006-{name}-{}-{serial}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create fixture");
        let git = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&dir)
            .output()
            .expect("initialize fixture Git repository");
        assert!(git.status.success(), "git init: {}", text(&git.stderr));
        let fixture = Self(dir);
        fixture.write("grund.toml", CONFIG);
        fixture.write("docs/architecture/AR-cache.md", stub);
        for (path, contents) in TREE {
            fixture.write(path, contents);
        }
        fixture
    }

    fn write(&self, path: &str, contents: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    fn grund(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .expect("run grund")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// Asserts `out` is exit `0` with exactly `stdout` and `stderr`.
fn assert_lists(out: &Output, stdout: &str, stderr: &str) {
    assert_eq!(
        (
            out.status.code(),
            text(&out.stdout).as_str(),
            text(&out.stderr).as_str()
        ),
        (Some(0), stdout, stderr),
    );
}

#[test]
fn control_repaired_stub_lists_the_three_sites() {
    let fixture = Fixture::new("control", REPAIRED_STUB);
    assert_lists(&fixture.grund(&["refs", "AR-cache"]), SITES, "");
    assert_lists(
        &fixture.grund(&["refs", "AR-cache", "--summary"]),
        SUMMARY,
        "",
    );
    assert_lists(&fixture.grund(&["refs", "AR-cache", "--total"]), TOTAL, "");
}

#[test]
fn refs_lists_past_a_broken_stub_beside_a_real_home() {
    let fixture = Fixture::new("text", BROKEN_STUB);
    assert_lists(&fixture.grund(&["refs", "AR-cache"]), SITES, WARNING);
}

#[test]
fn refs_summary_lists_past_a_broken_stub_beside_a_real_home() {
    let fixture = Fixture::new("summary", BROKEN_STUB);
    assert_lists(
        &fixture.grund(&["refs", "AR-cache", "--summary"]),
        SUMMARY,
        WARNING,
    );
}

#[test]
fn refs_total_counts_past_a_broken_stub_beside_a_real_home() {
    let fixture = Fixture::new("total", BROKEN_STUB);
    assert_lists(
        &fixture.grund(&["refs", "AR-cache", "--total"]),
        TOTAL,
        WARNING,
    );
}

#[test]
fn refs_json_gives_the_repaired_trees_records_and_an_empty_stderr() {
    let control = Fixture::new("json-control", REPAIRED_STUB);
    let expected = control.grund(&["refs", "AR-cache", "--format", "json"]);
    assert_lists(&expected, &text(&expected.stdout), "");
    let fixture = Fixture::new("json", BROKEN_STUB);
    let out = fixture.grund(&["refs", "AR-cache", "--format", "json"]);
    assert_lists(&out, &text(&expected.stdout), "");
}

/// Every stub that cannot be paired is set aside, not only one whose target is missing
/// (§FS-refs.4.1.1): here the old path holds a file that declares no `AR-cache`.
#[test]
fn refs_lists_past_a_stub_whose_target_lacks_the_id_beside_a_real_home() {
    let fixture = Fixture::new("lacks-id", BROKEN_STUB);
    fixture.write("src/cache.rs", LEFT_AT_OLD_PATH);
    assert_lists(
        &fixture.grund(&["refs", "AR-cache"]),
        SITES,
        LACKS_ID_WARNING,
    );
}

/// Several broken stubs with no real home beside them set nothing aside: `refs` lists the sites
/// and warns of none (§FS-refs.4.1.1).
#[test]
fn refs_lists_past_two_broken_stubs_with_no_real_home_and_warns_of_none() {
    let fixture = Fixture::new("two-broken", BROKEN_STUB);
    fs::remove_file(fixture.0.join("src/store/cache.rs")).unwrap();
    fixture.write(
        "docs/architecture/AR-cache-old.md",
        "# AR-cache: [`../../src/old.rs`](../../src/old.rs)\n",
    );
    assert_lists(&fixture.grund(&["refs", "AR-cache"]), SITES, "");
}

#[test]
fn show_still_refuses_the_broken_stub_beside_a_real_home() {
    let fixture = Fixture::new("show", BROKEN_STUB);
    let out = fixture.grund(&["AR-cache"]);
    let stderr = text(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "stderr: {stderr}");
    assert!(out.stdout.is_empty(), "stdout: {}", text(&out.stdout));
    assert!(
        stderr.starts_with("ambiguous ID: AR-cache (declared at ")
            && stderr.contains("broken stub: "),
        "stderr: {stderr}",
    );
}

#[test]
fn refs_still_refuses_two_real_homes_beside_a_broken_stub() {
    let fixture = Fixture::new("two-homes", BROKEN_STUB);
    fixture.write(
        "src/cache_copy.rs",
        "/// AR-cache: A second, independent cache\npub struct CacheCopy;\n",
    );
    let out = fixture.grund(&["refs", "AR-cache"]);
    let stderr = text(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "stderr: {stderr}");
    assert!(out.stdout.is_empty(), "stdout: {}", text(&out.stdout));
    assert!(
        stderr.starts_with("ambiguous ID: AR-cache (declared at "),
        "stderr: {stderr}"
    );
}
