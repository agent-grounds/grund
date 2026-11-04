//! Black-box: a version-2 `[schema.kinds.<K>] index` is held to the `.md` name test
//! (§FS-config.3.4.2.3.2), and its refusal says it is the name that failed, in version 1's
//! form (§FS-config.3.4.2.3.1). Ports grund.200's reproducer (#1184): an `index.txt` under
//! `txt = "markdown"` stays refused with exit 2, and its message no longer reads as a claim
//! about a file this config reads as Markdown; `index.md` still loads. The cross-reference
//! pass is why the name test stopped being a statement about Markdown in version 2: there it
//! follows the language, not the suffix (§FS-fmt.6.1).

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The report's sources, reading `.txt` as Markdown, with `row` as the FS row's places.
fn config(row: &str) -> String {
    format!(
        "grund_config_version = 2\nproject_name = \"g200\"\n\n\
         [schema.sources]\nlanguages = [\"markdown\"]\n\n\
         [schema.sources.extensions]\ntxt = \"markdown\"\n\n\
         [schema.kinds.FS]\n{row}\n"
    )
}

/// The report's config with `index` set to `index`; the `index` key is on line 12.
fn index_config(index: &str) -> String {
    config(&format!("folders = [\"fs\"]\nindex = \"{index}\""))
}

struct Fixture(PathBuf);

impl Fixture {
    fn new(config: &str, files: &[(&str, &str)]) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("grund-1184-{}-{serial}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        for (path, contents) in [("grund.toml", config)].iter().chain(files) {
            let path = dir.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, contents).unwrap();
        }
        Self(dir)
    }

    fn run(&self, args: &[&str]) -> (Option<i32>, String, String) {
        let out = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .expect("run grund");
        (
            out.status.code(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    }

    fn read(&self, path: &str) -> String {
        fs::read_to_string(self.0.join(path)).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const ALPHA: &str = "# FS-a: Alpha\n\nAlpha body.\n";
const CITES_BETA: &str = "# FS-a: Alpha\n\nAlpha cites \u{a7}FS-b.\n";
const CITES_ALPHA: &str = "# FS-b: Beta\n\nBeta cites \u{a7}FS-a.\n";

/// §FS-config.3.4.2.3.1: the report's fixture is refused at the key's line, exit 2, by a
/// message that names the value and the `.md` suffix it lacks, though `[schema.sources]`
/// reads `.txt` as Markdown; `check` and `config show` exit 2, `config validate` 1.
#[test]
fn a_v2_index_failing_the_name_test_is_refused_naming_the_name() {
    let fixture = Fixture::new(
        &index_config("index.txt"),
        &[
            ("fs/a.txt", ALPHA),
            ("fs/index.txt", "# FS specs\n\n- `a.txt#fs-a-alpha`\n"),
        ],
    );
    let refusal = "error: grund.toml:12: `[schema.kinds.FS] index` must name a Markdown file \
                   (`index.txt` is not a `.md` file)\n";
    for (args, code) in [
        (&["check"][..], 2),
        (&["config", "show"][..], 2),
        (&["config", "validate"][..], 1),
    ] {
        assert_eq!(
            fixture.run(args),
            (Some(code), String::new(), refusal.to_string()),
            "grund {}",
            args.join(" ")
        );
    }
}

/// §FS-config.3.4.2.3.2: the name test is unchanged, so an index named `.md` under the same
/// mapping still loads and its entry satisfies the kind's index.
#[test]
fn a_v2_index_passing_the_name_test_still_loads() {
    let fixture = Fixture::new(
        &index_config("index.md"),
        &[
            ("fs/a.txt", ALPHA),
            (
                "fs/index.md",
                "# FS specs\n\n- [\u{a7}FS-a](a.txt#fs-a-alpha)\n",
            ),
        ],
    );
    let (code, stdout, stderr) = fixture.run(&["check"]);
    assert_eq!(code, Some(0), "stdout:\n{stdout}\nstderr:\n{stderr}");
}

/// §FS-fmt.6.1: version 2's pass takes a file by the language `[schema.sources]` gives it,
/// so under `txt = "markdown"` it wraps a citation in a `.txt` file.
#[test]
fn the_v2_cross_reference_pass_wraps_a_file_read_as_markdown_whatever_its_name() {
    let fixture = Fixture::new(
        &config("files = [\"fs/a.txt\", \"fs/b.txt\"]"),
        &[("fs/a.txt", CITES_BETA), ("fs/b.txt", CITES_ALPHA)],
    );
    let (code, stdout, stderr) = fixture.run(&["fmt", "--write"]);
    assert_eq!(code, Some(0), "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert!(
        fixture.read("fs/a.txt").contains("(b.txt#fs-b-beta)"),
        "fs/a.txt:\n{}",
        fixture.read("fs/a.txt")
    );
}

/// §FS-fmt.6.1: version 1's pass goes by the `.md` suffix, so it wraps the citation in the
/// `.md` file and leaves the one in a scanned `.txt` file bare. Its link has no anchor, because
/// the `.txt` it points at is no Markdown home (§FS-fmt.6.2).
#[test]
fn the_v1_cross_reference_pass_wraps_only_md_files() {
    let fixture = Fixture::new(
        "grund_config_version = 1\n\n\
         [[kinds]]\nkind = \"FS\"\nfolder = \"fs\"\nindex = false\n\n\
         [scan]\ninclude = [\"fs\"]\nextensions = [\"md\", \"txt\"]\n",
        &[("fs/a.txt", CITES_BETA), ("fs/b.md", CITES_ALPHA)],
    );
    let (code, stdout, stderr) = fixture.run(&["fmt", "--write"]);
    assert_eq!(code, Some(0), "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert_eq!(fixture.read("fs/a.txt"), CITES_BETA);
    assert!(
        fixture.read("fs/b.md").contains("(a.txt)"),
        "fs/b.md:\n{}",
        fixture.read("fs/b.md")
    );
}
