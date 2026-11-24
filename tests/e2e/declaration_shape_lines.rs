//! Black-box: a field's line shows the heading to write, with its placement and content in
//! check's words (§FS-schema.4.1), in the managed block's `### Declaration shapes`
//! (§FS-init.2.3.5) and in `grund schema`. The triage reproducer's five cases are pinned as
//! the rendered words: `directly after` where check holds a field directly after another, in the
//! block (A) and in the view (B); a scalar's title as its value (C); a value chapter's named
//! roots and numbered parts (D); and a value kind's file form with the citation its first element
//! answers to (E), for a JSON, a YAML and a Markdown file, a folder home and a fetched kind
//! (§FS-schema.4.1), with front matter shown as the block at the top of the file
//! (§FS-schema.4.1). The table's legend says where a field's heading sits (§FS-schema.8),
//! Mermaid and DOT label the same lines (§FS-schema.9, §FS-schema.10), and `config migrate`
//! prints them in its refusal (§FS-config-migrate.5.3). A document written from each rendered
//! line as shown checks clean. A shape-rendering config renders the v49/v50 pair, sharing the
//! v47–v50 bump of #1408 (§FS-init.2.3.7), and a scoped block above an unscoped selection stays
//! stale `scoped guidance` (§FS-init.2.3.7). The red tests wait on #1373; the controls pass
//! before it and after.
#![cfg(unix)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The legend line §FS-schema.8 adds where some kind prints a field line.
const DEPTH_LINE: &str = "## = under a declaration written # <ID>: … · a field always sits one \
                          level below its declaration's heading";

/// The legend's three lines (§FS-schema.8), which stay.
const LEGEND: &str =
    "must = check error · warn = check warning · should = suggestion (check --suggestions)
may = allowed · must not / warn not / should not = forbidden at that strength
A|B = one obligation, met by citing either · — = no direction rule names it";

/// The triage reproducer's fixture: FS with placed fields, DS with a scalar and a value chapter,
/// and a JSON value kind.
const REPORT_CONFIG: &str = r#"grund_config_version = 2
project_name = "repro-1373"

[presentation]
rules = "root"

[schema.sources]
languages = ["markdown"]

[schema.kinds.FS]
folders = ["docs/fs"]
index = false

[schema.kinds.FS.fields.reason]
title = "Reason for Existence"
presence = "must"
position = "first"

[schema.kinds.FS.fields.terminology]
title = "Terminology"
presence = "must"
after = "reason"

[schema.kinds.FS.fields.requirements]
title = "Requirements"
presence = "must"
position = "last"

[schema.kinds.DS]
folders = ["docs/design"]
index = false

[schema.kinds.DS.fields.risk]
presence = "must"
position = "first"
one_of = ["Low", "Medium", "High"]

[schema.kinds.DS.fields.values]
title = "Values"
presence = "must"
content = "values"

[schema.kinds.CONST]
files = ["config/constants.json"]
form = "value"
"#;

/// The report's fixture with FS closed and an untitled field before its last, a front-matter
/// kind, and a value kind of every home: YAML, a Markdown folder, and a fetched file.
fn full_config() -> String {
    let edits = [
        (
            "folders = [\"docs/fs\"]\nindex = false\n",
            "folders = [\"docs/fs\"]\nindex = false\nclosed = \"must\"\n",
        ),
        (
            "[schema.kinds.FS.fields.requirements]",
            "[schema.kinds.FS.fields.notes]\npresence = \"may\"\n\n[schema.kinds.FS.fields.requirements]",
        ),
        ("[schema.kinds.CONST]", ADR_ROW),
    ];
    let mut config = REPORT_CONFIG.to_owned();
    for (anchor, replacement) in edits {
        assert_eq!(
            config.matches(anchor).count(),
            1,
            "one {anchor:?} in the report's config"
        );
        config = config.replace(anchor, replacement);
    }
    config + VALUE_ROWS
}

/// The front-matter kind, written ahead of the report's `CONST` row.
const ADR_ROW: &str = r#"[schema.kinds.ADR]
folders = ["docs/adr"]
index = false

[schema.kinds.ADR.fields.front-matter]
presence = "must"
closed = "must"

[schema.kinds.ADR.fields.front-matter.keys.status]
presence = "must"
one_of = ["proposed", "accepted"]

