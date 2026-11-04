//! Issue #1180's contract: fetching output a home already holds changes nothing, whatever the
//! output's lead, final line ending or the home's shape, and a real replacement still replaces
//! (§FS-fetch.6.1). Each shape fetches the same output three times and compares the home's bytes
//! after every fetch with the bytes the first one left. `.gitattributes` holds the e2e corpus to
//! LF, so the CR shapes live here as table rows; the fetcher is a POSIX script.

#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

#[path = "fetch_fixed_point_ownership.rs"]
mod ownership;

const MID: &str = "AR-002-mid";
const AFTER: &str = "AR-003-after";

/// One throwaway project whose fetcher prints `scripts/payload-<ID>`, removed however it ends.
struct Project(PathBuf);

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[derive(Clone, Copy)]
enum Home {
    /// `docs/arch.md` holding these bytes, or absent.
    File(Option<&'static str>),
    /// `docs/ar/`, holding an index and, if given, `<ID>.md` with these bytes for `MID`.
    Folder(Option<&'static str>),
}

impl Project {
    fn new(home: Home) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "grund-1180-fixed-point-{}-{serial}",
            std::process::id()
        ));
        let project = Self(root);
        let _ = fs::remove_dir_all(&project.0);
        fs::create_dir_all(project.0.join("docs")).expect("create docs");
        fs::create_dir_all(project.0.join("scripts")).expect("create scripts");
        let home_line = match home {
            Home::File(_) => "file = \"docs/arch.md\"",
            Home::Folder(_) => "folder = \"docs/ar\"",
        };
        let config = format!(
            "grund_config_version = 1\n\n[id]\nnamed_sections = true\n\n[[kinds]]\n\
             kind = \"AR\"\n{home_line}\nfetch = \"scripts/fetch\"\n\n\
             [scan]\ninclude = [\"docs\"]\nrespect_gitignore = false\n"
        );
        fs::write(project.0.join("grund.toml"), config).expect("write config");
        match home {
            Home::File(Some(bytes)) => {
                fs::write(project.0.join("docs/arch.md"), bytes).expect("write home");
            }
            Home::File(None) => {}
            Home::Folder(file) => {
                let folder = project.0.join("docs/ar");
                fs::create_dir_all(&folder).expect("create folder home");
                let index = format!("# Index\n\n- [§{MID}]({MID}.md#ar-002-mid-mid)\n");
                fs::write(folder.join("README.md"), index).expect("write index");
                if let Some(bytes) = file {
                    fs::write(folder.join(format!("{MID}.md")), bytes).expect("write file");
                }
            }
        }
        let fetcher = project.0.join("scripts/fetch");
        fs::write(&fetcher, "#!/bin/sh\nexec cat \"scripts/payload-$1\"\n").expect("fetcher");
        fs::set_permissions(&fetcher, fs::Permissions::from_mode(0o755)).expect("chmod");
        project
    }

    fn print(&self, id: &str, payload: &str) {
        fs::write(self.0.join(format!("scripts/payload-{id}")), payload).expect("write payload");
    }

    /// Every file under `docs/`, path and bytes, so a file set that moves shows too.
    fn docs(&self) -> Vec<(PathBuf, Vec<u8>)> {
        fn walk(dir: &Path, root: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) {
            let mut entries = fs::read_dir(dir)
                .expect("read docs")
                .map(|entry| entry.expect("entry").path())
                .collect::<Vec<_>>();
            entries.sort();
            for path in entries {
                if path.is_dir() {
                    walk(&path, root, out);
                } else {
                    let bytes = fs::read(&path).expect("read file");
                    out.push((path.strip_prefix(root).expect("under root").into(), bytes));
                }
            }
        }
        let mut out = Vec::new();
        walk(&self.0.join("docs"), &self.0, &mut out);
        out
    }

    fn arch(&self) -> String {
        fs::read_to_string(self.0.join("docs/arch.md")).expect("read home")
    }

    /// `grund fetch <id>`: its exit, stdout and stderr.
    fn fetch(&self, id: &str) -> (Option<i32>, String) {
        let out = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(["fetch", id])
            .current_dir(&self.0)
            .output()
            .expect("spawn grund fetch");
        let said = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        (out.status.code(), said)
    }

    /// Fetch, holding the run to §FS-fetch.7's silent exit 0.
    fn fetch_ok(&self, id: &str) -> Result<(), String> {
        match self.fetch(id) {
            (Some(0), said) if said.is_empty() => Ok(()),
            (code, said) => Err(format!("fetch {id} exited {code:?}, said {said:?}")),
        }
    }

    fn check_green(&self) -> Result<(), String> {
        let out = Command::new(env!("CARGO_BIN_EXE_grund"))
            .arg("check")
            .current_dir(&self.0)
            .output()
            .expect("spawn grund check");
        if out.status.code() == Some(0) {
            Ok(())
        } else {
            Err(format!(
                "grund check exited {:?}: {}",
                out.status.code(),
                String::from_utf8_lossy(&out.stdout)
            ))
        }
    }
}

