//! Black-box: a declaration record whose location is the report base reads `.` (§FS-cli.3.4). An
//! E2E case is the record that gets there, because its location is its directory, and the
//! ambiguity refusal naming one carries `.` in its message and its `sites` (§FS-errors.5.2.1).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const CONFIG: &str = "grund_config_version = 1\n\n[reference]\nstrict = false\n\n\
    [id]\nformat = \"{kind}-{slug}\"\n\n\
    [[kinds]]\nkind = \"E2E\"\nfolder = \"e2e/cases\"\ntitle = \"End-to-end test\"\n";

const CASE: &str = "e2e/cases/login";

/// A project holding one E2E case, `E2E-login`, at `e2e/cases/login`.
struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("grund-1178-record-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(CASE)).expect("create the case");
        fs::write(root.join("grund.toml"), CONFIG).expect("write the config");
        fs::write(root.join(CASE).join("command.args"), "check repo\n").expect("write the args");
        fs::write(root.join(CASE).join("expected.exit"), "0\n").expect("write the exit");
        Self { root }
    }

    /// Stdout and stderr together: a refusal prints on the second, a record on the first.
    fn run(&self, cwd: &Path, args: &[&str]) -> String {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("run grund");
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    }

    /// `grund <args> e2e/cases/login --path-base=<base>`.
    fn narrowed(&self, args: &[&str], base: &str) -> String {
        let mut all = args.to_vec();
        all.extend([CASE, base]);
        self.run(&self.root, &all)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

const AT_BASE: &str = "--path-base=invocation";
const FROM_ROOT: &str = "--path-base=project";

#[test]
fn list_names_a_case_at_the_base_dot() {
    let fixture = Fixture::new("list");
    assert_eq!(fixture.narrowed(&["list"], AT_BASE), "E2E-login  .:1\n");
    assert_eq!(
        fixture.narrowed(&["list"], FROM_ROOT),
        "E2E-login  e2e/cases/login:1\n"
    );
    let json = |base| fixture.narrowed(&["list", "--format", "json"], base);
    assert!(
        json(AT_BASE).contains(r#""path":".","line":1"#),
        "{}",
        json(AT_BASE)
    );
    assert!(
        json(FROM_ROOT).contains(r#""path":"e2e/cases/login","line":1"#),
        "{}",
        json(FROM_ROOT)
    );
}

#[test]
fn list_sizes_names_a_case_at_the_base_dot() {
    let fixture = Fixture::new("sizes");
    let text = fixture.narrowed(&["list", "--size=words"], AT_BASE);
    assert!(text.starts_with("E2E-login  .:1  words="), "{text}");
    let json = fixture.narrowed(&["list", "--size=words", "--format", "json"], AT_BASE);
    assert!(json.contains(r#""path":".","line":1"#), "{json}");
}

/// `show`'s text form of a case names no path at all, so only its JSON can move with the base.
#[test]
fn show_names_a_case_at_the_base_dot() {
    let fixture = Fixture::new("show");
    let json = |args: &[&str], base| {
        let mut all = vec!["show", "--format", "json"];
        all.extend(args);
        fixture.narrowed(&all, base)
    };
    for args in [&["E2E-login"][..], &["--batch", "--glob", "E2E-*"]] {
        assert!(
            json(args, AT_BASE).contains(r#""kind":"E2E","path":".","args""#),
            "{}",
            json(args, AT_BASE)
        );
        assert!(
            json(args, FROM_ROOT).contains(r#""kind":"E2E","path":"e2e/cases/login","args""#),
            "{}",
            json(args, FROM_ROOT)
        );
    }
    let text = fixture.narrowed(&["show", "E2E-login"], AT_BASE);
    assert!(text.starts_with("grund check repo\n"), "{text}");
    assert_eq!(text, fixture.narrowed(&["show", "E2E-login"], FROM_ROOT));
}

#[test]
fn id_refusal_names_a_case_at_the_base_dot() {
    let fixture = Fixture::new("id");
    assert_eq!(
        fixture.narrowed(&["id", "E2E", "login"], AT_BASE),
        "proposed ID `E2E-login` already declared at .:1\n"
    );
}

/// One tree, three commands: `check`, `show` and `refs` publish the same `sites`, and the refusal's
/// message names the pairs they carry.
#[test]
fn every_command_names_an_ambiguous_site_at_the_base_dot() {
    let fixture = Fixture::new("ambiguous");
    fs::write(
        fixture.root.join("e2e/cases/dup.md"),
        "# E2E-login: A duplicate\n\nBody.\n",
    )
    .expect("write the duplicate");
    let case = fixture.root.join(CASE);
    const SITES: &str = r#""sites":[{"path":"../dup.md","line":1},{"path":".","line":1}]"#;
    let run = |args: &[&str]| fixture.run(&case, args);
    let check = run(&["check", AT_BASE, "--format", "json"]);
    assert!(check.contains(SITES), "{check}");
    for command in ["show", "refs"] {
        let json = run(&[command, "E2E-login", AT_BASE, "--format", "json"]);
        assert!(json.contains(SITES), "grund {command}: {json}");
        assert!(
            json.contains("ambiguous ID: E2E-login (declared at ../dup.md:1, .:1)"),
            "grund {command}: {json}"
        );
    }
    assert_eq!(
        run(&["show", "E2E-login", AT_BASE]),
        "ambiguous ID: E2E-login (declared at ../dup.md:1, .:1)\n"
    );
}

/// The fixture with a broken stub for `E2E-login` beside the case.
fn with_broken_stub(name: &str) -> Fixture {
    let fixture = Fixture::new(name);
    fs::write(
        fixture.root.join("e2e/cases/s.md"),
        "# E2E-login: [x](missing.md)\n",
    )
    .expect("write the stub");
    fixture
}

/// A broken stub adds a repair to the refusal, and the repair names the record as the sites do.
#[test]
fn a_broken_stub_refusal_names_its_one_home_at_the_base_dot() {
    let fixture = with_broken_stub("stub-one");
    let text = fixture.run(&fixture.root.join(CASE), &["show", "E2E-login", AT_BASE]);
    assert!(
        text.starts_with("ambiguous ID: E2E-login (declared at .:1, ../s.md:1); "),
        "{text}"
    );
    assert!(text.contains("; E2E-login is declared at .:1; "), "{text}");
    let control = fixture.run(&fixture.root, &["show", "E2E-login", FROM_ROOT]);
    assert!(
        control.contains("; E2E-login is declared at e2e/cases/login:1; "),
        "{control}"
    );
}

#[test]
fn a_broken_stub_refusal_names_several_homes_at_the_base_dot() {
    let fixture = with_broken_stub("stub-several");
    fs::write(
        fixture.root.join("e2e/cases/dup.md"),
        "# E2E-login: A duplicate\n\nBody.\n",
    )
    .expect("write the duplicate");
    let text = fixture.run(&fixture.root.join(CASE), &["show", "E2E-login", AT_BASE]);
    assert!(
        text.contains("; E2E-login has several real declaration homes: ../dup.md:1, .:1"),
        "{text}"
    );
    let control = fixture.run(&fixture.root, &["show", "E2E-login", FROM_ROOT]);
    assert!(
        control.contains(
            "; E2E-login has several real declaration homes: e2e/cases/dup.md:1, e2e/cases/login:1"
        ),
        "{control}"
    );
}
