//! Black-box: `grund check` holds the whole managed block (§FS-check.3.5.4), so the
//! two scenarios of grund#906 go red until `grund init` runs where the findings say,
//! and every way of naming the tree reports the same findings; and one `grund init`
//! per project of a fresh workspace, in either order, leaves none (grund#931).

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const V1: &str = "grund_config_version = 1
project_name = \"demo\"

[reference]
strict = true

[id]
format = \"{kind}-{slug}\"
named_sections = true

[[kinds]]
kind = \"FS\"
file = \"requirements.md\"

[scan]
include = [\"requirements.md\", \"src\"]
";

const SHOP: &str = "grund_config_version = 2
project_name = \"shop\"

[schema.kinds.FS]
files = [\"requirements.md\"]

[workspace]
members = [\"packages/billing\"]
";

struct Fixture(PathBuf);

impl Fixture {
    fn new(name: &str) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("grund-906-{name}-{}-{serial}", std::process::id()));
        fs::create_dir_all(&dir).expect("create fixture");
        let git = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&dir)
            .output()
            .expect("initialize fixture Git repository");
        assert!(git.status.success(), "git init: {}", text(&git.stderr));
        Self(dir)
    }

    fn write(&self, path: &str, contents: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    fn run_in(&self, dir: &str, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(self.0.join(dir))
            .output()
            .expect("run grund")
    }

    fn run(&self, args: &[&str]) -> Output {
        self.run_in(".", args)
    }

    fn init_in(&self, dir: &str) {
        let out = self.run_in(dir, &["init"]);
        assert_eq!(
            out.status.code(),
            Some(0),
            "init in {dir}: {}",
            text(&out.stderr)
        );
    }

    fn member(&self, name: &str) {
        self.write(
            &format!("packages/{name}/grund.toml"),
            &format!(
                "grund_config_version = 2\nproject_name = \"{name}\"\n\n\
                 [schema.kinds.FS]\nfiles = [\"spec.md\"]\n"
            ),
        );
        self.write(
            &format!("packages/{name}/spec.md"),
            &format!("# Spec\n\n# FS-{name}: {name}\n\nThe package.\n"),
        );
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove fixture");
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn agents_init_lines(out: &Output) -> Vec<String> {
    text(&out.stdout)
        .lines()
        .filter(|line| line.contains("grund init block"))
        .map(str::to_owned)
        .collect()
}

fn assert_current(fixture: &Fixture, dir: &str) {
    let out = fixture.run_in(dir, &["init", "--check"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "init --check in {dir}: {}",
        text(&out.stdout)
    );
}

/// §FS-config-migrate.5.3: `--write` is accepted, the next `check` reports the
/// block's config-describing text stale, and `grund init` clears it.
#[test]
fn config_migrate_write_then_check() {
    let fixture = Fixture::new("migrate");
    fixture.write("grund.toml", V1);
    fixture.write("requirements.md", "# FS-login: Login\n");
    fixture.write("src/a.py", "# Implements \u{a7}FS-login\n");
    fixture.init_in(".");
    assert!(agents_init_lines(&fixture.run(&["check"])).is_empty());

    let migrate = fixture.run(&["config", "migrate", "--to", "2", "--write"]);
    assert_eq!(
        migrate.status.code(),
        Some(0),
        "migrate: {}",
        text(&migrate.stderr)
    );

    let check = fixture.run(&["check"]);
    let stale = agents_init_lines(&check);
    assert_eq!(
        check.status.code(),
        Some(1),
        "check after migrate: {}",
        text(&check.stdout)
    );
    assert!(
        stale.iter().any(|line| line.starts_with(
            "AGENTS.md:3: error: repo maintenance: stale grund init block: citation scheme \
             differ from grund.toml"
        )),
        "no citation scheme finding: {stale:?}"
    );

    fixture.init_in(".");
    let check = fixture.run(&["check"]);
    assert_eq!(
        check.status.code(),
        Some(0),
        "check after init: {}",
        text(&check.stdout)
    );
    assert!(agents_init_lines(&check).is_empty());
}

/// §FS-init.requirements.clean, §FS-init.2.3.4.15.6: one `grund init` in each
/// project of a fresh workspace, root first or member first, leaves the root's
/// `check` green, and no block describes a sibling as not yet initialized.
/// grund#931: the project initialized first kept that mark and went stale.
#[test]
#[ignore = "red until #931 links each workspace member by its directory; implement removes this"]
fn one_init_per_project_in_either_order_leaves_the_root_check_green() {
    let stale: Vec<_> = [[".", "packages/billing"], ["packages/billing", "."]]
        .into_iter()
        .map(|order| {
            let fixture = Fixture::new("order");
            fixture.write("grund.toml", SHOP);
            fixture.write("requirements.md", "# FS-receipt: Receipts\n");
            fixture.member("billing");
            for dir in order {
                fixture.init_in(dir);
            }
            let check = fixture.run(&["check"]);
            let marked: Vec<_> = ["AGENTS.md", "packages/billing/AGENTS.md"]
                .into_iter()
                .filter(|block| {
                    fs::read_to_string(fixture.0.join(block))
                        .expect("entrypoint")
                        .contains("not yet initialized")
                })
                .collect();
            let current = order
                .iter()
                .all(|dir| fixture.run_in(dir, &["init", "--check"]).status.success());
            (
                order,
                check.status.code(),
                agents_init_lines(&check),
                marked,
                current,
            )
        })
        .filter(|(_, code, lines, marked, current)| {
            *code != Some(0) || !lines.is_empty() || !marked.is_empty() || !current
        })
        .collect();
    assert!(stale.is_empty(), "{stale:#?}");
}

/// §FS-check.3.5.5, §FS-errors.3.6.1: a new member turns three blocks stale, each
/// finding names where to run `grund init`, and running it there clears them all.
#[test]
#[ignore = "red until #931 links each workspace member by its directory; implement removes this"]
fn new_member_blocks_clear_after_init_in_each_named_directory() {
    let fixture = Fixture::new("member");
    fixture.write("grund.toml", SHOP);
    fixture.write("requirements.md", "# FS-receipt: Receipts\n");
    fixture.member("billing");
    for dir in [".", "packages/billing"] {
        fixture.init_in(dir);
    }
    assert!(agents_init_lines(&fixture.run(&["check"])).is_empty());

    fixture.member("notify");
    fixture.init_in("packages/notify");
    fixture.write(
        "grund.toml",
        &SHOP.replace(
            "members = [\"packages/billing\"]",
            "members = [\"packages/billing\", \"packages/notify\"]",
        ),
    );

    let check = fixture.run(&["check"]);
    let stale = agents_init_lines(&check);
    assert_eq!(
        check.status.code(),
        Some(1),
        "check: {}",
        text(&check.stdout)
    );
    for (path, dir) in [
        ("AGENTS.md", None),
        ("packages/billing/AGENTS.md", Some("packages/billing")),
        ("packages/notify/AGENTS.md", Some("packages/notify")),
    ] {
        let run = match dir {
            Some(dir) => format!("run `grund init` in {dir} to refresh"),
            None => "run `grund init` to refresh".to_owned(),
        };
        assert!(
            stale
                .iter()
                .any(|line| line.starts_with(&format!("{path}:3: "))
                    && line.contains("workspace members differ from the workspace")
                    && line.contains(&run)),
            "{path} not reported with `{run}`: {stale:?}"
        );
    }

    for dir in [".", "packages/billing", "packages/notify"] {
        fixture.init_in(dir);
    }
    for dir in [".", "packages/billing", "packages/notify"] {
        assert_current(&fixture, dir);
    }
    let check = fixture.run(&["check"]);
    assert_eq!(
        check.status.code(),
        Some(0),
        "check after init: {}",
        text(&check.stdout)
    );
    assert!(agents_init_lines(&check).is_empty());
}

/// §FS-check.3.5.4: the render reads the config and the workspace, never the scan
/// or a path argument, so `check`, `check --full` and `check <root>` agree.
#[test]
fn check_full_and_path_report_the_same_stale_blocks() {
    let fixture = Fixture::new("agree");
    fixture.write("grund.toml", SHOP);
    fixture.write("requirements.md", "# FS-receipt: Receipts\n");
    fixture.member("billing");
    for dir in [".", "packages/billing"] {
        fixture.init_in(dir);
    }
    fixture.write(
        "grund.toml",
        &format!("{SHOP}\n[schema.kinds.AR]\nfiles = [\"architecture.md\"]\n"),
    );
    fixture.write("architecture.md", "# AR-receipt: Receipts\n");

    let plain = agents_init_lines(&fixture.run(&["check"]));
    assert!(
        !plain.is_empty(),
        "check reports no stale block after a kind was added"
    );
    assert_eq!(plain, agents_init_lines(&fixture.run(&["check", "--full"])));
    assert_eq!(plain, agents_init_lines(&fixture.run(&["check", "."])));
}