/// The home shapes a file-home row starts from, each also with CRLF line endings.
const FILE_HOMES: [(&str, Option<&str>); 7] = [
    ("absent", None),
    ("preamble", Some("# Architecture\n\n")),
    ("preamble-no-final-newline", Some("# Architecture")),
    ("preamble-two-blank-lines", Some("# Architecture\n\n\n")),
    (
        "sibling-after",
        Some("# Architecture\n\n## AR-003-after: After\nafter\n"),
    ),
    (
        "sibling-before",
        Some("# Architecture\n\n## AR-001-before: Before\nbefore\n"),
    ),
    (
        "siblings-around",
        Some(
            "# Architecture\n\n## AR-001-before: Before\nbefore\n\n\
             ## AR-003-after: After\nafter\n",
        ),
    ),
];

/// The outputs a row fetches for `MID`; `{h}` is the heading, `## ` in a file home, `# ` in a
/// folder home.
const OUTPUTS: [(&str, &str); 12] = [
    ("plain", "{h}AR-002-mid: Mid\nkept\n"),
    ("no-final-newline", "{h}AR-002-mid: Mid\nkept"),
    ("leading-blank-line", "\n{h}AR-002-mid: Mid\nkept\n"),
    ("two-leading-blank-lines", "\n\n{h}AR-002-mid: Mid\nkept\n"),
    ("leading-crlf-blank-line", "\r\n{h}AR-002-mid: Mid\nkept\n"),
    ("leading-space-only-line", " \n{h}AR-002-mid: Mid\nkept\n"),
    ("final-cr", "{h}AR-002-mid: Mid\ntail\r"),
    ("trailing-blank-line", "{h}AR-002-mid: Mid\nkept\n\n"),
    ("two-trailing-blank-lines", "{h}AR-002-mid: Mid\nkept\n\n\n"),
    ("crlf", "{h}AR-002-mid: Mid\r\nkept\r\n"),
    ("crlf-then-final-cr", "{h}AR-002-mid: Mid\r\nkept\r\n\r"),
    (
        "leading-nbsp-only-line",
        "\u{a0}\n{h}AR-002-mid: Mid\nkept\n",
    ),
];

/// Folder-home outputs beyond `OUTPUTS`: the declaration line indented by U+00A0.
const FOLDER_ONLY: [(&str, &str); 1] = [("nbsp-indent", "\u{a0}# AR-002-mid: Mid\nkept\n")];

struct Row {
    name: String,
    home: Home,
    output: String,
}

fn rows() -> Vec<Row> {
    let mut rows = Vec::new();
    for (home_name, home) in FILE_HOMES {
        for crlf in [false, true] {
            let home = home.map(|bytes| {
                let bytes = if crlf {
                    bytes.replace('\n', "\r\n")
                } else {
                    bytes.to_owned()
                };
                &*Box::leak(bytes.into_boxed_str())
            });
            if crlf && home.is_none_or(|bytes| !bytes.contains('\n')) {
                continue;
            }
            let ending = if crlf { "crlf-" } else { "" };
            for (output_name, output) in OUTPUTS {
                rows.push(Row {
                    name: format!("file/{ending}{home_name}/{output_name}"),
                    home: Home::File(home),
                    output: output.replace("{h}", "## "),
                });
            }
        }
    }
    for (output_name, output) in OUTPUTS {
        rows.push(Row {
            name: format!("folder/{output_name}"),
            home: Home::Folder(None),
            output: output.replace("{h}", "# "),
        });
    }
    for (output_name, output) in FOLDER_ONLY {
        rows.push(Row {
            name: format!("folder/{output_name}"),
            home: Home::Folder(None),
            output: output.to_owned(),
        });
    }
    rows
}

