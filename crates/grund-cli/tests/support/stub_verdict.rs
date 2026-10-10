//! The columns of the stub-verdict agreement table
//! (§FS-declarations.checks.broken-stub.4): the fixture tree each row builds,
//! and how each command's answer is read off it and compared with the row's
//! verdict (§FS-declarations.stubs.verdict).

use grund_core::{LspSnapshotOpts, ShowOpts, lsp_snapshot, show_with_overlays};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

/// The citation every row carries, before `fmt` has touched it.
const USES: &str = "Uses \u{a7}FS-a.\n";

/// A `path:line`, the path relative to the fixture root and `/`-separated.
type Site = (String, usize);

/// One tree, and the verdict every command must read off it.
pub struct Row {
    pub name: &'static str,
    /// `[scan] include`.
    pub include: &'static [&'static str],
    /// The file name under `notes/` the stub at `docs/a.md:1` links to.
    pub target: &'static str,
    /// Files on disk beside the stub and the citation.
    pub files: &'static [(&'static str, &'static str)],
    /// `link -> target` symlinks, the target relative to the link's directory.
    pub links: &'static [(&'static str, &'static str)],
    /// Unsaved editor text over a file on disk.
    pub editor: &'static [(&'static str, &'static str)],
    /// `check` reports the stub broken.
    pub broken: bool,
    /// The declarations of `FS-a`, its stubs paired, in `path:line` order.
    pub declarations: &'static [(&'static str, usize)],
}

impl Row {
    fn declarations(&self) -> Vec<Site> {
        self.declarations
            .iter()
            .map(|(path, line)| (path.to_string(), *line))
            .collect()
    }

    /// The duplicate findings `check` must report: one, at every declaration, or none.
    fn duplicates(&self) -> Vec<Vec<Site>> {
        if self.declarations.len() > 1 {
            vec![self.declarations()]
        } else {
            Vec::new()
        }
    }

    /// The declaration `show` answers, the refusal it gives, or the link `fmt` writes.
    fn shown(&self) -> Shown {
        match (self.declarations(), self.broken) {
            (declarations, _) if declarations.len() > 1 => Shown::Ambiguous(declarations),
            (_, true) => Shown::Broken,
            (declarations, false) => Shown::Body(declarations[0].clone()),
        }
    }

    fn is_broken_stub(&self, site: &Site) -> bool {
        self.broken && site.0 == "docs/a.md" && site.1 == 1
    }
}

#[derive(Debug, PartialEq)]
enum Shown {
    Body(Site),
    Ambiguous(Vec<Site>),
    Broken,
    Other(String),
}

/// A fixture tree under the worktree's own `target/`.
struct Repo(PathBuf);

