//! §FS-scan.path.none, §FS-scan.path.list: below a workspace root and outside every
//! member, a run given no path covers the whole workspace, and only an explicit path
//! narrows what `list`, `show --batch --all|--glob`, `refs` and `complete ids` report,
//! as it narrows `check` from there (§FS-scan.projects.narrowed) and the same forms
//! inside a member. The fixture is grund#858's: a root with `include_root = false`,
//! members `api` and `web` that each declare `FS-<m>`, with the named section
//! `format.1`, and `FS-<m>-audit` citing it, and an empty `tools/sub/` no member
//! contains. Its variant nests `api` at `apps/api`, so `apps/` is a path outside every
//! member that holds one member's entries, and narrowing cannot pass by printing nothing.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const ROOT_CONFIG: &str = "grund_config_version = 1\n\n[reference]\nmarker = \"\u{a7}\"\n\
                           strict = true\n\n[id]\nformat = \"{kind}-{slug}\"\n\
                           slug_pattern = \"[a-z][a-z0-9-]*\"\n\n[workspace]\n";
const MEMBER_CONFIG: &str = "grund_config_version = 1\n\n[reference]\nmarker = \"\u{a7}\"\n\
                             strict = true\n\n[id]\nformat = \"{kind}-{slug}\"\n\
                             slug_pattern = \"[a-z][a-z0-9-]*\"\nnamed_sections = true\n\n\
                             [scan]\ninclude = [\"docs\", \"src\"]\n";

/// What every run from below grund#858's root prints when it covers the workspace.
const CATALOG: &str = "api/FS-api        api/docs/FS-api.md:1  api\n\
                       api/FS-api-audit  api/docs/FS-api-audit.md:1  Audit\n\
                       web/FS-web        web/docs/FS-web.md:1  web\n\
                       web/FS-web-audit  web/docs/FS-web-audit.md:1  Audit\n";

static NEXT: AtomicUsize = AtomicUsize::new(0);

/// One run's exit code, stdout and stderr.
type Run = (Option<i32>, String, String);

struct Workspace {
    root: PathBuf,
}