[schema.kinds.ADR.fields.front-matter.keys.date]
presence = "must"

[schema.kinds.ADR.fields.context]
presence = "must"

[schema.kinds.CONST]"#;

/// The value kinds after the report's `CONST`: a YAML file, a Markdown folder, a fetched file.
const VALUE_ROWS: &str = r#"
[schema.kinds.YCONST]
files = ["config/constants.yaml"]
form = "value"

[schema.kinds.MCONST]
folders = ["consts"]
index = false
form = "value"

[schema.kinds.FCONST]
files = ["values/limits.md"]
form = "value"
fetch = "scripts/fetch-const"
"#;

/// The full fixture's `### Declaration shapes`, as §FS-init.2.3.5 prints it.
const FULL_SECTION: &str = "### Declaration shapes

`grund check` holds each declaration of these kinds to these fields, as `grund schema` lists them. Each heading is shown under a declaration written `# <ID>: …`; a field always sits one level below its declaration's heading:

- **FS**
  - `## reason: Reason for Existence` (must, first)
  - `## terminology: Terminology` (must, directly after reason)
  - `## notes: <title>` (may)
  - `## requirements: Requirements` (must, last)
  - no other chapter except numbered ones (must)
- **DS**
  - `## risk: <one of Low, Medium, High>` (must, first): the title is the value
  - `## values: Values` (must): only values, each `### values.<name>: <title>` with parts `#### .1: <number or word>`; a unit is a part of its own
- **ADR**
  - `---` YAML or `+++` TOML front matter at the top of the file (must)
    - `status: <one of proposed, accepted>` (must)
    - `date: <value>` (must)
    - no other key (must)
  - `## context: <title>` (must)
- **CONST**
  - `{\"<ID>\": [<number or word>, …], …}` in `config/constants.json`: part 1 is `\u{a7}<ID>.1`; a unit is a part of its own
- **YCONST**
  - `<ID>: [<number or word>, …]` in `config/constants.yaml`: part 1 is `\u{a7}<ID>.1`; a unit is a part of its own
- **MCONST**
  - `# <ID>: <title>` with only `## 1. <number or word>`, `## 2. …` under it in a Markdown file in `consts/`, `{\"<ID>\": [<number or word>, …], …}` in a JSON file in `consts/`, or `<ID>: [<number or word>, …]` in a YAML file in `consts/`: part 1 is `\u{a7}<ID>.1`; a unit is a part of its own
- **FCONST**
  - written by `grund fetch` into `values/limits.md`: part 1 is `\u{a7}<ID>.1`";

/// The kinds the full fixture's section lists, in order.
const FULL_KINDS: [&str; 7] = ["FS", "DS", "ADR", "CONST", "YCONST", "MCONST", "FCONST"];

/// A project in a fresh directory of its own, removed when dropped.
struct Tree {
    root: PathBuf,
}

