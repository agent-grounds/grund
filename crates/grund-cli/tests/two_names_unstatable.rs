//! Black-box: a value source that exists and cannot be stated fails once, whichever names reach it
//! (§FS-scan.identity.buffers, §FS-values.5.3.1). A directory listed but not searched is the tree:
//! the whole-path `canonicalize` is denied there, so only the prefix walk gives two names one key.
//! Git cannot retain a directory's mode, so each tree is built here and skips where it does not
//! bar this process, which supplies no evidence for the pin.
#![cfg(unix)]

#[path = "support/two_names.rs"]
#[allow(dead_code)]
mod two_names;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use two_names::{Run, Tree};

const REMEDY: &str = "make the path readable, or keep it out of the scan by moving it out of the \
    value home, because exclude does not reach a value source";

/// Version 1, because version 2 refuses two places with one target at load
/// (§FS-config-v2.schema.places.boundaries.3).
const TWO_HOMES: &str = "grund_config_version = 1\n\
    [[kinds]]\nkind = \"CONST\"\nfolder = \"values\"\nindex = false\nvalues = true\n\
    [[kinds]]\nkind = \"OTHER\"\nfolder = \"alias\"\nindex = false\nvalues = true\n";

const TXT_IS_RUST: &str = "grund_config_version = 2\nproject_name = \"names\"\n\
    [schema.sources]\nlanguages = [\"markdown\", \"rust\"]\nexclude = [\"data\"]\n\
    [schema.sources.extensions]\ntxt = \"rust\"\n\
    [schema.kinds.CONST]\nfolders = [\"values\"]\nform = \"value\"\nindex = false\n";

/// A directory at mode 0444 until this drops, so a failing test leaves a tree that can be deleted.
/// Declared after its `Tree`, it restores the mode before the tree is removed.
struct Barred(PathBuf);

impl Barred {
    fn new(directory: PathBuf) -> Self {
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o444)).expect("bar the search");
        Self(directory)
    }
}

impl Drop for Barred {
    fn drop(&mut self) {
        let _ = fs::set_permissions(&self.0, fs::Permissions::from_mode(0o755));
    }
}

/// Whether stating `file` is really denied, which running as root is not.
fn stating_is_denied(file: &Path) -> bool {
    let denied =
        fs::metadata(file).is_err_and(|error| error.kind() == std::io::ErrorKind::PermissionDenied);
    if !denied {
        eprintln!(
            "skipped: stating {} was not denied, as when running as root",
            file.display()
        );
    }
    denied
}

/// `list` and `check` each exit 2 and print exactly this one failure line.
fn each_command_fails_once(tree: &Tree, expected: &str) {
    for command in ["list", "check"] {
        let run: Run = tree.grund(&[command]);
        let failures: Vec<&str> = run
            .stderr
            .lines()
            .filter(|line| line.starts_with("error: "))
            .collect();
        assert_eq!(failures, [expected], "grund {command}: {}", run.stderr);
        assert_eq!(run.code, 2, "grund {command}: {}", run.stderr);
    }
}

/// §FS-scan.identity.buffers: spellings of one location share one key, so the failure §FS-values.5.3.1
/// reports for one candidate is not reported again under its other name.
#[test]
fn a_source_two_homes_reach_that_cannot_be_stated_fails_once() {
    let tree = Tree::new("unstatable-two-homes", TWO_HOMES);
    tree.put("values/prices.json", "{\"CONST-a\": [\"a price\"]}\n");
    tree.link("alias", "values");
    let _barred = Barred::new(tree.root.join("values"));
    if !stating_is_denied(&tree.root.join("values/prices.json")) {
        return;
    }
    each_command_fails_once(
        &tree,
        &format!(
            "error: values/prices.json: could not read value JSON source: permission denied; {REMEDY}"
        ),
    );
}

/// §FS-values.5.3.1: a failed status is never a determination that the file is no source, and entry
/// stays on the spelling (§FS-scan.identity.reading.entry), so a listed link whose target's own
/// name would read as no value source still fails, named by the format its own name claims. `data`
/// is out of the walk, whose own line would otherwise report `data/x.txt` at that name as well.
#[test]
fn a_listed_link_whose_target_cannot_be_stated_is_not_dropped() {
    let tree = Tree::new("unstatable-link-target", TXT_IS_RUST);
    tree.put("data/x.txt", "{\"CONST-a\": [\"a price\"]}\n");
    tree.link("values/a.json", "../data/x.txt");
    let _barred = Barred::new(tree.root.join("data"));
    if !stating_is_denied(&tree.root.join("values/a.json")) {
        return;
    }
    each_command_fails_once(
        &tree,
        &format!(
            "error: values/a.json: could not read value JSON source: permission denied; {REMEDY}"
        ),
    );
}
