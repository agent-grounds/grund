//! Issue #1284: refusal details use lexical config-relative paths, independently
//! of an include or home's spelling (§FS-config-migrate.3.1.4.1,
//! §FS-config-migrate.3.1.5.1). Ports tool report grund.267's run.sh to the CLI
//! harness; the scanner remains the oracle for what version 1 reads.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Fixture {
    root: PathBuf,
    config: String,
    nested: bool,
}

impl Fixture {
    fn new(include: &str, home: &str, nested: bool) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "grund-1284-parent-paths-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("docs/x")).unwrap();
        fs::create_dir_all(root.join("other")).unwrap();
        let git = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(git.status.success(), "{git:?}");
        for (path, id) in [
            ("docs/b.md", "FS-b"),
            ("docs/x/a.md", "FS-a"),
            ("other/c.md", "FS-c"),
        ] {
            fs::write(root.join(path), format!("# {id}: {}\n", &id[3..])).unwrap();
        }
        let mut config = format!(
            "grund_config_version = 1\nproject_name = \"demo\"\n\n\
             [id]\nformat = \"{{kind}}-{{slug}}\"\nnamed_sections = true\n\
             slug_pattern = \"[a-z][a-z0-9-]*\"\n\n\
             [[kinds]]\nkind = \"FS\"\nfolder = \"{home}\"\nindex = false\n\n"
        );
        if nested {
            config.push_str("[[kinds]]\nkind = \"REQ\"\nfolder = \"docs/x\"\nindex = false\n\n");
        }
        config.push_str(&format!("[scan]\ninclude = [\"{include}\"]\n"));
        if !nested {
            config.push_str("exclude = [\"docs/x\"]\n");
        }
        config.push_str("extensions = [\"md\"]\n");
        fs::write(root.join("grund.toml"), &config).unwrap();
        Self {
            root,
            config,
            nested,
        }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.root)
            .env("NO_COLOR", "1")
            .env("XDG_CONFIG_HOME", self.root.join(".config"))
            .env("GIT_CONFIG_GLOBAL", self.root.join(".git/no-global-config"))
            .env("GIT_CONFIG_SYSTEM", self.root.join(".git/no-system-config"))
            .output()
            .unwrap()
    }

    fn refusal(&self, write: bool) -> String {
        let before = snapshot(&self.root);
        let mut args = vec!["config", "migrate", "--to", "2"];
        if write {
            args.push("--write");
        }
        let out = self.run(&args);
        let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
        assert_eq!(out.status.code(), Some(1), "{stderr}");
        assert!(out.stdout.is_empty(), "refusal has no candidate: {out:?}");
        assert_eq!(snapshot(&self.root), before, "refusal writes nothing");
        assert_eq!(
            stderr.lines().filter(|l| l.starts_with("refused:")).count(),
            1,
            "{stderr}"
        );
        let repair = if self.nested {
            "  repair: un-nest the two homes under version 1"
        } else {
            "  repair: remove `docs/x` from exclude under version 1, which keeps what is scanned exactly as it is"
        };
        assert!(stderr.lines().any(|l| l == repair), "{stderr}");
        let reason = if self.nested {
            "[[kinds]] `REQ` home `docs/x` is inside `FS` home `docs`; version 1 gives the files in both to the complement, version 2 to the deepest place"
        } else {
            "[scan] exclude entry `docs/x` contains `/`, so version 1 never matched it; as a version 2 glob it would stop reading "
        };
        assert!(stderr.contains(reason), "{stderr}");
        let header = if write {
            "grund.toml: version 1 → 2, --write"
        } else {
            "grund.toml: version 1 → 2, preview, nothing written"
        };
        let closing = if write {
            "not written: 1 refusal (follow the repairs above); grund.toml is unchanged"
        } else {
            "would not write: 1 refusal (follow the repairs above)"
        };
        assert_eq!(stderr.lines().next(), Some(header), "{stderr}");
        assert_eq!(stderr.lines().last(), Some(closing), "{stderr}");
        stderr
    }

    fn assert_detail(&self, count: usize) {
        // Check both modes before failing, so the evidence covers preview and write.
        let mut failures = Vec::new();
        for write in [false, true] {
            let stderr = self.refusal(write);
            let expected = if self.nested {
                vec!["  would change owner: docs/x/a.md"]
            } else if count == 0 {
                vec![]
            } else {
                vec!["  would stop reading: docs/x/a.md"]
            };
            let actual: Vec<_> = stderr
                .lines()
                .filter(|l| l.starts_with("  would "))
                .collect();
            let correct_count = self.nested
                || stderr.lines().any(|l| {
                    l.starts_with("refused:")
                        && l.ends_with(&format!(
                            "would stop reading {count} {}",
                            if count == 1 { "file" } else { "files" }
                        ))
                });
            if actual != expected || !correct_count {
                failures.push(format!(
                    "write={write}: expected count={count}, details={expected:?}\n{stderr}"
                ));
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    fn assert_v1_reads_after_repair(&self, includes_other: bool) {
        let repaired = self.config.replace("exclude = [\"docs/x\"]\n", "");
        fs::write(self.root.join("grund.toml"), repaired).unwrap();
        let out = self.run(&["list"]);
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        assert!(out.status.success(), "{out:?}");
        let ids: Vec<_> = stdout
            .lines()
            .map(|line| line.split_whitespace().next().unwrap())
            .collect();
        let expected = if includes_other {
            vec!["FS-a", "FS-b", "FS-c"]
        } else {
            vec!["FS-a", "FS-b"]
        };
        assert_eq!(ids, expected, "{stdout}");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, at: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(at).unwrap() {
            let path = entry.unwrap().path();
            if path.file_name().unwrap() == ".git" {
                continue;
            }
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(root).unwrap().into(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

#[test]
fn exclude_include_parent_after_child() {
    Fixture::new("docs/x/..", "docs", false).assert_detail(1);
}

#[test]
fn exclude_include_parent_before_home() {
    Fixture::new("docs/../docs", "docs", false).assert_detail(1);
}

#[test]
fn exclude_home_parent_after_child() {
    Fixture::new("other", "docs/x/..", false).assert_detail(1);
}

#[test]
fn exclude_home_parent_before_home() {
    Fixture::new("other", "docs/../docs", false).assert_detail(1);
}

#[test]
fn nested_include_parent_after_child() {
    Fixture::new("docs/x/..", "docs", true).assert_detail(1);
}

#[test]
fn nested_include_parent_before_home() {
    Fixture::new("docs/../docs", "docs", true).assert_detail(1);
}

/// Ordinary spellings from grund.267 remain controls for both refusal details.
#[test]
fn ordinary_include_and_home_spellings_are_controls() {
    for spelling in ["docs", "./docs", "docs/"] {
        Fixture::new(spelling, "docs", false).assert_detail(1);
        Fixture::new("other", spelling, false).assert_detail(1);
        Fixture::new(spelling, "docs", true).assert_detail(1);
    }
}

/// The documented repair leaves the affected files in the version 1 read set.
#[test]
fn version_one_reads_both_files_through_parent_spellings_after_repair() {
    for spelling in ["docs/x/..", "docs/../docs"] {
        Fixture::new(spelling, "docs", false).assert_v1_reads_after_repair(false);
        Fixture::new("other", spelling, false).assert_v1_reads_after_repair(true);
    }
}

/// Normalizing only file names must not lose scan-root protection.
#[test]
fn equivalent_parent_home_spellings_protect_the_scan_root() {
    for home in ["docs/x", "docs/x/../x", "docs/../docs/x"] {
        Fixture::new("other", home, false).assert_detail(0);
    }
}

/// An include inside a home becomes no protected place (§FS-config-migrate.3.1.4.1).
#[test]
fn exclude_include_inside_home() {
    let fixture = Fixture::new("docs/x", "docs", false);
    fixture.assert_detail(1);
    fixture.assert_v1_reads_after_repair(false);
}

#[test]
fn exclude_include_inside_home_parent_after_child() {
    let fixture = Fixture::new("docs/x", "docs/x/..", false);
    fixture.assert_detail(1);
    fixture.assert_v1_reads_after_repair(false);
}

#[test]
fn exclude_include_inside_home_parent_before_home() {
    let fixture = Fixture::new("docs/x", "docs/../docs", false);
    fixture.assert_detail(1);
    fixture.assert_v1_reads_after_repair(false);
}
