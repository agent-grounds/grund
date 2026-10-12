//! Black-box contract for the narrowed-path warning under `show --batch`
//! (§FS-scan.path.show, §FS-scan.path.warning.shape): each query whose answer the owning project
//! would change gets its own stderr line, in input order, and stdout and the exit code keep
//! today's bytes. The single-read, `list`, `id` and `check <dir>` forms are e2e cases.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const CLAUSE: &str = "; from grund 0.19.0 a path narrows what is reported, not what resolves";

struct Repo(PathBuf);

impl Repo {
    fn new(name: &str) -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/path-narrowing-warns")
            .join(format!("{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        write(
            &root.join("grund.toml"),
            "grund_config_version = 1\nproject_name = \"fixture\"\n\n\
             [reference]\nmarker = \"§\"\nstrict = true\n\n\
             [id]\nformat = \"{kind}-{number}-{slug}\"\n\n\
             [[kinds]]\nkind = \"FS\"\nfolder = \"specs\"\n",
        );
        write(
            &root.join("specs/a/fs-a.md"),
            "# FS-001-a: Alpha\n\nAlpha body.\n",
        );
        write(
            &root.join("specs/b/fs-b.md"),
            "# FS-002-b: Beta\n\nBeta body.\n",
        );
        write(
            &root.join("specs/b/dup.md"),
            "# FS-001-a: Alpha again\n\nThe second home.\n",
        );
        Self(root)
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write(path: &Path, body: &str) {
    fs::create_dir_all(path.parent().expect("fixture file has a parent")).expect("create dir");
    fs::write(path, body).unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
}

#[test]
fn batch_warns_once_per_query_the_project_would_answer_differently() {
    let repo = Repo::new("batch");
    let mut child = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(["show", "--batch", "specs/a", "--format=json"])
        .current_dir(&repo.0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn grund");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(b"{\"id\":\"FS-002-b\"}\n{\"id\":\"FS-009-z\"}\n{\"id\":\"FS-001-a\"}\n")
        .expect("write queries");
    let output = child.wait_with_output().expect("wait for grund");

    let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
    assert_eq!(output.status.code(), Some(1), "a failed query keeps exit 1");
    let codes: Vec<bool> = stdout
        .lines()
        .map(|line| line.contains("\"ok\":true"))
        .collect();
    assert_eq!(
        codes,
        [false, false, true],
        "stdout keeps today's three results: {stdout}"
    );
    assert!(
        stdout.contains("\"message\":\"ID not found: FS-002-b\""),
        "{stdout}"
    );
    assert!(stdout.contains("\"body\":\"Alpha body.\\n\""), "{stdout}");

    let stderr = String::from_utf8(output.stderr).expect("utf-8 stderr");
    assert_eq!(
        stderr,
        format!(
            "warning: FS-002-b is declared at specs/b/fs-b.md:1, outside specs/a{CLAUSE}\n\
             warning: FS-001-a is also declared at specs/b/dup.md:1, outside specs/a{CLAUSE}\n"
        ),
        "one line per moved query, in input order; none for FS-009-z, declared nowhere"
    );
}

#[test]
fn batch_warns_where_a_short_form_is_ambiguous_in_the_project() {
    let repo = Repo::new("batch-short-form");
    write(
        &repo.0.join("specs/a/fs-c.md"),
        "# FS-003-c: Gamma\n\nGamma body.\n",
    );
    write(
        &repo.0.join("specs/b/z.md"),
        "# FS-003-z: Zed\n\nZed body.\n",
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(["show", "--batch", "specs/a", "--format=json"])
        .current_dir(&repo.0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn grund");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(b"{\"id\":\"FS-003\"}\n{\"id\":\"FS-003-c\"}\n")
        .expect("write queries");
    let output = child.wait_with_output().expect("wait for grund");

    let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
    assert_eq!(output.status.code(), Some(0), "both queries resolve today");
    assert_eq!(
        stdout.matches("\"body\":\"Gamma body.\\n\"").count(),
        2,
        "{stdout}"
    );

    let stderr = String::from_utf8(output.stderr).expect("utf-8 stderr");
    assert_eq!(
        stderr,
        format!("warning: FS-003 is also declared at specs/b/z.md:1, outside specs/a{CLAUSE}\n"),
        "the short form gains the line, as written; the full ID resolves to one home"
    );
}