impl Tree {
    fn new(test: &str, config: &str) -> Self {
        let root = std::env::temp_dir().join(format!("grund-1373-{test}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let tree = Self { root };
        tree.write("grund.toml", config);
        let status = Command::new("git")
            .args(["init", "-q", "."])
            .current_dir(&tree.root)
            .status()
            .expect("run git init");
        assert!(status.success(), "git init failed");
        tree
    }

    /// The report's fixture, with a valid value file so its other kinds read cleanly.
    fn report(test: &str) -> Self {
        let tree = Self::new(test, REPORT_CONFIG);
        tree.write("config/constants.json", "{\"CONST-seed\": [1]}\n");
        tree.dir("docs/design");
        tree
    }

    /// The full fixture, every value home present as check requires.
    fn full(test: &str) -> Self {
        let tree = Self::new(test, &full_config());
        tree.write("config/constants.json", "{\"CONST-seed\": [1]}\n");
        tree.write("config/constants.yaml", "YCONST-seed: [1]\n");
        tree.write("values/limits.md", "");
        tree.write("scripts/fetch-const", "#!/bin/sh\nexit 99\n");
        tree.dir("docs/design");
        tree.dir("consts");
        tree
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("a parent")).expect("create the parent");
        fs::write(path, content).expect("write the fixture");
    }

    fn dir(&self, relative: &str) {
        fs::create_dir_all(self.root.join(relative)).expect("create the directory");
    }

    fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.root.join(relative)).expect("read the file")
    }

    fn grund(&self, args: &[&str]) -> Run {
        run_in(&self.root, args)
    }

    /// Writes the managed block and says what it rendered.
    fn init(&self) -> String {
        let run = self.grund(&["init", "--agents-md"]);
        assert_eq!(run.code, 0, "{}", run.report());
        self.read("AGENTS.md")
    }

    /// Rewrites the block's `(block vN)` heading to `version`, as an older or newer grund wrote it.
    fn set_block_version(&self, version: u32) {
        let text = self.read("AGENTS.md");
        let start = text.find("(block v").expect("a version heading") + "(block v".len();
        let end = start + text[start..].find(')').expect("a closing parenthesis");
        self.write(
            "AGENTS.md",
            &format!("{}{version}{}", &text[..start], &text[end..]),
        );
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn run_in(dir: &Path, args: &[&str]) -> Run {
    let output = Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .env("NO_COLOR", "1")
        .current_dir(dir)
        .output()
        .expect("run grund");
    Run {
        shown: format!("grund {}", args.join(" ")),
        code: output.status.code().expect("exited with a status"),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

struct Run {
    shown: String,
    code: i32,
    stdout: String,
    stderr: String,
}

impl Run {
    fn report(&self) -> String {
        let Self {
            shown,
            code,
            stdout,
            stderr,
        } = self;
        format!("{shown}: exit {code}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    }

    /// No `error:` line on either stream: a warning, such as a declaration nothing cites, is fine.
    fn assert_no_error(&self) {
        let errors = self
            .stdout
            .lines()
            .chain(self.stderr.lines())
            .any(|line| line.starts_with("error:") || line.contains(": error:"));
        assert!(!errors && self.code != 2, "{}", self.report());
    }

    /// `stdout` holds `line` exactly.
    fn assert_line(&self, line: &str) {
        assert!(
            self.stdout.lines().any(|l| l == line),
            "expected the line\n{line}\n{}",
            self.report()
        );
    }
}

/// The block's `### Declaration shapes` section, from its heading to the next.
fn shapes_section(agents: &str) -> String {
    let start = agents
        .find("### Declaration shapes")
        .unwrap_or_else(|| panic!("no Declaration shapes section in\n{agents}"));
    let rest = &agents[start..];
    let end = rest[1..]
        .find("\n### ")
        .or_else(|| rest[1..].find("\n<!-- END"))
        .map_or(rest.len(), |at| at + 1);
    rest[..end].trim_end().to_owned()
}

/// The `(block vN)` the rendered block carries.
fn block_version(agents: &str) -> String {
    let start = agents.find("(block v").expect("a version heading");
    let end = start + agents[start..].find(')').expect("a closing parenthesis");
    agents[start + 1..end].to_owned()
}

#[test]
fn block_shows_each_field_as_the_heading_to_write() {
    let tree = Tree::full("block");
    let section = shapes_section(&tree.init());
    assert_eq!(
        section, FULL_SECTION,
        "the block's Declaration shapes section"
    );
}

#[test]
fn schema_table_shows_each_field_as_the_heading_to_write() {
    let tree = Tree::report("table");
    let run = tree.grund(&["schema"]);
    assert_eq!(run.code, 0, "{}", run.report());
    let expected = format!(
        "repro-1373 · grund.toml · v2

kinds and places
  FS     kind                  docs/fs/
           field reason        `## reason: Reason for Existence` (must, first)
           field terminology   `## terminology: Terminology` (must, directly after reason)
           field requirements  `## requirements: Requirements` (must, last)
  DS     kind                  docs/design/
           field risk          `## risk: <one of Low, Medium, High>` (must, first): the title is the value
           field values        `## values: Values` (must): only values, each `### values.<name>: <title>` with parts `#### .1: <number or word>`; a unit is a part of its own
  CONST  kind                  config/constants.json
           field <number>      `{{\"<ID>\": [<number or word>, …], …}}` in `config/constants.json`: part 1 is `\u{a7}<ID>.1`; a unit is a part of its own
  code   complement            every scanned file no other row claims

citations
  FS     —
  DS     —
  CONST  —
  code   —

{LEGEND}
{DEPTH_LINE}
"
    );
    assert_eq!(run.stdout, expected, "{}", run.report());
}

#[test]
fn schema_table_shows_front_matter_and_every_value_home() {
    let tree = Tree::full("table-full");
    let run = tree.grund(&["schema"]);
    assert_eq!(run.code, 0, "{}", run.report());
    for line in [
        "            field notes         `## notes: <title>` (may)",
        "            closed              no other chapter except numbered ones (must)",
        "            field front-matter  `---` YAML or `+++` TOML front matter at the top of the file (must)",
        "              key status        `status: <one of proposed, accepted>` (must)",
        "              key date          `date: <value>` (must)",
        "              closed            no other key (must)",
        "            field context       `## context: <title>` (must)",
        "            field <number>      `<ID>: [<number or word>, …]` in `config/constants.yaml`: part 1 is `\u{a7}<ID>.1`; a unit is a part of its own",
        "            field <number>      `# <ID>: <title>` with only `## 1. <number or word>`, `## 2. …` under it in a Markdown file in `consts/`, `{\"<ID>\": [<number or word>, …], …}` in a JSON file in `consts/`, or `<ID>: [<number or word>, …]` in a YAML file in `consts/`: part 1 is `\u{a7}<ID>.1`; a unit is a part of its own",
        "            field <number>      written by `grund fetch` into `values/limits.md`: part 1 is `\u{a7}<ID>.1`",
    ] {
        run.assert_line(line);
    }
}

/// The issue's RFC kind, from the `schema-*-field-title` cases.
const RFC_CONFIG: &str = r#"grund_config_version = 2
project_name = "skerry"

[schema]
id_format = "{kind}-{number}-{slug}"

[schema.kinds.RFC]
folders = ["rfcs/text"]
index = false
closed = "must"

[schema.kinds.RFC.fields.summary]
title = "Summary"
presence = "must"
position = "first"

[schema.kinds.RFC.fields.motivation]
title = "Motivation"
presence = "must"
after = "summary"

[schema.kinds.RFC.fields.guide-level-explanation]
title = "Guide-level explanation"
presence = "must"
after = "motivation"

[schema.kinds.RFC.fields.drawbacks]
title = "Drawbacks & \"open\" *questions*"
presence = "may"
after = "guide-level-explanation"

[schema.kinds.RFC.fields.amendments]
presence = "may"
position = "last"

[presentation]
rules = "root"
"#;

/// The comment lines Mermaid and DOT share; neither gains the table's depth line.
const GRAPH_COMMENTS: [&str; 4] = [
    "skerry · grund.toml · v2",
    "must = check error · warn = check warning · should = suggestion (check --suggestions)",
    "may = allowed · must not / warn not / should not = forbidden at that strength",
    "any of = one obligation, met by citing any one of its arrows",
];

#[test]
fn mermaid_labels_the_shape_lines() {
    let tree = Tree::new("mermaid", RFC_CONFIG);
    let [header, must, may, any] = GRAPH_COMMENTS;
    let mermaid = tree.grund(&["schema", "--format", "mermaid"]);
    let expected = format!(
        "flowchart LR
  %% {header}
  %% {must}
  %% {may}
  %% {any}
  classDef place stroke-dasharray:5 5
  classDef unscanned stroke-dasharray:2 2
  subgraph unruled[\"no direction rule names these\"]
    n0[\"RFC<br/>kind<br/>rfcs/text/<br/>field summary: `## summary: Summary` (must, first)<br/>field motivation: `## motivation: Motivation` (must, directly after summary)<br/>field guide-level-explanation: `## guide-level-explanation: Guide-level explanation` (must, directly after motivation)<br/>field drawbacks: `## drawbacks: Drawbacks #amp; #quot;open#quot; *questions*` (may, directly after guide-level-explanation)<br/>field amendments: `## amendments: #lt;title#gt;` (may, last)<br/>closed: no other chapter except numbered ones (must)\"]
    n1[\"code<br/>complement<br/>every scanned file no other row claims\"]
  end
"
    );
    assert_eq!(mermaid.stdout, expected, "{}", mermaid.report());
}

#[test]
fn dot_labels_the_shape_lines() {
    let tree = Tree::new("dot", RFC_CONFIG);
    let [header, must, may, any] = GRAPH_COMMENTS;
    let dot = tree.grund(&["schema", "--format", "dot"]);
    let expected = format!(
        "digraph schema {{
  // {header}
  // {must}
  // {may}
  // {any}
  rankdir=LR;
  node [shape=box];
  subgraph cluster_unruled {{
    label=\"no direction rule names these\";
    n0 [label=\"RFC\\nkind\\nrfcs/text/\\nfield summary: `## summary: Summary` (must, first)\\nfield motivation: `## motivation: Motivation` (must, directly after summary)\\nfield guide-level-explanation: `## guide-level-explanation: Guide-level explanation` (must, directly after motivation)\\nfield drawbacks: `## drawbacks: Drawbacks & \\\"open\\\" *questions*` (may, directly after guide-level-explanation)\\nfield amendments: `## amendments: <title>` (may, last)\\nclosed: no other chapter except numbered ones (must)\"];
    n1 [label=\"code\\ncomplement\\nevery scanned file no other row claims\"];
  }}
}}
"
    );
    assert_eq!(dot.stdout, expected, "{}", dot.report());
}

/// The table node §FS-schema.4.1 shows for the issue's RFC kind, with the case's `drawbacks`.
#[test]
fn schema_table_shows_the_issues_rfc_kind() {
    let tree = Tree::new("table-rfc", RFC_CONFIG);
    let run = tree.grund(&["schema"]);
    assert_eq!(run.code, 0, "{}", run.report());
    let node = "  RFC   kind                             rfcs/text/
          field summary                  `## summary: Summary` (must, first)
          field motivation               `## motivation: Motivation` (must, directly after summary)
          field guide-level-explanation  `## guide-level-explanation: Guide-level explanation` (must, directly after motivation)
          field drawbacks                `## drawbacks: Drawbacks & \"open\" *questions*` (may, directly after guide-level-explanation)
          field amendments               `## amendments: <title>` (may, last)
          closed                         no other chapter except numbered ones (must)
";
    assert!(
        run.stdout.contains(node),
        "expected the node\n{node}\n{}",
        run.report()
    );
}

#[test]
fn legend_says_where_a_field_heading_sits_under_a_level_two_declaration() {
    // The traceability example declares `## SWR-001: …`, so its fields sit at `###`.
    let repo =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/traceability/1-schema/repo");
    let run = run_in(&repo, &["schema"]);
    assert_eq!(run.code, 0, "{}", run.report());
    let lines: Vec<&str> = run.stdout.lines().collect();
    let tail = lines[lines.len().saturating_sub(4)..].join("\n");
    assert_eq!(tail, format!("{LEGEND}\n{DEPTH_LINE}"), "{}", run.report());
    let field = "field verification  `## verification: <one of Inspection, Analysis, Demonstration, \
                 Test>` (must, first): the title is the value";
    assert!(
        lines.iter().any(|line| line.trim_start() == field),
        "expected a line\n{field}\n{}",
        run.report()
    );
}

#[test]
fn legend_without_fields_keeps_its_three_lines() {
    let tree = Tree::new(
        "legend-plain",
        "grund_config_version = 2\nproject_name = \"plain\"\n\n[schema.kinds.FS]\n\
         folders = [\"docs/fs\"]\nindex = false\n",
    );
    let run = tree.grund(&["schema"]);
    assert_eq!(run.code, 0, "{}", run.report());
    assert!(
        run.stdout.ends_with(&format!("\n\n{LEGEND}\n")),
        "{}",
        run.report()
    );
    assert!(!run.stdout.contains(DEPTH_LINE), "{}", run.report());
}

/// A config that renders no declaration shape and no scoped file: the unscoped pair.
const UNSCOPED_CONFIG: &str = "grund_config_version = 2\nproject_name = \"plain\"\n\n\
                               [schema.kinds.FS]\nfolders = [\"docs/fs\"]\nindex = false\n";

/// A config that renders a declaration shape, so the scoped pair; with a rule kind on request.
fn scoped_config(rule_kind: bool) -> String {
    let rule = if rule_kind {
        "\n[schema.kinds.RULE]\nfolders = [\"docs/rules\"]\nform = \"rule\"\n"
    } else {
        ""
    };
    format!(
        "grund_config_version = 2\nproject_name = \"scoped\"\n\n[presentation]\nrules = \"root\"\n\n\
         [schema.kinds.FS]\nfolders = [\"docs/fs\"]\nindex = false\n\n\
         [schema.kinds.FS.fields.reason]\npresence = \"must\"\nposition = \"first\"\n{rule}"
    )
}

/// The line check prints for a block older than the selected revision.
fn outdated(found: u32, selected: u32) -> String {
    format!(
        "AGENTS.md:3: error: repo maintenance: outdated grund init block v{found} — run `grund \
         init` to update to v{selected} (does not affect citation validity)"
    )
}

/// The line check prints for a block of the unselected scoped member (§FS-init.2.3.7).
const STALE_SCOPED: &str = "AGENTS.md:3: error: repo maintenance: stale grund init block: the \
                            scoped guidance section differs from grund.toml — run `grund init` \
                            to refresh (does not affect citation validity)";

#[test]
fn a_shape_rendering_config_selects_the_new_scoped_pair() {
    for (rule_kind, selected, older) in [(false, 49, 45), (true, 50, 46)] {
        let tree = Tree::new(&format!("scoped-{selected}"), &scoped_config(rule_kind));
        tree.dir("docs/rules");
        let agents = tree.init();
        assert_eq!(block_version(&agents), format!("block v{selected}"));
        tree.grund(&["check"]).assert_no_error();
        tree.set_block_version(older);
        let run = tree.grund(&["check"]);
        assert_eq!(run.code, 1, "{}", run.report());
        run.assert_line(&outdated(older, selected));
    }
}

#[test]
fn a_new_scoped_block_under_an_unscoped_selection_is_stale_scoped_guidance() {
    for version in [49, 50] {
        let tree = Tree::new(&format!("unscoped-{version}"), UNSCOPED_CONFIG);
        tree.init();
        tree.set_block_version(version);
        let run = tree.grund(&["check"]);
        assert_eq!(run.code, 1, "{}", run.report());
        run.assert_line(STALE_SCOPED);
    }
}

#[test]
fn an_unscoped_config_renders_v47_and_checks_clean() {
    let tree = Tree::new("unscoped", UNSCOPED_CONFIG);
    assert_eq!(block_version(&tree.init()), "block v47");
    tree.grund(&["check"]).assert_no_error();
}

/// A version-1 project whose `TERM` row is a value row in `terms.md`, from the
/// `config-migrate-refuse-values-*` cases; `fetch` makes it a fetched one.
fn migrate_tree(test: &str, fetch: bool) -> Tree {
    let fetch_line = if fetch {
        "fetch = \"scripts/fetch\"\n"
    } else {
        ""
    };
    let tree = Tree::new(
        test,
        &format!(
            "grund_config_version = 1\nproject_name = \"example\"\n\n[id]\n\
             format = \"{{kind}}-{{slug}}\"\nnamed_sections = true\n\
             slug_pattern = \"[a-z][a-z0-9-]*\"\n\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\n\n\
             [[kinds]]\nkind = \"TERM\"\nfile = \"terms.md\"\nvalues = true\n{fetch_line}\n\
             [scan]\ninclude = [\"docs\", \"terms.md\"]\nextensions = [\"md\"]\n"
        ),
    );
    tree.write("terms.md", "# Terms\n\n## TERM-a: A\n\nThe first term.\n");
    tree.write("docs/FS-a.md", "# FS-a: A\n\nText.\n");
    tree.write(
        "scripts/fetch",
        "#!/bin/sh\ntouch fetcher-must-not-run\nexit 99\n",
    );
    tree
}

#[test]
fn config_migrate_refusal_prints_the_value_kind_shape_line() {
    let would_write = [
        (
            false,
            "  would write:   - `# <ID>: <title>` with only `## 1. <number or word>`, `## 2. …` under \
             it in `terms.md`: part 1 is `\u{a7}<ID>.1`; a unit is a part of its own",
        ),
        (
            true,
            "  would write:   - written by `grund fetch` into `terms.md`: part 1 is `\u{a7}<ID>.1`",
        ),
    ];
    for (fetch, line) in would_write {
        let tree = migrate_tree(&format!("migrate-{fetch}"), fetch);
        let run = tree.grund(&["config", "migrate", "--to", "2", "."]);
        assert_eq!(run.code, 1, "{}", run.report());
        assert!(
            run.stderr.lines().any(|l| l == line),
            "expected the line\n{line}\n{}",
            run.report()
        );
        assert!(!run.stderr.contains("field <number>"), "{}", run.report());
    }
}

#[test]
fn a_document_written_as_each_rendered_line_shows_checks_clean() {
    let tree = Tree::full("as-shown");
    let section = shapes_section(&tree.init());
    write_as_shown(&tree, &section);
    tree.grund(&["check"]).assert_no_error();
}

#[test]
fn a_document_written_as_the_specified_lines_show_checks_clean_today() {
    // The words §FS-schema.4.1 specifies already describe documents check accepts.
    let tree = Tree::full("as-specified");
    tree.init();
    write_as_shown(&tree, FULL_SECTION);
    tree.grund(&["check"]).assert_no_error();
}

/// One run of a bullet's text: plain words, or a code span's content.
enum Piece {
    Text(String),
    Code(String),
}

/// Splits `text` into its plain runs and its code spans (§FS-schema.4.1).
fn pieces(text: &str) -> Vec<Piece> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find('`') {
        if open > 0 {
            out.push(Piece::Text(rest[..open].to_owned()));
        }
        let fence = rest[open..].len() - rest[open..].trim_start_matches('`').len();
        let body = &rest[open + fence..];
        let close = body
            .find(&"`".repeat(fence))
            .unwrap_or_else(|| panic!("an unclosed code span in {text:?}"));
        let mut code = &body[..close];
        if code.len() > 1 && code.starts_with(' ') && code.ends_with(' ') && !code.trim().is_empty()
        {
            code = &code[1..code.len() - 1];
        }
        out.push(Piece::Code(code.to_owned()));
        rest = &body[close + fence..];
    }
    if !rest.is_empty() {
        out.push(Piece::Text(rest.to_owned()));
    }
    out
}

/// Fills a code span's placeholders as an author would (§FS-schema.4.1), refusing one it
/// does not know.
fn fill(code: &str, id: &str) -> String {
    let mut text = code.replace(", …", "");
    while let Some(start) = text.find("<one of ") {
        let end = start + text[start..].find('>').expect("a closing bracket");
        let first = text[start + "<one of ".len()..end]
            .split(", ")
            .next()
            .expect("a value")
            .to_owned();
        text.replace_range(start..=end, &first);
    }
    let text = text
        .replace("<title>", "Shown")
        .replace("<name>", "shown")
        .replace("<value>", "shown")
        .replace("<number or word>", "30")
        .replace("<ID>", id);
    assert!(
        !text.contains('<'),
        "a placeholder the guard cannot fill: {code:?}"
    );
    text
}

/// Writes, for each kind of `section`, a document exactly as its lines show, and a file citing
/// each value's first part as its line says it answers. Panics on a line it cannot read.
fn write_as_shown(tree: &Tree, section: &str) {
    let homes = BTreeMap::from([
        ("FS", "docs/fs"),
        ("DS", "docs/design"),
        ("ADR", "docs/adr"),
    ]);
    let mut kinds: Vec<(String, Vec<(String, Vec<String>)>)> = Vec::new();
    for line in section.lines().skip_while(|line| !line.starts_with("- **")) {
        if let Some(kind) = line.strip_prefix("- **").and_then(|l| l.strip_suffix("**")) {
            kinds.push((kind.to_owned(), Vec::new()));
        } else if let Some(bullet) = line.strip_prefix("  - ") {
            let (_, bullets) = kinds.last_mut().expect("a kind");
            bullets.push((bullet.to_owned(), Vec::new()));
        } else if let Some(nested) = line.strip_prefix("    - ") {
            let (_, bullets) = kinds.last_mut().expect("a kind");
            let (_, keys) = bullets.last_mut().expect("a bullet to nest under");
            keys.push(nested.to_owned());
        } else {
            panic!("a line the guard cannot read as shown: {line:?}");
        }
    }
    let names: Vec<&str> = kinds.iter().map(|(kind, _)| kind.as_str()).collect();
    assert_eq!(names, FULL_KINDS, "the kinds the section lists");
    let mut citations = Vec::new();
    let mut count = 0;
    for (kind, bullets) in &kinds {
        let (mut front, mut chapters) = (String::new(), String::new());
        for (bullet, keys) in bullets {
            let parts = pieces(bullet);
            match parts.first() {
                Some(Piece::Text(t)) if t.starts_with("no other ") => {}
                Some(Piece::Text(t)) if t.starts_with("written by ") => {}
                Some(Piece::Code(c)) if c.starts_with("## ") => {
                    chapters.push_str(&format!("{}\n\n", fill(c, "")));
                    let codes: Vec<&str> = parts[1..]
                        .iter()
                        .filter_map(|p| {
                            if let Piece::Code(c) = p {
                                Some(c.as_str())
                            } else {
                                None
                            }
                        })
                        .collect();
                    match codes.as_slice() {
                        [] => chapters.push_str("Text.\n\n"),
                        [root, part] => {
                            chapters.push_str(&format!(
                                "{}\n\n{}\n\n",
                                fill(root, ""),
                                fill(part, "")
                            ));
                        }
                        _ => panic!("a heading line the guard cannot read as shown: {bullet:?}"),
                    }
                }
                Some(Piece::Code(c)) if c == "---" => {
                    front.push_str("---\n");
                    for key in keys.iter().filter(|k| !k.starts_with("no other ")) {
                        match pieces(key).first() {
                            Some(Piece::Code(c)) => front.push_str(&format!("{}\n", fill(c, ""))),
                            _ => panic!("a key line the guard cannot read as shown: {key:?}"),
                        }
                    }
                    front.push_str("---\n");
                }
                Some(Piece::Code(_)) => {
                    count = write_values(tree, kind, &parts, count, &mut citations);
                }
                _ => panic!("a line the guard cannot read as shown: {bullet:?}"),
            }
        }
        if !front.is_empty() || !chapters.is_empty() {
            let home = homes
                .get(kind.as_str())
                .expect("a home for a kind with chapters");
            tree.write(
                &format!("{home}/shown.md"),
                &format!("{front}# {kind}-shown: Shown\n\n{}\n", chapters.trim_end()),
            );
        }
    }
    tree.write("uses.md", &format!("# Uses\n\n{}\n", citations.join("\n")));
}

/// Writes each value form a value kind's line shows where it says, and records the citation the
/// line says its first element answers to. Returns the next ID counter.
fn write_values(
    tree: &Tree,
    kind: &str,
    parts: &[Piece],
    mut count: usize,
    citations: &mut Vec<String>,
) -> usize {
    let (mut doc, mut ids, mut place) = (String::new(), Vec::new(), String::new());
    let mut cite = false;
    for part in parts {
        match part {
            Piece::Code(c) if cite => {
                citations.extend(ids.iter().map(|id: &String| format!("- {}", fill(c, id))));
                cite = false;
            }
            Piece::Code(c) if !place.is_empty() => {
                let id = ids.last().expect("a form before its place");
                let path = if c.ends_with('/') {
                    let ext = match place.as_str() {
                        "Markdown" => "md",
                        "JSON" => "json",
                        "YAML" => "yaml",
                        _ => panic!("a place the guard cannot read: {place:?}"),
                    };
                    format!("{c}{}.{ext}", id.to_lowercase())
                } else {
                    c.clone()
                };
                tree.write(&path, &format!("{}\n", doc.trim_end()));
                (doc, place) = (String::new(), String::new());
            }
            Piece::Code(c) if c.starts_with("## 2.") => {}
            Piece::Code(c) if c.starts_with("## 1.") => {
                doc.push_str(&format!("{}\n", fill(c, "")));
            }
            Piece::Code(c) if c.contains("<ID>") => {
                count += 1;
                let id = format!("{kind}-shown{count}");
                doc = format!("{}\n\n", fill(c, &id));
                ids.push(id);
            }
            Piece::Text(t) if t.starts_with(": part 1 is ") => cite = true,
            Piece::Text(t) if t.ends_with(" in ") => {
                place = match t.rsplit_once(" in a ") {
                    Some((_, form)) => form.trim_end_matches(" file in ").to_owned(),
                    None => "file".to_owned(),
                };
            }
            Piece::Text(_) => {}
            Piece::Code(c) => panic!("a code span the guard cannot read as shown: {c:?}"),
        }
    }
    assert!(
        !ids.is_empty() && doc.is_empty(),
        "a value line the guard cannot read as shown"
    );
    count
}