/// Fetch `row.output` three times: every fetch silent and 0, the output verbatim in the home
/// after the first, the second and third leaving the first's file set and bytes.
fn row_failure(row: &Row) -> Option<String> {
    let project = Project::new(row.home);
    project.print(MID, &row.output);
    let run = || -> Result<(), String> {
        project.fetch_ok(MID)?;
        let first = project.docs();
        let holds = first.iter().any(|(_, bytes)| {
            bytes
                .windows(row.output.len())
                .any(|w| w == row.output.as_bytes())
        });
        if !holds {
            return Err(format!(
                "fetch 1 left no verbatim copy of the output: {}",
                shown(&first)
            ));
        }
        for n in 2..=3 {
            project
                .fetch_ok(MID)
                .map_err(|err| format!("fetch {n}: {err}"))?;
            let now = project.docs();
            if now != first {
                return Err(format!(
                    "fetch {n} changed what fetch 1 left: {} -> {}",
                    shown(&first),
                    shown(&now)
                ));
            }
        }
        project.check_green()
    };
    run().err().map(|err| format!("{}: {err}", row.name))
}

/// A sequence of fetches over one file home, and what the home must be at the end.
struct Sequence {
    name: &'static str,
    run: fn(&Project) -> Result<(), String>,
}

/// A file set as text, every byte visible.
fn shown(docs: &[(PathBuf, Vec<u8>)]) -> String {
    docs.iter()
        .map(|(path, bytes)| format!("{}: {:?}", path.display(), String::from_utf8_lossy(bytes)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn same(label: &str, was: &str, now: &str) -> Result<(), String> {
    if was == now {
        Ok(())
    } else {
        Err(format!("{label}: {was:?} -> {now:?}"))
    }
}

const SEQUENCES: [Sequence; 3] = [
    // Two IDs fetched in turn, the later one printing a lead: neither takes the other's bytes.
    Sequence {
        name: "sequence/alternate-ids-later-one-leading-blank-line",
        run: |p| {
            p.print(MID, "## AR-002-mid: Mid\nkept\n");
            p.print(AFTER, "\n## AR-003-after: After\nafter\n");
            p.fetch_ok(MID)?;
            p.fetch_ok(AFTER)?;
            let settled = p.arch();
            p.fetch_ok(MID)?;
            same("fetch mid again", &settled, &p.arch())?;
            p.fetch_ok(AFTER)?;
            same("fetch after again", &settled, &p.arch())
        },
    },
    // A fetcher that keeps its lead and changes its text replaces the text and nothing more.
    Sequence {
        name: "sequence/leading-blank-line-text-changes",
        run: |p| {
            p.print(MID, "\n## AR-002-mid: Mid\nold\n");
            p.fetch_ok(MID)?;
            let expected = p.arch().replace("\nold\n", "\nnew\n");
            p.print(MID, "\n## AR-002-mid: Mid\nnew\n");
            p.fetch_ok(MID)?;
            same("fetch with new text", &expected, &p.arch())?;
            p.fetch_ok(MID)?;
            same("fetch new text again", &expected, &p.arch())
        },
    },
    // A final CR last in the home, then a later ID inserted after it and both fetched again.
    Sequence {
        name: "sequence/final-cr-then-insert-after",
        run: |p| {
            p.print(MID, "## AR-002-mid: Mid\ntail\r");
            p.print(AFTER, "## AR-003-after: After\nafter\n");
            p.fetch_ok(MID)?;
            p.fetch_ok(AFTER)?;
            let settled = p.arch();
            p.fetch_ok(AFTER)?;
            same("fetch after again", &settled, &p.arch())?;
            p.fetch_ok(MID)?;
            same("fetch mid again", &settled, &p.arch())
        },
    },
];

fn sequence_failure(sequence: &Sequence) -> Option<String> {
    let project = Project::new(Home::File(Some("# Architecture\n\n")));
    let result = (sequence.run)(&project).and_then(|()| project.check_green());
    result.err().map(|err| format!("{}: {err}", sequence.name))
}

/// Every row and every sequence reaches its fixed point on the first fetch.
#[test]
fn fetch_fixed_point_holds() {
    let mut failures = rows().iter().filter_map(row_failure).collect::<Vec<_>>();
    failures.extend(SEQUENCES.iter().filter_map(sequence_failure));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// What a fetch must leave, byte for byte, where §FS-fetch.4, .5 and .6.1 fix it: the controls
/// that keep the unchanged-output path from swallowing a real replacement.
struct Control {
    name: &'static str,
    home: Home,
    output: &'static str,
    /// The home after one fetch and after a second, or `None` for a refusal that leaves it.
    expected: Option<&'static str>,
}

const CONTROLS: [Control; 6] = [
    Control {
        name: "home-declaration-holds-more-before-a-sibling",
        home: Home::File(Some(
            "# Architecture\n\n## AR-002-mid: Mid\nkept\n\nextra\n\n## AR-003-after: After\nafter\n",
        )),
        output: "## AR-002-mid: Mid\nkept\n",
        expected: Some(
            "# Architecture\n\n## AR-002-mid: Mid\nkept\n\n## AR-003-after: After\nafter\n",
        ),
    },
    Control {
        name: "home-declaration-holds-more-at-the-end",
        home: Home::File(Some("# Architecture\n\n## AR-002-mid: Mid\nkept\nmore\n")),
        output: "## AR-002-mid: Mid\nkept\n",
        expected: Some("# Architecture\n\n## AR-002-mid: Mid\nkept\n"),
    },
    Control {
        name: "preamble-blank-lines-stay-without-a-lead",
        home: Home::File(Some("# Architecture\n\n\n## AR-002-mid: Mid\nold\n")),
        output: "## AR-002-mid: Mid\nkept\n",
        expected: Some("# Architecture\n\n\n## AR-002-mid: Mid\nkept\n"),
    },
    Control {
        name: "final-cr-last-in-the-home",
        home: Home::File(Some("# Architecture\n\n")),
        output: "## AR-002-mid: Mid\ntail\r",
        expected: Some("# Architecture\n\n## AR-002-mid: Mid\ntail\r"),
    },
    Control {
        name: "folder-file-content-before-the-heading",
        home: Home::Folder(Some("intro\n\n# AR-002-mid: Mid\nold\n")),
        output: "# AR-002-mid: Mid\nkept\n",
        expected: None,
    },
    Control {
        name: "folder-file-nbsp-then-content-before-the-heading",
        home: Home::Folder(Some("\u{a0}intro\n# AR-002-mid: Mid\nold\n")),
        output: "# AR-002-mid: Mid\nkept\n",
        expected: None,
    },
];

fn control_failure(control: &Control) -> Option<String> {
    let project = Project::new(control.home);
    project.print(MID, control.output);
    let run = || -> Result<(), String> {
        let Some(expected) = control.expected else {
            let before = project.docs();
            match project.fetch(MID) {
                (Some(2), _) if project.docs() == before => return Ok(()),
                (code, said) => return Err(format!("expected a refusal, got {code:?} {said:?}")),
            }
        };
        for n in 1..=2 {
            project
                .fetch_ok(MID)
                .map_err(|err| format!("fetch {n}: {err}"))?;
            same(&format!("after fetch {n}"), expected, &project.arch())?;
        }
        project.check_green()
    };
    run().err().map(|err| format!("{}: {err}", control.name))
}

/// The controls hold today and must still hold after the fix.
#[test]
fn fetch_fixed_point_controls() {
    let failures = CONTROLS
        .iter()
        .filter_map(control_failure)
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
