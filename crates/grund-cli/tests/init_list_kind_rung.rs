//! Black-box: the managed block's discovery rung names the project's own kinds in its `--kind`
//! example, so the recipe runs in the project it was rendered for (§FS-init.2.3.4.3). Ports the
//! #972 reproducer: a REQ-only project, on its first init and after migrating to version 2.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The reproducer's version-1 config: one kind, `REQ`, homed in `docs/req`.
const REQ_ONLY: &str = "[id]
format = \"{kind}-{slug}\"
named_sections = true

[[kinds]]
kind = \"REQ\"
folder = \"docs/req\"

[scan]
include = [\"docs\", \"README.md\"]
";

/// The reproducer's tree beside `REQ_ONLY`.
const REQ_TREE: &[(&str, &str)] = &[
    (
        "docs/req/REQ-login.md",
        "# REQ-login: Users log in\n\nA user logs in with a name and a password.\n",
    ),
    (
        "docs/req/README.md",
        "# Requirements\n\n- [\u{a7}REQ-login](REQ-login.md): users log in\n",
    ),
    (
        "README.md",
        "# Demo\n\nLogin works as \u{a7}REQ-login says.\n",
    ),
];

/// A config that leaves the kinds out, so the built-in default kinds apply.
const DEFAULT_KINDS: &str = "[id]\nformat = \"{kind}-{slug}\"\n";

/// A config that declares no kind at all, so none is citable.
const NO_KINDS: &str = "kinds = []\n\n[id]\nformat = \"{kind}-{slug}\"\n";

const DISCOVERY_TAIL: &str = " — discover IDs if you get lost";

struct Fixture(PathBuf);

impl Fixture {
    /// A Git tree at a fresh temp path holding `config`, when given, and `files`.
    fn new(name: &str, config: Option<&str>, files: &[(&str, &str)]) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("grund-972-{name}-{}-{serial}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create fixture");
        let git = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&dir)
            .output()
            .expect("initialize fixture Git repository");
        assert!(git.status.success(), "git init: {}", text(&git.stderr));
        if let Some(config) = config {
            fs::write(dir.join("grund.toml"), config).unwrap();
        }
        for (path, contents) in files {
            let path = dir.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, contents).unwrap();
        }
        Self(dir)
    }

    fn grund(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .expect("run grund")
    }

    /// Run `args` and require exit 0.
    fn ok(&self, args: &[&str]) {
        let out = self.grund(args);
        assert!(
            out.status.success(),
            "grund {} exited {:?}: stdout:\n{}\nstderr:\n{}",
            args.join(" "),
            out.status.code(),
            text(&out.stdout),
            text(&out.stderr)
        );
    }

    /// Run `grund init` and return the discovery bullet of the AGENTS.md it wrote.
    fn init_discovery_bullet(&self) -> String {
        self.ok(&["init"]);
        let agents = fs::read_to_string(self.0.join("AGENTS.md")).expect("read AGENTS.md");
        let bullets: Vec<&str> = agents
            .lines()
            .filter(|line| line.starts_with("- `grund list`") && line.ends_with(DISCOVERY_TAIL))
            .collect();
        assert_eq!(bullets.len(), 1, "one discovery bullet in:\n{agents}");
        bullets[0].to_string()
    }

    /// Every `grund list --kind …` the bullet teaches exits 0 in this project.
    fn run_kind_recipes(&self, bullet: &str) {
        for recipe in bullet
            .split('`')
            .filter(|s| s.starts_with("grund list --kind "))
        {
            let args: Vec<&str> = recipe.split_whitespace().skip(1).collect();
            self.ok(&args);
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn kind_bullet(kinds: &str) -> String {
    format!("- `grund list` / `grund list --kind {kinds}`{DISCOVERY_TAIL}")
}

/// The #972 reproducer: the REQ-only block teaches `--kind REQ`, which runs, on the version-1
/// first init and again after `config migrate --to 2 --write` and a refresh init.
#[test]
#[ignore = "red until #972 the block renders --kind from the project's kinds; implement removes this"]
fn req_only_block_teaches_a_kind_recipe_that_runs() {
    let project = Fixture::new("req-only", Some(REQ_ONLY), REQ_TREE);
    for run in ["v1 first init", "v2 refresh init"] {
        if run == "v2 refresh init" {
            project.ok(&["config", "migrate", "--to", "2", "--write"]);
        }
        let bullet = project.init_discovery_bullet();
        assert_eq!(bullet, kind_bullet("REQ"), "{run}");
        project.run_kind_recipes(&bullet);
    }
}

/// A config that leaves the kinds out keeps the default example, `FS,AR`, byte for byte.
#[test]
fn default_kinds_block_keeps_fs_ar() {
    let project = Fixture::new("default-kinds", Some(DEFAULT_KINDS), &[]);
    let bullet = project.init_discovery_bullet();
    assert_eq!(bullet, kind_bullet("FS,AR"));
    project.run_kind_recipes(&bullet);
}

/// A bare `grund init` writes the software-development config (FS, REQ, AR in folders), whose
/// first other folder kind after FS is REQ.
#[test]
#[ignore = "red until #972 the block renders --kind from the project's kinds; implement removes this"]
fn software_development_block_teaches_fs_req() {
    let project = Fixture::new("software-development", None, &[]);
    let bullet = project.init_discovery_bullet();
    assert_eq!(bullet, kind_bullet("FS,REQ"));
    project.run_kind_recipes(&bullet);
}

/// With no citable kind the bullet has no `--kind` half to refuse.
#[test]
#[ignore = "red until #972 the bullet drops --kind when no kind is citable; implement removes this"]
fn no_kind_block_teaches_no_kind_recipe() {
    let project = Fixture::new("no-kinds", Some(NO_KINDS), &[]);
    let bullet = project.init_discovery_bullet();
    assert_eq!(bullet, format!("- `grund list`{DISCOVERY_TAIL}"));
}
