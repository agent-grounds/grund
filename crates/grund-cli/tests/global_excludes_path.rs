//! The readable global excludes file is the one Git resolves, with literal
//! interior tildes and one leading-home expansion (§FS-scan.scope.ignore-files.pathname).
//! Port of #788's control/interior-tilde reproduction; repository anchoring has
//! separate coverage in `global_excludes_anchor.rs`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[path = "support/global_excludes_compatibility.rs"]
mod compatibility;

const CONFIG: &str = "grund_config_version = 2\n\n\
    [schema]\nid_format = \"{kind}-{slug}\"\n\n\
    [schema.sources]\nrespect_gitignore = true\n\n\
    [schema.kinds.FS]\nfolders = [\"docs/fs\"]\nindex = false\n\n\
    [schema.kinds.code]\nfolders = [\"docs\", \"notes\"]\n";
const LISTED: &str = "FS-x  docs/fs/x.md:1  X\n";
const BOTH: &str = "FS-x  docs/fs/x.md:1  X\nFS-y  notes/gen/b.md:1  Y\n";

struct Fixture {
    dir: PathBuf,
    repo: PathBuf,
    home: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        Self::new_in(name, &std::env::temp_dir())
    }

    fn new_in(name: &str, base: &Path) -> Self {
        let dir = base.join(format!(
            "grund-788-global-path-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create fixture");
        // Normalize only the base, before adding any deliberate literal tilde.
        // Windows temp_dir may itself contain an unrelated short-name tilde.
        let dir = dir.canonicalize().expect("canonicalize fixture base");
        let fixture = Self {
            repo: dir.join("repo"),
            home: dir.join("home"),
            dir,
        };
        fs::create_dir_all(&fixture.home).expect("create isolated home");
        fs::create_dir_all(fixture.dir.join("xdg/git")).expect("create isolated XDG");
        fs::create_dir_all(&fixture.repo).expect("create repository");
        let init = fixture
            .command("git")
            .args(["init", "-q", "."])
            .output()
            .unwrap();
        assert!(init.status.success(), "git init: {init:?}");
        write(&fixture.repo.join("grund.toml"), CONFIG);
        write(&fixture.repo.join("docs/fs/x.md"), "# FS-x: X\n\nLead.\n");
        write(
            &fixture.repo.join("docs/fs/refs.md"),
            "Keeps \u{a7}FS-x live.\n",
        );
        write(
            &fixture.repo.join("notes/gen/b.md"),
            "# FS-y: Y\n\nCites \u{a7}FS-gone.\n",
        );
        fixture
    }

    fn command(&self, program: &str) -> Command {
        let mut command = Command::new(program);
        command
            .current_dir(&self.repo)
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("XDG_CONFIG_HOME", self.dir.join("xdg"))
            .env("GIT_CONFIG_GLOBAL", self.dir.join("xdg/git/config"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env_remove("GIT_CONFIG")
            .env_remove("GIT_CONFIG_SYSTEM")
            .env_remove("GIT_CONFIG_COUNT")
            .env_remove("GIT_CONFIG_PARAMETERS")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE");
        command
    }

    fn configure(&self, value: &str) {
        let output = self
            .command("git")
            .args(["config", "--global", "core.excludesFile", value])
            .output()
            .expect("configure global excludes");
        assert!(output.status.success(), "git config: {output:?}");
        let stored = fs::read_to_string(self.dir.join("xdg/git/config")).unwrap();
        assert!(
            stored.contains(value),
            "configured pathname changed: {stored}"
        );
    }

    fn excludes(&self, relative: &str) -> String {
        let file = self.dir.join(relative);
        write(&file, "notes/gen/\n");
        let value = git_path(&file);
        self.configure(&value);
        value
    }

    fn git_ignores(&self) {
        let output = self
            .command("git")
            .args(["check-ignore", "-v", "notes/gen/b.md"])
            .output()
            .expect("git check-ignore");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success(),
            "Git must validate fixture: {output:?}"
        );
        assert!(stdout.contains(":1:notes/gen/\t"), "{stdout}");
    }

    fn grund(&self, args: &[&str]) -> Output {
        self.command(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .output()
            .expect("run grund")
    }

    fn listed(&self) -> (String, Option<i32>) {
        let output = self.grund(&["list"]);
        (
            String::from_utf8(output.stdout).unwrap(),
            output.status.code(),
        )
    }

    fn assert_filtered(&self) {
        assert_eq!(self.listed(), (LISTED.into(), Some(0)));
        let checked = self.grund(&["check"]);
        assert_eq!(checked.status.code(), Some(0), "{checked:?}");
        assert_eq!(String::from_utf8_lossy(&checked.stdout), "success\n");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).expect("create fixture parent");
    fs::write(path, text).expect("write fixture file");
}

fn git_path(path: &Path) -> String {
    let path = path.to_string_lossy().replace('\\', "/");
    // Git's config value uses normal drive paths; keep the authored a~b intact.
    path.strip_prefix("//?/").unwrap_or(&path).to_string()
}

#[test]
fn an_interior_tilde_global_excludes_file_is_applied() {
    let fixture = Fixture::new("interior");
    for relative in ["ab/global.ignore", "a~b/global.ignore"] {
        let value = fixture.excludes(relative);
        fixture.git_ignores();
        assert_eq!(
            fixture.listed(),
            (LISTED.into(), Some(0)),
            "Git applies {relative}; grund must load the same file ({value})"
        );
        let checked = fixture.grund(&["check"]);
        assert_eq!(String::from_utf8_lossy(&checked.stdout), "success\n");
        assert_eq!(checked.status.code(), Some(0));
    }
}

/// This preservation guard passes before the repair: replacement must stay
/// single-pass even when the substituted home itself contains a literal tilde.
#[test]
fn a_leading_home_abbreviation_preserves_tildes_in_the_home_path() {
    let mut fixture = Fixture::new("leading-home");
    for home in ["home", "h~me"] {
        fixture.home = fixture.dir.join(home);
        write(&fixture.home.join("global.ignore"), "notes/gen/\n");
        fixture.configure("~/global.ignore");
        fixture.git_ignores();
        assert_eq!(fixture.listed(), (LISTED.into(), Some(0)), "home {home}");
    }
}

#[test]
fn an_interior_tilde_global_excludes_file_refuses_stub_targets() {
    let fixture = Fixture::new("stub");
    let value = fixture.excludes("a~b/global.ignore");
    fixture.git_ignores();
    write(
        &fixture.repo.join("docs/fs/x.md"),
        "# FS-x: [x](../../notes/gen/b.md)\n",
    );
    write(
        &fixture.repo.join("notes/gen/b.md"),
        "# FS-x: X\n\nTARGET BODY MUST NOT LEAK.\n",
    );
    let checked = fixture.grund(&["check", "--format", "json"]);
    let shown = fixture.grund(&["show", "FS-x"]);
    let findings: Vec<serde_json::Value> = String::from_utf8_lossy(&checked.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).expect("JSON finding"))
        .collect();
    let codes: Vec<_> = findings.iter().map(|f| f["code"].clone()).collect();
    assert_eq!(
        (codes, checked.status.code(), shown.status.code()),
        (vec![serde_json::json!("broken-stub")], Some(1), Some(1)),
        "Git ignores target through {value}; check: {}; show: {}{}",
        String::from_utf8_lossy(&checked.stdout),
        String::from_utf8_lossy(&shown.stdout),
        String::from_utf8_lossy(&shown.stderr)
    );
    assert_eq!(findings[0]["path"], "docs/fs/x.md");
    let plain = fixture.grund(&["check"]);
    let text = String::from_utf8_lossy(&plain.stdout).replace('\\', "/");
    assert!(text.contains("notes/gen/"), "{text}");
    assert!(text.contains("a~b/global.ignore"), "{text}");
    assert!(!String::from_utf8_lossy(&shown.stdout).contains("TARGET BODY"));
}

/// A concrete witness for §REQ-deterministic-output.2: with an absolute pathname,
/// HOME must not select a different file. All files exist before either run.
/// Unix only because the incorrectly substituted path embeds an absolute home
/// midway through the name, which is not a valid Windows drive pathname.
#[cfg(unix)]
#[test]
fn an_absolute_global_excludes_path_is_independent_of_home() {
    let mut fixture = Fixture::new("home-independent");
    let value = fixture.excludes("a~b/global.ignore");
    let first_home = fixture.home.clone();
    let second_home = fixture.dir.join("other-home");
    fs::create_dir_all(&second_home).unwrap();
    let wrongly_substituted = value.replace('~', &git_path(&first_home));
    write(Path::new(&wrongly_substituted), "notes/gen/\n");
    fixture.git_ignores();
    let first = fixture.listed();
    fixture.home = second_home;
    fixture.git_ignores();
    let second = fixture.listed();
    assert_eq!(
        (&first, &second),
        (&(LISTED.into(), Some(0)), &(LISTED.into(), Some(0))),
        "same absolute excludes file and Git ignore state, two HOME values"
    );
}

/// A preservation guard: opting out still reads the ignored declaration.
#[test]
fn a_literal_tilde_global_file_still_obeys_respect_gitignore_false() {
    let fixture = Fixture::new("opt-out");
    fixture.excludes("a~b/global.ignore");
    fixture.git_ignores();
    write(
        &fixture.repo.join("grund.toml"),
        &CONFIG.replace("respect_gitignore = true", "respect_gitignore = false"),
    );
    assert_eq!(fixture.listed(), (BOTH.into(), Some(0)));
}

/// R1-01: a second separator must not replace the expanded home with root.
#[test]
fn repeated_separators_after_home_keep_the_home_prefix() {
    let mut fixture = Fixture::new("home-separators");
    fixture.home = fixture.dir.join("h~me");
    write(&fixture.home.join("a~b/global.ignore"), "notes/gen/\n");
    write(&fixture.home.join("global.ignore"), "notes/gen/\n");
    for value in [
        "~//global.ignore",
        "~///global.ignore",
        "~//a~b/global.ignore",
    ] {
        fixture.configure(value);
        fixture.git_ignores();
        fixture.assert_filtered();
    }
}

/// The named-user branch has the same concatenation semantics. Its fixture
/// lives under the user's issue-specific scratch so losing HOME changes the file.
#[cfg(all(unix, not(target_os = "android")))]
#[test]
fn repeated_separators_after_a_named_home_keep_its_prefix() {
    let discovery = Fixture::new("named-home-user");
    let user = discovery.command("id").arg("-un").output().unwrap();
    assert!(user.status.success(), "{user:?}");
    let user = String::from_utf8(user.stdout).unwrap();
    discovery.configure(&format!("~{}/", user.trim()));
    let home = discovery
        .command("git")
        .args(["config", "--global", "--path", "--get", "core.excludesFile"])
        .output()
        .unwrap();
    assert!(home.status.success(), "{home:?}");
    let home = String::from_utf8(home.stdout).unwrap();
    let home = Path::new(home.trim()).canonicalize().unwrap();
    let fixture = Fixture::new_in("named-home-separators", &home.join("ag/tmp"));
    let file = fixture.dir.join("a~b/global.ignore");
    write(&file, "notes/gen/\n");
    fixture.configure(&format!(
        "~{}//{}",
        user.trim(),
        git_path(file.strip_prefix(home).unwrap())
    ));
    fixture.git_ignores();
    fixture.assert_filtered();
}

/// R1-02: the baseline selects the first setting, even where Git uses the last.
#[test]
fn repeated_global_excludes_settings_keep_first_value_precedence() {
    let fixture = Fixture::new("first-setting");
    fixture.excludes("real.ignore");
    let empty = fixture.dir.join("empty.ignore");
    write(&empty, "");
    let added = fixture
        .command("git")
        .args([
            "config",
            "--global",
            "--add",
            "core.excludesFile",
            &git_path(&empty),
        ])
        .output()
        .unwrap();
    assert!(added.status.success(), "{added:?}");
    fixture.assert_filtered();
    // The same selection applies to HOME's config, which precedes XDG's.
    fs::rename(
        fixture.dir.join("xdg/git/config"),
        fixture.home.join(".gitconfig"),
    )
    .unwrap();
    fixture.configure(&git_path(&empty));
    fixture.assert_filtered();
}

/// R1-02: an empty setting still permits the optional XDG default.
#[test]
fn an_empty_global_excludes_setting_keeps_default_discovery() {
    let fixture = Fixture::new("empty-setting");
    write(&fixture.dir.join("xdg/git/ignore"), "notes/gen/\n");
    fixture.configure("");
    fixture.assert_filtered();
    fs::rename(
        fixture.dir.join("xdg/git/config"),
        fixture.home.join(".gitconfig"),
    )
    .unwrap();
    fixture.assert_filtered();
    // An empty HOME setting also permits the next config before the default.
    fs::remove_file(fixture.dir.join("xdg/git/ignore")).unwrap();
    fixture.excludes("a~b/global.ignore");
    fixture.assert_filtered();
}

/// R1-03: unrelated malformed config previously allowed list/check to pass.
#[test]
fn unrelated_malformed_global_config_does_not_abort_scanning() {
    let fixture = Fixture::new("malformed-unrelated");
    write(
        &fixture.home.join(".gitconfig"),
        "[alias]\n broken = \"unterminated\n",
    );
    write(&fixture.repo.join("notes/gen/b.md"), "Ordinary text.\n");
    fixture.assert_filtered();
}

/// Optional config read/parse failures permit the next config or default;
/// neither is reported as successfully loading a configured excludes file.
#[test]
fn optional_global_config_failures_keep_fallback_discovery() {
    let fixture = Fixture::new("config-fallbacks");
    let malformed = "[alias]\n broken = \"unterminated\n";
    write(&fixture.home.join(".gitconfig"), malformed);
    fixture.excludes("a~b/global.ignore");
    fixture.assert_filtered();
    write(&fixture.dir.join("xdg/git/config"), malformed);
    write(&fixture.dir.join("xdg/git/ignore"), "notes/gen/\n");
    fixture.assert_filtered();
    // A directory in place of a config exercises an optional read failure on
    // Unix and Windows without depending on permissions or running as non-root.
    fs::remove_file(fixture.home.join(".gitconfig")).unwrap();
    fs::create_dir(fixture.home.join(".gitconfig")).unwrap();
    fixture.assert_filtered();
}