impl Workspace {
    /// grund#858's workspace with its members at `members`, each named for its last
    /// component, and an empty `tools/sub/`, once a root `fmt --write` has written the
    /// members' section locks, as the issue's reproducer does before `list`.
    fn new(what: &str, members: &[&str]) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-858-{what}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("tools/sub")).unwrap();
        let ws = Self { root };
        let quoted: Vec<String> = members.iter().map(|m| format!("\"{m}\"")).collect();
        let config = format!(
            "{ROOT_CONFIG}members = [{}]\ninclude_root = false\n",
            quoted.join(", ")
        );
        ws.write("grund.toml", &config);
        for member in members {
            let name = member.rsplit('/').next().unwrap();
            let config = format!("project_name = \"{name}\"\n{MEMBER_CONFIG}");
            ws.write(&format!("{member}/grund.toml"), &config);
            ws.write(
                &format!("{member}/docs/FS-{name}.md"),
                &format!(
                    "# FS-{name}: {name}\n\n## format: Formatting\n\n### .1: Width\n\nWrap at 80.\n"
                ),
            );
            ws.write(
                &format!("{member}/docs/FS-{name}-audit.md"),
                &format!("# FS-{name}-audit: Audit\n\nCites \u{a7}FS-{name}.format.1.\n"),
            );
            ws.write(
                &format!("{member}/src/lib.rs"),
                &format!("/// Wraps (\u{a7}FS-{name}.format.1).\npub fn wrap() {{}}\n"),
            );
        }
        let (code, _, stderr) = ws.run("", &["fmt", "--write"]);
        assert_eq!(code, Some(0), "the fixture's fmt --write failed: {stderr}");
        ws
    }

    /// The issue's own layout: members `api` and `web` directly under the root.
    fn issue(what: &str) -> Self {
        Self::new(what, &["api", "web"])
    }

    /// The variant: `api` at `apps/api`, below a directory no member is.
    fn nested(what: &str) -> Self {
        Self::new(what, &["apps/api", "web"])
    }

    fn write(&self, path: &str, body: &str) {
        let path = self.root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    /// Runs grund in `cwd`, relative to the workspace root.
    fn run(&self, cwd: &str, args: &[&str]) -> Run {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(self.root.join(cwd))
            .output()
            .expect("spawn grund");
        (
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        )
    }

    /// Records a mismatch unless the run's exit and stdout are `code` and `stdout`, and,
    /// where `stderr` is given, its stderr is too.
    fn expect(
        &self,
        mismatches: &mut Vec<String>,
        cwd: &str,
        args: &[&str],
        want: (Option<i32>, &str, Option<&str>),
    ) {
        let got = self.run(cwd, args);
        let (code, stdout, stderr) = want;
        if got.0 != code || got.1 != stdout || stderr.is_some_and(|stderr| got.2 != stderr) {
            mismatches.push(format!(
                "(cd {} && grund {}): want exit {code:?}, stdout {stdout:?}{}; got {got:?}",
                if cwd.is_empty() { "." } else { cwd },
                args.join(" "),
                stderr
                    .map(|stderr| format!(", stderr {stderr:?}"))
                    .unwrap_or_default(),
            ));
        }
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn assert_none(mismatches: &[String]) {
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A run that prints nothing, exit 0 and an empty stderr.
const NOTHING: (Option<i32>, &str, Option<&str>) = (Some(0), "", Some(""));

/// §FS-scan.path.none: from `tools/sub/`, a bare `list` covers every project the
/// workspace-root run loads, and `check .` already narrows to `tools/sub/`
/// (§FS-scan.projects.narrowed); inside a member, `.` narrows `list`, `show --batch`
/// and `complete ids` to a directory that declares nothing (§FS-scan.path.list). These
/// hold today and must keep holding once the run outside every member narrows too.
#[test]
fn outside_every_member_no_path_is_the_workspace_and_a_member_narrows_to_dot() {
    let ws = Workspace::issue("controls");
    let mut mismatches = Vec::new();
    ws.expect(
        &mut mismatches,
        "tools/sub",
        &["list"],
        (Some(0), CATALOG, Some("")),
    );
    ws.expect(&mut mismatches, "", &["list"], (Some(0), CATALOG, Some("")));
    ws.expect(
        &mut mismatches,
        "tools/sub",
        &["check", "."],
        (Some(0), "", None),
    );
    ws.expect(&mut mismatches, "api/src", &["list", "."], NOTHING);
    let all = ["show", "--batch", "--all", "--path", ".", "--format=json"];
    ws.expect(&mut mismatches, "api/src", &all, NOTHING);
    let ids = ["complete", "ids", "--path", ".", "--prefix", "FS-"];
    ws.expect(&mut mismatches, "api/src", &ids, NOTHING);
    // §FS-completions.2.1: an unqualified prefix has no current project to complete from
    // outside every member, so this holds today and guards the narrowed run.
    ws.expect(&mut mismatches, "tools/sub", &ids, NOTHING);
    assert_none(&mismatches);
}

/// §FS-scan.path.list: from `tools/sub/`, which no member contains and which declares
/// nothing, `list .` prints nothing in each of its forms, `--size`, `--unused` and
/// `--summary` (§FS-list.3.3: an empty summary prints nothing) included, and so does
/// `list tools/sub` from the root. Today each prints the whole workspace catalog.
#[test]
fn list_below_the_root_outside_every_member_prints_nothing_where_no_member_is() {
    let ws = Workspace::issue("list");
    let mut mismatches = Vec::new();
    let forms: &[&[&str]] = &[
        &["list", "."],
        &["list", ".", "--size"],
        &["list", ".", "--unused"],
        &["list", ".", "--summary"],
        &["list", ".", "--format", "json"],
        &["list", ".", "--summary", "--format", "json"],
    ];
    for args in forms {
        ws.expect(&mut mismatches, "tools/sub", args, NOTHING);
    }
    ws.expect(&mut mismatches, "", &["list", "tools/sub"], NOTHING);
    assert_none(&mismatches);
}

/// §FS-scan.path.show: `--batch --all` generates its queries from the entries
/// `list <path>` prints, and `--glob` filters that set (§FS-show.2.6.2), so from
/// `tools/sub/` both generate nothing. Today they read every body in the workspace.
#[test]
fn show_batch_below_the_root_outside_every_member_generates_nothing_where_no_member_is() {
    let ws = Workspace::issue("show");
    let mut mismatches = Vec::new();
    let all = ["show", "--batch", "--all", "--path", ".", "--format=json"];
    ws.expect(&mut mismatches, "tools/sub", &all, NOTHING);
    let glob = [
        "show",
        "--batch",
        "--glob",
        "api/FS-*",
        "--path",
        ".",
        "--format=json",
    ];
    ws.expect(&mut mismatches, "tools/sub", &glob, NOTHING);
    let from_root = [
        "show",
        "--batch",
        "--all",
        "--path",
        "tools/sub",
        "--format=json",
    ];
    ws.expect(&mut mismatches, "", &from_root, NOTHING);
    assert_none(&mismatches);
}

/// §FS-scan.path.refs: in each mode (§FS-refs.1.1) a workspace run lists only the sites
/// at or under its path, so from `tools/sub/` every mode lists none, and `--total` reads
/// `not cited` (§FS-refs.3.4). Stderr is not pinned: an ID with no site may earn
/// §FS-refs.2.1's note. `--from` pins only that its sites are bounded, since how it
/// resolves a source declared outside the path is grund#857's.
#[test]
fn refs_below_the_root_outside_every_member_lists_no_site_outside_the_path() {
    let ws = Workspace::issue("refs");
    let mut mismatches = Vec::new();
    let forms: &[&[&str]] = &[
        &["refs", "api/FS-api", "."],
        &["refs", "api/FS-api", ".", "--descendants"],
        &["refs", "api/FS-api", ".", "--summary"],
        &["refs", "api/FS-api", ".", "--format", "json"],
        &["refs", "--glob", "*/FS-*{,.**}", "."],
    ];
    for args in forms {
        ws.expect(&mut mismatches, "tools/sub", args, (Some(0), "", None));
    }
    let total = ["refs", "api/FS-api", ".", "--total"];
    ws.expect(
        &mut mismatches,
        "tools/sub",
        &total,
        (Some(0), "not cited\n", None),
    );
    let from_root = ["refs", "api/FS-api", "tools/sub"];
    ws.expect(&mut mismatches, "", &from_root, (Some(0), "", None));
    let from = ["refs", "--from", "api/FS-api-audit", "."];
    let got = ws.run("tools/sub", &from);
    if !got.1.is_empty() {
        mismatches.push(format!(
            "(cd tools/sub && grund {}) listed sites outside the path: {got:?}",
            from.join(" ")
        ));
    }
    assert_none(&mismatches);
}

/// §FS-completions.2.1: a path narrower than its config root bounds the candidates to
/// the entries `list <path>` prints, so from `tools/sub/` an alias-path prefix completes
/// no member's IDs or sections. Today it offers the member's whole catalog.
#[test]
fn complete_ids_below_the_root_outside_every_member_offers_no_member_entry() {
    let ws = Workspace::issue("complete");
    let mut mismatches = Vec::new();
    let ids = ["complete", "ids", "--path", ".", "--prefix", "api/FS-"];
    ws.expect(&mut mismatches, "tools/sub", &ids, NOTHING);
    let sections = [
        "complete",
        "ids",
        "--path",
        ".",
        "--prefix",
        "api/FS-api.",
        "--sections",
    ];
    ws.expect(&mut mismatches, "tools/sub", &sections, NOTHING);
    let from_root = [
        "complete",
        "ids",
        "--path",
        "tools/sub",
        "--prefix",
        "web/FS-",
    ];
    ws.expect(&mut mismatches, "", &from_root, NOTHING);
    assert_none(&mismatches);
}

/// §FS-scan.path.list: below the root and outside every member, `apps/` holds `api`
/// alone, so `list apps` prints `api`'s entries and no others, in the catalog's shape
/// §FS-workspace.8.3 states: qualified IDs, JSON `project` and the project's own `refs`
/// count, and the per-project `--size` and `--summary` rows. `show --batch --all`,
/// `refs --glob` and `complete ids` are bounded the same way (§FS-scan.path.show,
/// §FS-scan.path.refs, §FS-completions.2.1). Today each prints `web`'s entries too.
#[test]
fn a_path_outside_every_member_reports_the_entries_of_the_members_under_it() {
    let ws = Workspace::nested("positive");
    let mut mismatches = Vec::new();
    let rows = "api/FS-api        apps/api/docs/FS-api.md:1  api\n\
                api/FS-api-audit  apps/api/docs/FS-api-audit.md:1  Audit\n";
    ws.expect(
        &mut mismatches,
        "",
        &["list", "apps"],
        (Some(0), rows, Some("")),
    );
    ws.expect(
        &mut mismatches,
        "apps",
        &["list", "."],
        (Some(0), rows, Some("")),
    );
    let unused = "api/FS-api-audit  apps/api/docs/FS-api-audit.md:1  Audit\n";
    let unused_args = ["list", "apps", "--unused"];
    ws.expect(
        &mut mismatches,
        "",
        &unused_args,
        (Some(0), unused, Some("")),
    );
    let json = concat!(
        r#"{"project":"api","id":"api/FS-api","kind":"FS","path":"apps/api/docs/FS-api.md","#,
        r#""line":1,"title":"api","stub":false,"defines":null,"refs":2,"duplicate":false}"#,
        "\n",
        r#"{"project":"api","id":"api/FS-api-audit","kind":"FS","#,
        r#""path":"apps/api/docs/FS-api-audit.md","line":1,"title":"Audit","stub":false,"#,
        r#""defines":null,"refs":0,"duplicate":false}"#,
        "\n",
    );
    let json_args = ["list", "apps", "--format", "json"];
    ws.expect(&mut mismatches, "", &json_args, (Some(0), json, Some("")));
    let summary = "api  FS      2  docs/functional-spec\n";
    let summary_args = ["list", "apps", "--summary"];
    ws.expect(
        &mut mismatches,
        "",
        &summary_args,
        (Some(0), summary, Some("")),
    );
    // The relative `format.1` is measured at its canonical heading (§FS-list.3.4.1).
    let size = "api/FS-api           apps/api/docs/FS-api.md:1  lines=0/3 words=0/9 bytes=0/50\n\
                api/FS-api.format    apps/api/docs/FS-api.md:3  lines=1/3 words=3/9 bytes=22/50\n\
                api/FS-api.format.1  apps/api/docs/FS-api.md:5  lines=2/2 words=6/6 bytes=33/33\n\
                api/FS-api-audit     apps/api/docs/FS-api-audit.md:1  \
                lines=1/1 words=2/2 bytes=25/25\n";
    ws.expect(
        &mut mismatches,
        "",
        &["list", "apps", "--size"],
        (Some(0), size, Some("")),
    );

    let all = ws.run(
        "",
        &[
            "show",
            "--batch",
            "--all",
            "--path",
            "apps",
            "--format=json",
        ],
    );
    let generated: Vec<(String, Option<String>, String)> = all
        .1
        .lines()
        .map(|line| {
            let line: serde_json::Value = serde_json::from_str(line).expect("NDJSON line");
            (
                line["query"]["id"].as_str().unwrap_or_default().to_owned(),
                line["query"]["section"].as_str().map(str::to_owned),
                line["result"]["path"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned(),
            )
        })
        .collect();
    let query = |id: &str, section: Option<&str>, path: &str| {
        (id.to_owned(), section.map(str::to_owned), path.to_owned())
    };
    let want = vec![
        query("api/FS-api", None, "apps/api/docs/FS-api.md"),
        query("api/FS-api", Some("format"), "apps/api/docs/FS-api.md"),
        query("api/FS-api", Some("format.1"), "apps/api/docs/FS-api.md"),
        query("api/FS-api-audit", None, "apps/api/docs/FS-api-audit.md"),
    ];
    if all.0 != Some(0) || generated != want {
        mismatches.push(format!(
            "show --batch --all --path apps: want {want:?}, exit 0; got {generated:?}, {all:?}"
        ));
    }

    let sites = "apps/api/docs/FS-api-audit.md:3: \u{a7}FS-api.format.1\n\
                 apps/api/src/lib.rs:1: \u{a7}FS-api.format.1\n";
    let glob = ["refs", "--glob", "*/FS-*{,.**}", "apps"];
    ws.expect(&mut mismatches, "", &glob, (Some(0), sites, None));
    ws.expect(
        &mut mismatches,
        "",
        &["refs", "web/FS-web", "apps"],
        (Some(0), "", None),
    );
    let api_ids = ["complete", "ids", "--path", "apps", "--prefix", "api/FS-"];
    let offered = "api/FS-api\napi/FS-api-audit\n";
    ws.expect(&mut mismatches, "", &api_ids, (Some(0), offered, Some("")));
    let web_ids = ["complete", "ids", "--path", "apps", "--prefix", "web/FS-"];
    ws.expect(&mut mismatches, "", &web_ids, NOTHING);
    assert_none(&mismatches);
}
