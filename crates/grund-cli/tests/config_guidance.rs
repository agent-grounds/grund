//! Loaded-version repair vocabulary: §FS-show.3.5.1,
//! §FS-declarations.checks.declaration-near-miss.4, §FS-check.4.5.2,
//! §FS-check.2.2.2.1 and §FS-cli.2.3. These black-box runs port #817's
//! near-miss, nothing-recognized and help reproducer, then cover each hint adapter.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use grund_core::{RefsOpts, refs, refs_outcome};

static NEXT: AtomicU64 = AtomicU64::new(0);
const GLOBAL: &str = "{kind}-{slug}";
const OVERRIDE: &str = "{kind}_{number}";

struct Fixture {
    root: PathBuf,
    version: u8,
    kind_override: bool,
}

impl Fixture {
    fn new(version: u8, kind_override: bool) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-817-guidance-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("docs/adr")).unwrap();
        let config = if version == 2 {
            format!(
                "grund_config_version = 2\nproject_name = \"guidance\"\n\
                 [schema]\nid_format = \"{GLOBAL}\"\n\
                 [schema.sources]\nexclude = []\n\
                 [schema.kinds.ADR]\nfolders = [\"docs/adr\"]\nindex = false\n{}",
                if kind_override {
                    format!("id_format = \"{OVERRIDE}\"\n")
                } else {
                    String::new()
                }
            )
        } else {
            format!(
                "grund_config_version = 1\nproject_name = \"guidance\"\n\
                 [id]\nformat = \"{GLOBAL}\"\n\
                 slug_pattern = \"[a-z][a-z0-9-]*\"\n\
                 [scan]\ninclude = [\"docs\"]\n\
                 [[kinds]]\nkind = \"ADR\"\nfolder = \"docs/adr\"\nindex = false\n{}",
                if kind_override {
                    format!("format = \"{OVERRIDE}\"\n")
                } else {
                    String::new()
                }
            )
        };
        fs::write(root.join("grund.toml"), config).unwrap();
        let fixture = Self {
            root,
            version,
            kind_override,
        };
        // A config error must never masquerade as the wording regression.
        let loaded = fixture.run(&["config", "show"]);
        assert_eq!(loaded.status.code(), Some(0), "{}", err(&loaded));
        assert!(out(&loaded).contains(&format!("grund_config_version = {version}")));
        fixture
    }

    fn write(&self, text: &str) {
        fs::write(self.root.join("docs/adr/notes.md"), text).unwrap();
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .current_dir(&self.root)
            .args(args)
            .output()
            .expect("run grund")
    }

    fn key(&self) -> &'static str {
        match (self.version, self.kind_override) {
            (2, true) => "[schema.kinds.ADR] id_format",
            (2, false) => "[schema] id_format",
            (_, true) => "[[kinds]] format",
            (_, false) => "[id] format",
        }
    }

    fn format(&self) -> &'static str {
        if self.kind_override { OVERRIDE } else { GLOBAL }
    }

    fn assert_hint(&self, text: &str) {
        assert!(
            text.contains(self.key()),
            "expected {} in:\n{text}",
            self.key()
        );
        assert!(
            text.contains(self.format()),
            "expected {} in:\n{text}",
            self.format()
        );
        if self.version == 2 {
            assert!(!text.contains("[id] format"), "{text}");
        }
        if self.kind_override {
            assert!(
                !text.contains(GLOBAL),
                "override replaced by global: {text}"
            );
        }
    }

    fn refs_opts(&self, id: &str) -> RefsOpts {
        let mut opts = RefsOpts::default();
        opts.path = self.root.clone();
        opts.path_provided = true;
        opts.id = id.into();
        opts
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn out(output: &Output) -> &str {
    std::str::from_utf8(&output.stdout).unwrap()
}

fn err(output: &Output) -> &str {
    std::str::from_utf8(&output.stderr).unwrap()
}

fn near_miss(version: u8, kind_override: bool) {
    let fixture = Fixture::new(version, kind_override);
    fixture.write("# ADR-0001: Use Postgres\n");
    let output = fixture.run(&["check", "--format", "json"]);
    assert_eq!(output.status.code(), Some(1), "{}", err(&output));
    let findings: Vec<serde_json::Value> = out(&output)
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let finding = findings
        .iter()
        .find(|f| f["code"] == "declaration-near-miss")
        .expect("the same declaration-near-miss finding");
    assert_eq!(finding["severity"], "error");
    assert_eq!(finding["path"], "docs/adr/notes.md");
    assert_eq!(finding["line"], 1);
    let message = finding["message"].as_str().unwrap();
    assert!(message.contains("resolves for compatibility"), "{message}");
    fixture.assert_hint(message);
}

fn query(version: u8, kind_override: bool, surface: &str) {
    let fixture = Fixture::new(version, kind_override);
    fixture.write("# Notes\n");
    if surface == "core-refs" {
        let error = refs(fixture.refs_opts("ADR-invalid!")).unwrap_err();
        let text = format!("{error:#}");
        assert!(text.starts_with("invalid ID `ADR-invalid!`\n"), "{text}");
        fixture.assert_hint(&text);
        return;
    }
    if surface == "core-qualified" {
        let error = refs_outcome(fixture.refs_opts("BAD/ADR-invalid!")).unwrap_err();
        let text = format!("{error:#}");
        assert!(text.starts_with("invalid project alias"), "{text}");
        // An alias refusal has selected no target kind: use the current global format.
        let key = if version == 2 {
            "[schema] id_format"
        } else {
            "[id] format"
        };
        assert!(
            text.contains(key) && text.contains(GLOBAL),
            "expected {key} and {GLOBAL}: {text}"
        );
        return;
    }
    let args = match surface {
        "show" => vec!["show", "ADR-invalid!"],
        "bare" => vec!["ADR-invalid!"],
        "refs" => vec!["refs", "ADR-invalid!"],
        _ => unreachable!(),
    };
    let output = fixture.run(&args);
    assert_eq!(output.status.code(), Some(1), "{}", err(&output));
    assert_eq!(out(&output), "");
    assert!(
        err(&output).starts_with("invalid ID `ADR-invalid!`\n"),
        "{}",
        err(&output)
    );
    fixture.assert_hint(err(&output));
    // Human-readable keys must not leak into the structured query envelope.
    let mut json_args = args;
    json_args.extend(["--format", "json"]);
    let json = fixture.run(&json_args);
    assert_eq!(json.status.code(), Some(1));
    assert_eq!(out(&json), "");
    let failure: serde_json::Value = serde_json::from_str(err(&json)).unwrap();
    assert_eq!(failure["code"], "invalid-id");
    assert_eq!(failure["message"], "invalid ID `ADR-invalid!`");
    assert_eq!(failure.as_object().unwrap().len(), 7);
}

macro_rules! red_query {
    ($name:ident, $version:expr, $override:expr, $surface:expr) => {
        #[test]
        fn $name() {
            query($version, $override, $surface);
        }
    };
}

red_query!(v2_global_show, 2, false, "show");
red_query!(v2_global_bare, 2, false, "bare");
red_query!(v2_global_refs, 2, false, "refs");
red_query!(v2_global_core_refs, 2, false, "core-refs");
red_query!(v2_global_core_qualified, 2, false, "core-qualified");
red_query!(v2_override_show, 2, true, "show");
red_query!(v2_override_bare, 2, true, "bare");
red_query!(v2_override_refs, 2, true, "refs");
red_query!(v2_override_core_refs, 2, true, "core-refs");
red_query!(v1_override_show, 1, true, "show");
red_query!(v1_override_bare, 1, true, "bare");
red_query!(v1_override_refs, 1, true, "refs");
red_query!(v1_override_core_refs, 1, true, "core-refs");

#[test]
fn v1_global_positive_controls() {
    near_miss(1, false);
    for surface in ["show", "bare", "refs", "core-refs", "core-qualified"] {
        query(1, false, surface);
    }
    nothing_recognized(1);
}

/// §FS-show.3.5.1: selecting repair wording must preserve valid override queries.
#[test]
fn kind_override_resolution_controls() {
    for version in [1, 2] {
        let fixture = Fixture::new(version, true);
        fixture.write("# ADR_001: Choice\n\nA decision.\n");
        let output = fixture.run(&["show", "ADR_001"]);
        assert_eq!(output.status.code(), Some(0), "{}", err(&output));
        assert!(out(&output).contains("A decision."), "{}", out(&output));
    }
}

#[test]
fn v2_global_near_miss() {
    near_miss(2, false);
}

#[test]
fn v2_override_near_miss() {
    near_miss(2, true);
}

#[test]
fn v1_override_near_miss() {
    near_miss(1, true);
}

fn nothing_recognized(version: u8) {
    let fixture = Fixture::new(version, false);
    fixture.write("# Notes\n\nA prose line.\n");
    let output = fixture.run(&["check"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(out(&output), "");
    assert!(
        err(&output).contains("nothing recognized — grund read 1 file"),
        "{}",
        err(&output)
    );
    fixture.assert_hint(err(&output));
}

#[test]
fn v2_global_nothing_recognized() {
    nothing_recognized(2);
}

#[test]
fn v2_excluded_root_scope() {
    let fixture = Fixture::new(2, false);
    fixture.write("# Notes\n");
    let path = fixture.root.join("grund.toml");
    let config = fs::read_to_string(&path)
        .unwrap()
        .replace("exclude = []", "exclude = [\"**/*.md\"]");
    fs::write(path, config).unwrap();
    let output = fixture.run(&["check"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(out(&output), "");
    let text = err(&output);
    assert!(text.contains("nothing to scan"), "{text}");
    assert!(!text.contains("[scan] include"), "{text}");
    assert!(
        text.contains("[schema.sources] exclude"),
        "expected source-exclusion repair in:\n{text}"
    );
}

/// §FS-check.1.3.7: the caution's scope label must describe a v2 project.
#[test]
fn v2_explicit_full_scope_caution() {
    let fixture = Fixture::new(2, false);
    fixture.write("# ADR-choice: Choice\n\n§ADR-choice\n");
    let output = fixture.run(&["check", "docs", "--full"]);
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    assert_eq!(out(&output), "");
    let text = err(&output);
    assert!(
        text.contains("--full has no effect with an explicit PATH"),
        "{text}"
    );
    assert!(
        !text.contains("[scan] include"),
        "v2 caution names a v1 key:\n{text}"
    );
}

/// §FS-check.3.14.6: the tier label is also live source-scope guidance.
#[test]
fn v2_full_outside_scope_label() {
    let fixture = Fixture::new(2, false);
    fixture.write("# ADR-choice: Choice\n\n§ADR-choice\n");
    let path = fixture.root.join("grund.toml");
    let mut config = fs::read_to_string(&path).unwrap();
    config.push_str(
        "\n[schema.kinds.archive]\ncitable = false\nfolders = [\"archive\"]\nscan = false\n",
    );
    fs::write(path, config).unwrap();
    fs::create_dir_all(fixture.root.join("archive")).unwrap();
    fs::write(fixture.root.join("archive/old.md"), "§ADR-missing\n").unwrap();
    let output = fixture.run(&["check", "--full", "--format", "json"]);
    assert_eq!(output.status.code(), Some(1), "{}", err(&output));
    let finding: serde_json::Value = serde_json::from_str(out(&output)).unwrap();
    assert_eq!(finding["code"], "out-of-scope-dangling");
    assert_eq!(finding["severity"], "error");
    let text = finding["message"].as_str().unwrap();
    assert!(text.starts_with("outside "), "{text}");
    assert!(text.contains("unknown reference ADR-missing"), "{text}");
    assert!(
        !text.contains("[scan] include"),
        "v2 tier names a v1 key:\n{text}"
    );
}

/// §FS-check.4.10.5: another project may use either supported scope model.
#[test]
fn v2_unread_workspace_repair() {
    let fixture = Fixture::new(2, false);
    fixture.write("# ADR-choice: Choice\n");
    let path = fixture.root.join("grund.toml");
    let mut config = fs::read_to_string(&path).unwrap();
    config.push_str("\n[workspace]\nmembers = [\"member\"]\ninclude_root = false\n");
    fs::write(path, config).unwrap();
    fs::create_dir_all(fixture.root.join("member")).unwrap();
    fs::write(
        fixture.root.join("member/grund.toml"),
        "grund_config_version = 2\nproject_name = \"member\"\n[schema.kinds.ADR]\n",
    )
    .unwrap();
    let output = fixture.run(&["check"]);
    assert_eq!(output.status.code(), Some(0), "{}", err(&output));
    let text = err(&output);
    assert!(text.contains("no project scans"), "{text}");
    assert!(text.contains("include_root = true"), "{text}");
    assert!(
        !text.contains("[scan] include"),
        "v2 workspace repair names a v1 key:\n{text}"
    );
}

#[test]
fn check_help_scope_vocabulary() {
    // Help is static, including in a directory with an invalid config.
    let fixture = Fixture::new(2, false);
    fs::write(fixture.root.join("grund.toml"), "invalid config").unwrap();
    let output = fixture.run(&["check", "--help"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(err(&output), "");
    let text = out(&output);
    for directory in ["docs/", "e2e/", "src/"] {
        assert!(text.contains(directory), "{text}");
    }
    let full = text
        .lines()
        .find(|line| line.trim_start().starts_with("--full "))
        .unwrap();
    let distinguishes_versions = (text.contains("v1") || text.contains("version 1"))
        && (text.contains("v2") || text.contains("version 2"))
        && text.contains("places");
    assert!(
        !text.contains("[scan] include") || distinguishes_versions,
        "unqualified v1 scope advice:\n{text}"
    );
    assert!(
        full.contains("scope") || full.contains("places") || distinguishes_versions,
        "--full must explain configured scope: {full}"
    );
}
