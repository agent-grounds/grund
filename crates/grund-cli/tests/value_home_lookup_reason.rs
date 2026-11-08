//! Black-box regressions for §FS-fetch.4.3: a value home that is on disk but cannot be
//! looked up is refused with the lookup's reason, never as absent (§FS-fetch.4.1).
//! Directory modes cannot be committed, so the fixtures are built at run time.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};

const V1_HEAD: &str = "grund_config_version = 1\n\n\
    [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n\n";
const V2_HEAD: &str = "grund_config_version = 2\nproject_name = \"fixture\"\n\n\
    [schema.kinds.FS]\nfolders = [\"docs\"]\nindex = false\n\n";
const HINT: &str = "hint: make the home and every directory above it accessible";

#[derive(Clone, Copy)]
enum Version {
    V1,
    V2,
}

#[derive(Clone, Copy)]
enum Shape {
    File,
    Folder,
}

impl Shape {
    /// The home as the config spells it; it always lies under `data/`.
    fn home(self) -> &'static str {
        match self {
            Shape::File => "data/prices.json",
            Shape::Folder => "data/rates",
        }
    }
}

/// The config, and the 1-based line the refusal is located at: the key that makes RATE a
/// value kind, where the absent home's refusal stands too (§FS-fetch.4.1).
fn config(version: Version, shape: Shape) -> (String, usize) {
    let home = shape.home();
    let (head, body) = match (version, shape) {
        (Version::V1, Shape::File) => (
            V1_HEAD,
            format!("[[kinds]]\nkind = \"RATE\"\nfile = \"{home}\"\nvalues = true\n"),
        ),
        (Version::V1, Shape::Folder) => (
            V1_HEAD,
            format!(
                "[[kinds]]\nkind = \"RATE\"\nfolder = \"{home}\"\nindex = false\nvalues = true\n"
            ),
        ),
        (Version::V2, Shape::File) => (
            V2_HEAD,
            format!("[schema.kinds.RATE]\nfiles = [\"{home}\"]\nform = \"value\"\n"),
        ),
        (Version::V2, Shape::Folder) => (
            V2_HEAD,
            format!("[schema.kinds.RATE]\nfolders = [\"{home}\"]\nform = \"value\"\n"),
        ),
    };
    let text = format!("{head}{body}");
    let line = text
        .lines()
        .position(|l| l == "values = true" || l == "form = \"value\"")
        .expect("RATE is a value kind")
        + 1;
    (text, line)
}

struct Fixture {
    root: PathBuf,
    locked: Option<PathBuf>,
}

impl Fixture {
    /// A project whose value home is on disk and valid.
    fn new(name: &str, version: Version, shape: Shape) -> (Self, usize) {
        let root = std::env::temp_dir().join(format!("grund-1294-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::create_dir_all(root.join("data")).unwrap();
        fs::write(root.join("docs/FS-a.md"), "# FS-a: A\n\nPlain text.\n").unwrap();
        let (text, line) = config(version, shape);
        fs::write(root.join("grund.toml"), text).unwrap();
        match shape {
            Shape::File => fs::write(root.join("data/prices.json"), "{\"RATE-a\": 1}\n").unwrap(),
            Shape::Folder => {
                fs::create_dir(root.join("data/rates")).unwrap();
                fs::write(root.join("data/rates/a.json"), "{\"RATE-a\": 1}\n").unwrap();
            }
        }
        (Self { root, locked: None }, line)
    }

    /// Takes search permission off `data/`, the home's parent. False when the lookup still
    /// succeeds, as it does for a privileged user, so the caller skips.
    fn lock_parent(&mut self, shape: Shape) -> bool {
        let data = self.root.join("data");
        fs::set_permissions(&data, fs::Permissions::from_mode(0o444)).unwrap();
        self.locked = Some(data);
        let error = fs::canonicalize(self.root.join(shape.home())).err();
        match error {
            None => false,
            Some(error) => {
                assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
                true
            }
        }
    }

    fn list(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .arg("list")
            .current_dir(&self.root)
            .output()
            .expect("run grund")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(path) = &self.locked {
            let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o755));
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// §FS-fetch.4.3: exit 2 at the home's line, the reason and the access hint, and neither
/// the absent home's first line nor its `create the` repair.
fn assert_refused_with_reason(name: &str, version: Version, shape: Shape) {
    let (mut fixture, line) = Fixture::new(name, version, shape);
    let control = fixture.list();
    assert_eq!(
        control.status.code(),
        Some(0),
        "{name}: the fixture must load before the lock: {}",
        String::from_utf8_lossy(&control.stderr)
    );
    if !fixture.lock_parent(shape) {
        eprintln!("skipping {name}: a lookup under mode 444 succeeds for this user");
        return;
    }
    let output = fixture.list();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(2), "{name}: {stderr}");
    let home = shape.home();
    let first = format!(
        "error: grund.toml:{line}: could not look up value home for kind `RATE`: {home}: permission denied"
    );
    assert!(
        stderr.contains(&format!("{first}\n{HINT}\n")),
        "{name}: expected\n{first}\n{HINT}\ngot:\n{stderr}"
    );
    assert!(
        !stderr.contains("does not exist"),
        "{name}: says absent:\n{stderr}"
    );
    assert!(
        !stderr.contains("create the"),
        "{name}: says to create it:\n{stderr}"
    );
    assert!(
        !stderr.contains("os error"),
        "{name}: carries an OS error number:\n{stderr}"
    );
}

#[test]
fn v1_file_home_under_unsearchable_parent_names_the_reason() {
    assert_refused_with_reason("v1-file", Version::V1, Shape::File);
}

#[test]
fn v1_folder_home_under_unsearchable_parent_names_the_reason() {
    assert_refused_with_reason("v1-folder", Version::V1, Shape::Folder);
}

#[test]
fn v2_file_home_under_unsearchable_parent_names_the_reason() {
    assert_refused_with_reason("v2-file", Version::V2, Shape::File);
}

#[test]
fn v2_folder_home_under_unsearchable_parent_names_the_reason() {
    assert_refused_with_reason("v2-folder", Version::V2, Shape::Folder);
}

/// §FS-fetch.4.1: a home that really is absent keeps its refusal and hint, word for word.
#[test]
fn absent_home_keeps_the_absent_refusal() {
    for (name, version, shape, hint) in [
        (
            "v1-absent-file",
            Version::V1,
            Shape::File,
            "create the file",
        ),
        (
            "v2-absent-folder",
            Version::V2,
            Shape::Folder,
            "create the folder",
        ),
    ] {
        let (fixture, line) = Fixture::new(name, version, shape);
        let home = fixture.root.join(shape.home());
        match shape {
            Shape::File => fs::remove_file(&home).unwrap(),
            Shape::Folder => fs::remove_dir_all(&home).unwrap(),
        }
        let output = fixture.list();
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert_eq!(output.status.code(), Some(2), "{name}: {stderr}");
        let home = shape.home();
        let expected = format!(
            "error: grund.toml:{line}: value home for kind `RATE` does not exist: {home}\n\
             hint: a value kind's home must exist; {hint}\n"
        );
        assert!(
            stderr.contains(&expected),
            "{name}: expected\n{expected}got:\n{stderr}"
        );
    }
}