impl Repo {
    /// The row's tree, with the editor's text saved over the disk's when `saved`.
    fn new(row: &Row, saved: bool) -> Self {
        let serial = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/stub-resolution-agreement")
            .join(format!("{}-{}-{serial}", row.name, std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create agreement fixture");
        let repo = Self(root.canonicalize().expect("canonical fixture root"));
        let include: Vec<String> = row
            .include
            .iter()
            .map(|path| format!("\"{path}\""))
            .collect();
        repo.write(
            "grund.toml",
            &format!(
                "grund_config_version = 1\n[reference]\nstrict = true\nrequire_grounding = false\n\
                 [id]\nformat = \"{{kind}}-{{slug}}\"\n\
                 [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n\
                 [scan]\ninclude = [{}]\n",
                include.join(", ")
            ),
        );
        let target = row.target;
        repo.write(
            "docs/a.md",
            &format!("# FS-a: [../notes/{target}](../notes/{target})\n"),
        );
        repo.write("docs/uses.md", USES);
        for (path, body) in row.files {
            repo.write(path, body);
        }
        if saved {
            for (path, body) in row.editor {
                repo.write(path, body);
            }
        }
        for (link, target) in row.links {
            symlink(target, &repo.0.join(link));
        }
        repo
    }

    fn write(&self, relative: &str, body: &str) {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().expect("fixture parent")).expect("create parent");
        fs::write(path, body).expect("write fixture");
    }

    /// The editor's open documents, keyed by absolute path.
    fn editor(&self, row: &Row) -> BTreeMap<PathBuf, String> {
        row.editor
            .iter()
            .map(|(path, text)| (self.0.join(path), text.to_string()))
            .collect()
    }

    /// `path` relative to the root and `/`-separated, as reports print it.
    fn relative(&self, path: &Path) -> String {
        let path = path.strip_prefix(&self.0).unwrap_or(path);
        path.to_string_lossy().replace('\\', "/")
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(unix)]
fn symlink(target: &str, link: &Path) {
    std::os::unix::fs::symlink(target, link).expect("create fixture symlink");
}

#[cfg(not(unix))]
fn symlink(_: &str, link: &Path) {
    panic!("symlink rows run on unix only: {}", link.display());
}

/// Every column that disagrees with its row, collected so one run names them all.
#[derive(Default)]
struct Disagreements(Vec<String>);

impl Disagreements {
    fn expect<T: Debug + PartialEq>(&mut self, row: &Row, column: &str, got: T, want: T) {
        if got != want {
            let name = row.name;
            self.0.push(format!(
                "{name} / {column}:\n    got  {got:?}\n    want {want:?}"
            ));
        }
    }

    fn assert_none(self) {
        assert!(
            self.0.is_empty(),
            "{} column(s) disagree with the stub's verdict (§FS-declarations.checks.broken-stub.4):\n{}",
            self.0.len(),
            self.0.join("\n")
        );
    }
}

fn grund(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(root)
        .output()
        .unwrap_or_else(|error| panic!("run grund {args:?}: {error}"))
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn json_lines(text: &str) -> Vec<Value> {
    text.lines()
        .filter(|line| line.starts_with('{'))
        .map(|line| serde_json::from_str(line).unwrap_or_else(|error| panic!("{error}: {line}")))
        .collect()
}

fn site(value: &Value) -> Site {
    let path = value["path"].as_str().unwrap_or("?").to_string();
    (path, value["line"].as_u64().unwrap_or(0) as usize)
}

fn sorted_sites(value: &Value) -> Vec<Site> {
    let mut sites: Vec<Site> = value["sites"]
        .as_array()
        .into_iter()
        .flatten()
        .map(site)
        .collect();
    sites.sort();
    sites
}

fn ambiguity(declarations: &[Site]) -> String {
    let sites: Vec<String> = declarations
        .iter()
        .map(|(path, line)| format!("{path}:{line}"))
        .collect();
    format!("ambiguous ID: FS-a (declared at {})", sites.join(", "))
}

fn check_column(repo: &Repo, row: &Row, out: &mut Disagreements) {
    let findings = json_lines(&text(
        &grund(&repo.0, &["check", "--format", "json"]).stdout,
    ));
    let stub = ("docs/a.md".to_string(), 1);
    let broken = findings
        .iter()
        .any(|f| f["code"] == "broken-stub" && site(f) == stub);
    out.expect(row, "check broken-stub", broken, row.broken);
    let duplicates: Vec<Vec<Site>> = findings
        .iter()
        .filter(|f| f["code"] == "duplicate")
        .map(sorted_sites)
        .collect();
    out.expect(row, "check duplicate sites", duplicates, row.duplicates());
}

fn show_column(repo: &Repo, row: &Row, out: &mut Disagreements) {
    let shown = grund(&repo.0, &["FS-a", "--format", "json"]);
    let got = if shown.status.success() {
        let stdout = text(&shown.stdout);
        json_lines(&stdout)
            .first()
            .map(site)
            .map_or(Shown::Other(stdout), Shown::Body)
    } else {
        let stderr = text(&shown.stderr);
        match json_lines(&stderr).first() {
            Some(refusal) if refusal["code"] == "ambiguous" => {
                Shown::Ambiguous(sorted_sites(refusal))
            }
            Some(refusal) if refusal["code"] == "broken-stub" => Shown::Broken,
            _ => Shown::Other(stderr),
        }
    };
    out.expect(row, "show", got, row.shown());
}

fn refs_column(repo: &Repo, row: &Row, out: &mut Disagreements) {
    let refs = grund(&repo.0, &["refs", "FS-a"]);
    let got = if refs.status.success() {
        Ok(text(&refs.stdout)
            .lines()
            .map(str::to_string)
            .collect::<Vec<_>>())
    } else {
        Err(text(&refs.stderr)
            .lines()
            .next()
            .unwrap_or_default()
            .to_string())
    };
    let want = match row.declarations() {
        declarations if declarations.len() > 1 => Err(ambiguity(&declarations)),
        _ => Ok(vec!["docs/uses.md:1: \u{a7}FS-a".to_string()]),
    };
    out.expect(row, "refs", got, want);
}

fn list_column(repo: &Repo, row: &Row, out: &mut Disagreements) {
    let listed = json_lines(&text(&grund(&repo.0, &["list", "--format", "json"]).stdout));
    let rows: Vec<&Value> = listed
        .iter()
        .filter(|entry| entry["id"] == "FS-a")
        .collect();
    let duplicate = row.declarations.len() > 1;
    let flags: Vec<bool> = rows
        .iter()
        .map(|entry| entry["duplicate"] == true)
        .collect();
    out.expect(
        row,
        "list duplicate",
        flags,
        vec![duplicate; rows.len().max(1)],
    );
    if row.broken {
        let stub_row = rows
            .iter()
            .any(|entry| row.is_broken_stub(&site(entry)) && entry["stub"] == true);
        out.expect(row, "list broken stub row", stub_row, true);
    }
}

/// `list --size`: one row per declaration, the broken stub's unmeasured and labelled.
fn sizes_column(repo: &Repo, row: &Row, out: &mut Disagreements) {
    let listed = json_lines(&text(
        &grund(&repo.0, &["list", "--size", "--format", "json"]).stdout,
    ));
    let mut got: Vec<(Site, bool, bool, bool)> = listed
        .iter()
        .filter(|entry| entry["id"] == "FS-a" && entry["section"].is_null())
        .map(|entry| {
            let measured = !entry["lead_lines"].is_null();
            (
                site(entry),
                entry["stub"] == true,
                measured,
                entry["duplicate"] == true,
            )
        })
        .collect();
    got.sort();
    let duplicate = row.declarations.len() > 1;
    let want: Vec<(Site, bool, bool, bool)> = row
        .declarations()
        .into_iter()
        .map(|declaration| {
            let broken = row.is_broken_stub(&declaration);
            (declaration, broken, !broken, duplicate)
        })
        .collect();
    out.expect(
        row,
        "list --size (site, stub, measured, duplicate)",
        got,
        want,
    );
}

/// `fmt --write` links the citation only to an ID's one declaration, not a broken stub.
fn fmt_column(repo: &Repo, row: &Row, out: &mut Disagreements) {
    grund(&repo.0, &["fmt", "--write"]);
    let written = fs::read_to_string(repo.0.join("docs/uses.md")).expect("read docs/uses.md");
    let got = (written != USES).then(|| match written.split_once("](") {
        Some((_, link)) => link
            .split(['#', ')'])
            .next()
            .unwrap_or_default()
            .to_string(),
        None => written.clone(),
    });
    let want = match row.shown() {
        Shown::Body((path, _)) => Some(format!("../{path}")),
        _ => None,
    };
    out.expect(row, "fmt link", got, want);
}

/// The editor snapshot: its diagnostics are `check`'s, and a citation of an ID
/// with one declaration navigates to that declaration's record.
fn lsp_column(repo: &Repo, row: &Row, column: &str, out: &mut Disagreements) {
    let snapshot = lsp_snapshot(LspSnapshotOpts {
        path: repo.0.clone(),
        path_provided: true,
        open_documents: repo.editor(row),
    })
    .expect("lsp snapshot");
    let errors = &snapshot.report.errors;
    let located = |path: &Option<String>, line: Option<usize>| {
        (
            repo.relative(Path::new(path.as_deref().unwrap_or("?"))),
            line.unwrap_or(0),
        )
    };
    let broken = errors.iter().any(|finding| {
        finding.code == "broken-stub" && row.is_broken_stub(&located(&finding.path, finding.line))
    });
    out.expect(row, &format!("{column} broken-stub"), broken, row.broken);
    let duplicates: Vec<Vec<Site>> = errors
        .iter()
        .filter(|finding| finding.code == "duplicate")
        .map(|finding| {
            let mut sites: Vec<Site> = finding
                .sites
                .iter()
                .map(|s| (repo.relative(Path::new(&s.path)), s.line))
                .collect();
            sites.sort();
            sites
        })
        .collect();
    out.expect(
        row,
        &format!("{column} duplicate sites"),
        duplicates,
        row.duplicates(),
    );
    let target = snapshot
        .citations
        .iter()
        .find(|citation| {
            citation.query_id == "FS-a" && repo.relative(&citation.path) == "docs/uses.md"
        })
        .and_then(|citation| {
            Some((
                repo.relative(citation.target_path.as_ref()?),
                citation.target_line?,
            ))
        });
    let declarations = row.declarations();
    let lands = target
        .as_ref()
        .is_some_and(|site| match declarations.len() {
            1 => *site == declarations[0],
            _ => declarations.contains(site),
        });
    out.expect(
        row,
        &format!("{column} citation target {target:?} is one of the ID's declarations"),
        lands,
        true,
    );
}

/// `show` through the API, over the editor's text.
fn show_api_column(repo: &Repo, row: &Row, out: &mut Disagreements) {
    let opts = ShowOpts {
        path: repo.0.clone(),
        ..ShowOpts::default()
    };
    let got = match show_with_overlays("FS-a", opts, repo.editor(row)) {
        Ok(shown) => Shown::Body((repo.relative(&shown.path), shown.line)),
        Err(error) => {
            let message = error.to_string();
            match row.declarations() {
                declarations if message == ambiguity(&declarations) => {
                    Shown::Ambiguous(declarations)
                }
                _ if message.starts_with("broken stub: FS-a ") => Shown::Broken,
                _ => Shown::Other(message),
            }
        }
    };
    out.expect(row, "show (editor)", got, row.shown());
}

/// Every column on the saved tree, then the editor's columns over the unsaved one.
pub fn assert_rows_agree(rows: &[Row]) {
    let mut out = Disagreements::default();
    for row in rows {
        let saved = Repo::new(row, true);
        check_column(&saved, row, &mut out);
        show_column(&saved, row, &mut out);
        refs_column(&saved, row, &mut out);
        list_column(&saved, row, &mut out);
        sizes_column(&saved, row, &mut out);
        lsp_column(&saved, row, "lsp", &mut out);
        fmt_column(&saved, row, &mut out);
        if !row.editor.is_empty() {
            let edited = Repo::new(row, false);
            show_api_column(&edited, row, &mut out);
            lsp_column(&edited, row, "lsp (editor)", &mut out);
        }
    }
    out.assert_none();
}
