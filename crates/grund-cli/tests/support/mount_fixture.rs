//! The fixture and the relocation the mount proof compares with (§REQ-remote-equivalence.4):
//! a tree `lib` with a nested member `core`, and two consumers beside it that import it as
//! `dep`. Only the declared relocation fields are normalized: a reported `path`, resolved
//! from the side's report base to the file it names, and the alias that names a project,
//! `lib` and `core` in the tree's own run and `dep` and `dep/core` through the import
//! (§FS-imports.mount, §FS-remote-projects.mount.rule).

use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const KINDS: &str = "[schema.kinds.FS]\nfolders = [\"docs/fs\"]\n";
const IMPORT: &str = "[workspace.imports.dep]\npath = \"../lib\"\nproject = \"lib\"\n";

pub struct Fixture(pub PathBuf);

impl Fixture {
    pub fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-mount-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let fixture = Self(root);
        let lib = format!("grund_config_version = 2\nproject_name = \"lib\"\n\n{KINDS}");
        fixture.write(
            "lib/grund.toml",
            &format!("{lib}\n[workspace]\nmembers = [\"core\"]\n"),
        );
        fixture.write(
            "lib/docs/fs/FS-x.md",
            "# FS-x: X\n\n## 1. One\n\nCites \u{a7}core/FS-y, \u{a7}FS-z and \u{a7}FS-x.1 and \u{a7}FS-gone.\n",
        );
        fixture.write("lib/docs/fs/FS-z.md", "# FS-z: Z\n\nCites \u{a7}FS-x.\n");
        fixture.write(
            "lib/core/grund.toml",
            &format!("grund_config_version = 2\nproject_name = \"core\"\n\n{KINDS}"),
        );
        fixture.write(
            "lib/core/docs/fs/FS-y.md",
            "# FS-y: Y\n\nCites \u{a7}FS-y and \u{a7}lib/FS-x.\n",
        );
        for consumer in ["quiet", "app"] {
            let config = format!("grund_config_version = 2\nproject_name = \"{consumer}\"\n\n");
            fixture.write(
                &format!("{consumer}/grund.toml"),
                &format!("{config}{KINDS}\n{IMPORT}"),
            );
        }
        fixture.write(
            "app/docs/notes.md",
            "See \u{a7}dep/FS-x.\nSee \u{a7}dep/core/FS-y.\nSee \u{a7}dep/FS-x.1.\nSee \u{a7}dep/FS-gone.\n",
        );
        fixture
    }

    pub fn write(&self, relative: &str, body: &str) {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().expect("parent")).expect("create fixture parent");
        fs::write(path, body).expect("write fixture");
    }

    pub fn dir(&self, side: &str) -> PathBuf {
        self.0.join(side).canonicalize().expect("canonical side")
    }

    pub fn grund(&self, side: &str, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .current_dir(self.0.join(side))
            .env("HOME", self.0.join("home"))
            .env("XDG_CONFIG_HOME", self.0.join("xdg"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CEILING_DIRECTORIES", &self.0)
            .args(args)
            .output()
            .expect("run grund")
    }

    /// Every file under `side` and its bytes.
    pub fn snapshot(&self, side: &str) -> Vec<(PathBuf, Vec<u8>)> {
        let mut files = Vec::new();
        let mut stack = vec![self.0.join(side)];
        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(&dir).expect("read fixture dir") {
                let path = entry.expect("fixture entry").path();
                if path.is_dir() {
                    stack.push(path);
                } else {
                    files.push((path.clone(), fs::read(&path).expect("read fixture file")));
                }
            }
        }
        files.sort();
        files
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// The alias a project of the tree's own run is shown under through the import.
pub fn shown(native: &str) -> String {
    match native {
        "lib" => "dep".to_string(),
        other => format!("dep/{other}"),
    }
}

/// A coordinate of the tree's own run, qualified as the consumer names it.
pub fn qualified(native: &str) -> String {
    match native.rsplit_once('/') {
        Some((namespace, id)) => format!("{}/{id}", shown(namespace)),
        None => format!("dep/{native}"),
    }
}

/// `value` with its relocation fields declared: each `path` resolved from `base` and,
/// on the tree's own side, each `project` shown under the import's alias and, where
/// `ids` holds, each qualified `id` too.
pub fn relocate(value: &mut Value, base: &Path, native: bool, ids: bool) {
    match value {
        Value::Object(map) => {
            for (key, field) in map.iter_mut() {
                match (key.as_str(), field) {
                    ("path", Value::String(path)) => {
                        let file = base.join(&*path);
                        *path = file.canonicalize().unwrap_or(file).display().to_string();
                    }
                    ("project", Value::String(project)) if native => *project = shown(project),
                    ("id", Value::String(id)) if native && ids => *id = qualified(id),
                    (_, nested) => relocate(nested, base, native, ids),
                }
            }
        }
        Value::Array(items) => items
            .iter_mut()
            .for_each(|item| relocate(item, base, native, ids)),
        _ => {}
    }
}

/// Each JSON line of `output` relocated, as a set: the order follows the relocated fields.
pub fn rows(output: &Output, base: &Path, native: bool, ids: bool) -> BTreeSet<String> {
    text(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let mut value: Value = serde_json::from_str(line).expect("a JSON line");
            relocate(&mut value, base, native, ids);
            value.to_string()
        })
        .collect()
}

/// The source run in `lib` with `source` against the consumer `quiet` with `projected`,
/// both JSON, relocated, and alike in exit code.
pub fn compare_json(fixture: &Fixture, source: &[&str], projected: &[&str], ids: bool) {
    let (native, through) = (
        fixture.grund("lib", source),
        fixture.grund("quiet", projected),
    );
    assert_eq!(
        native.status.code(),
        through.status.code(),
        "{source:?} against {projected:?}"
    );
    let native_rows = rows(&native, &fixture.dir("lib"), true, ids);
    let through_rows = rows(&through, &fixture.dir("quiet"), false, ids);
    assert!(!native_rows.is_empty(), "{source:?} answers something");
    assert_eq!(
        native_rows, through_rows,
        "{source:?} against {projected:?}"
    );
}

/// The source run against the consumer's, byte for byte: these carry no relocation field.
pub fn compare_bytes(fixture: &Fixture, source: &[&str], projected: &[&str]) {
    let (native, through) = (
        fixture.grund("lib", source),
        fixture.grund("quiet", projected),
    );
    assert_eq!(
        (
            native.status.code(),
            text(&native.stdout),
            text(&native.stderr)
        ),
        (
            through.status.code(),
            text(&through.stdout),
            text(&through.stderr)
        ),
        "{source:?} against {projected:?}"
    );
}
