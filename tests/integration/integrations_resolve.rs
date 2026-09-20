//! §FS-integrations.4 — the headless half of `scripts/try-integrations.sh`,
//! run as a gate: install the resolver into a sandbox HOME from the binary
//! under test and resolve every citation form the clients hand it against this
//! repository — plain, sectioned, bare, punctuation-swept, workspace-qualified,
//! unknown, and `path:line` locations (§FS-integrations.3.1). The script is a
//! manual testbed for the clickable clients; its resolver checks need no
//! terminal, so nothing excuses them from CI. Unix only: the script is `bash`
//! and the resolver it installs is the Unix shell integration.
#![cfg(unix)]

#[path = "binaries.rs"]
mod binaries;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A fresh `HOME` and `XDG_CONFIG_HOME` pair for one case. The environment is
/// per-child rather than per-process: `--write` resolves its targets out of the
/// variables it is handed, so a sandbox passed on the child's own environment
/// cannot race the tests running beside it.
fn install_sandbox(name: &str) -> (PathBuf, PathBuf) {
    let sandbox = binaries::repo_root()
        .join("target/integration-work")
        .join(name);
    let _ = fs::remove_dir_all(&sandbox);
    let home = sandbox.join("home");
    let xdg = sandbox.join("xdg");
    fs::create_dir_all(&home).expect("create sandbox HOME");
    fs::create_dir_all(&xdg).expect("create sandbox XDG_CONFIG_HOME");
    (home, xdg)
}

/// Run the binary under test with a sandboxed environment and return its
/// `(stdout, stderr)`: the snippet a run *prints* is the artifact, every
/// install line and warning is reported beside it on stderr.
fn integrations(home: &Path, xdg: Option<&Path>, args: &[&str]) -> (String, String) {
    let mut command = Command::new(binaries::grund());
    command.args(args).env("HOME", home);
    match xdg {
        Some(path) => command.env("XDG_CONFIG_HOME", path),
        None => command.env_remove("XDG_CONFIG_HOME"),
    };
    let output = command.output().expect("run grund integrations");
    assert!(
        output.status.success(),
        "grund {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// The reported half of a run: what stderr said about every target it touched.
fn installed(home: &Path, xdg: Option<&Path>, args: &[&str]) -> String {
    integrations(home, xdg, args).1
}

/// §FS-integrations.4.1.7: where a `~` target resolves. A `~/.config/…` target
/// follows `$XDG_CONFIG_HOME` when it is set and non-empty — kitty reads its
/// directory from there, so a hardcoded `~/.config` would install where the
/// tool never looks and still report success — and falls back to `~/.config`
/// when the variable is unset or empty. Every target outside `~/.config`,
/// `~/.local/bin/grund-open` here, resolves against `$HOME` alone. The printed
/// hint stays `~`-rooted whatever the environment says, so the artifact is
/// byte-stable across machines and only resolution consults the environment.
#[test]
fn a_config_target_follows_xdg_config_home_and_every_other_target_follows_home() {
    let (home, xdg) = install_sandbox("integrations-xdg-set");
    let reported = installed(&home, Some(&xdg), &["integrations", "kitty", "--write"]);
    assert!(
        reported.contains(&format!("{}/kitty/kitty.conf", xdg.display())),
        "a ~/.config target resolves under XDG_CONFIG_HOME:\n{reported}"
    );
    assert!(
        !reported.contains(&format!("{}/.config/kitty", home.display())),
        "a set XDG_CONFIG_HOME is where kitty looks, so ~/.config must not be written:\n{reported}"
    );
    assert!(
        reported.contains(&format!("{}/.local/bin/grund-open", home.display())),
        "a target outside ~/.config resolves against $HOME alone:\n{reported}"
    );

    let (printed, _) = integrations(&home, Some(&xdg), &["integrations", "kitty"]);
    assert!(
        printed.contains("~/.config/kitty/kitty.conf")
            && !printed.contains(&xdg.display().to_string()),
        "the printed hint stays ~-rooted; only resolution consults the environment:\n{printed}"
    );

    for (name, empty) in [
        ("integrations-xdg-unset", false),
        ("integrations-xdg-empty", true),
    ] {
        let (home, _) = install_sandbox(name);
        let xdg = empty.then(|| PathBuf::from(""));
        let reported = installed(&home, xdg.as_deref(), &["integrations", "kitty", "--write"]);
        assert!(
            reported.contains(&format!("{}/.config/kitty/kitty.conf", home.display())),
            "an unset or empty XDG_CONFIG_HOME falls back to ~/.config ({name}):\n{reported}"
        );
    }
}

/// §FS-integrations.4.3.7: the user config is read and reported exactly once
/// per invocation, **before** any client artifact is installed. Reported per
/// target it would repeat itself once per file; reported after the installs it
/// would arrive as an explanation of writes the reader has already scrolled
/// past.
#[test]
fn the_user_config_is_reported_once_and_before_any_install() {
    let (home, xdg) = install_sandbox("integrations-config-warning-order");
    fs::create_dir_all(xdg.join("grund")).expect("create user config home");
    fs::write(
        xdg.join("grund/config.toml"),
        "[reference]\nconversation = \"link\"\nmarker = \"\u{a7}\"\n",
    )
    .expect("write user config");

    let reported = installed(&home, Some(&xdg), &["integrations", "kitty", "--write"]);
    let warnings = reported
        .lines()
        .filter(|line| line.starts_with("warning:"))
        .count();
    assert_eq!(
        warnings, 1,
        "the one unused key is reported once, not once per target:\n{reported}"
    );
    let warning_at = reported
        .lines()
        .position(|line| line.starts_with("warning:"))
        .expect("the unused key is reported");
    let first_install = reported
        .lines()
        .position(|line| {
            ["appended ", "updated ", "exists ", "wrote ", "skipped "]
                .iter()
                .any(|verb| line.starts_with(verb))
        })
        .expect("the run installs something");
    assert!(
        warning_at < first_install,
        "the file is read before any install, so its warnings come first:\n{reported}"
    );
}

/// §FS-integrations.4.3.9: a global agent instruction file is written only
/// where the agent is in use on this machine — its own directory exists. An
/// agent the machine does not have is reported `skipped <path> (no <dir>)` and
/// nothing is created for it, because `--write` installs a rendering layer, not
/// the configuration trees of agents nobody runs. The skip is reported rather
/// than silent, so installing the agent afterwards and re-running is visibly
/// the remedy.
#[test]
fn only_an_agent_in_use_on_this_machine_receives_the_instruction_block() {
    let (home, xdg) = install_sandbox("integrations-agents-in-use");
    fs::create_dir_all(home.join(".claude")).expect("create the one agent in use");

    let reported = installed(&home, Some(&xdg), &["integrations", "kitty", "--write"]);
    let claude = home.join(".claude/CLAUDE.md");
    assert!(
        reported.contains(&claude.display().to_string())
            && !reported.contains(&format!("skipped {}", claude.display())),
        "the agent whose directory exists is written, not skipped:\n{reported}"
    );
    assert!(claude.is_file(), "the instruction file is on disk");
    assert!(
        reported.contains(&format!(
            "skipped {}/.pi/agent/AGENTS.md (no ~/.pi)",
            home.display()
        )),
        "an absent agent is reported with the directory that would have selected it:\n{reported}"
    );
    assert!(
        !home.join(".pi").exists(),
        "a stray ~/.pi is a file the user did not ask for and will not think to remove"
    );

    let again = installed(&home, Some(&xdg), &["integrations", "kitty", "--write"]);
    assert!(
        again.contains(&format!("skipped {}/.pi/agent/AGENTS.md", home.display())),
        "every target is reported either way, so the skip stays visible:\n{again}"
    );
}

/// §FS-integrations.3.1.6: the recorded false positive of the marker tolerance.
/// A printed spec *path* carries ID-shaped segments no portable matcher can
/// tell from a citation — `docs/functional-spec/FS-integrations.md` also matches
/// at `/functional-spec/FS-integrations`. Following that fragment costs one
/// explanatory `unknown id` and nothing else: it never opens the file whose
/// path it was cut out of, while the real citation on the same line resolves.
#[test]
fn a_swept_in_spec_path_fragment_is_one_explanatory_error_never_a_wrong_file() {
    let (home, xdg) = install_sandbox("integrations-path-false-positive");
    installed(&home, Some(&xdg), &["integrations", "kitty", "--write"]);
    let resolver = home.join(".local/bin/grund-open");
    assert!(
        resolver.is_file(),
        "the client install writes the resolver the terminal calls"
    );
    std::os::unix::fs::symlink(binaries::grund(), home.join(".local/bin/grund"))
        .expect("put the binary under test on the resolver's PATH");

    let repo = binaries::repo_root();
    let resolve = |token: &str| {
        let output = Command::new(&resolver)
            .arg(token)
            .current_dir(&repo)
            .env("HOME", &home)
            .env("XDG_CONFIG_HOME", &xdg)
            .env(
                "PATH",
                format!(
                    "{}/.local/bin:{}",
                    home.display(),
                    std::env::var("PATH").unwrap_or_default()
                ),
            )
            // The resolved location is echoed rather than opened, so a run that
            // wrongly resolved would still name the file it chose.
            .env("GRUND_OPEN_CMD", "/bin/echo")
            .output()
            .expect("run grund-open");
        let reported = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        (output.status.success(), reported)
    };

    let (resolved, reported) = resolve("/functional-spec/FS-integrations");
    assert!(
        !resolved,
        "the swept-in path fragment is not an ID: {reported}"
    );
    assert!(
        reported.contains("unknown id"),
        "the cost is one explanatory error: {reported}"
    );
    assert!(
        !reported.contains("FS-integrations.md"),
        "never a wrong file — the fragment must not open the path it was cut from: {reported}"
    );

    let (resolved, reported) = resolve("\u{a7}FS-integrations");
    assert!(
        resolved && reported.contains("docs/functional-spec/FS-integrations.md:"),
        "the real citation on the same line still matches and resolves: {reported}"
    );
}

/// §FS-show.3.1: installed resolvers trust this terminal pair for declarations
/// and sections because authored body prose may contain location-shaped JSON.
#[test]
fn titled_show_json_keeps_path_and_line_as_the_terminal_pair() {
    let repo = binaries::repo_root();
    let sandbox = repo.join("target/integration-work/show-json-location-tail");
    let _ = fs::remove_dir_all(&sandbox);
    fs::create_dir_all(sandbox.join("docs")).expect("create show JSON fixture");
    fs::write(
        sandbox.join("grund.toml"),
        concat!(
            "grund_config_version = 1\n",
            "[id]\nformat = \"{kind}-{slug}\"\n",
            "[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\n",
            "index = false\ntitle = \"Product contracts\"\n",
        ),
    )
    .expect("write show JSON config");
    fs::write(
        sandbox.join("docs/FS-authored.md"),
        "# FS-authored: Authored\n\nLead.\n\n## 1. Detail\n\nDetail body.\n",
    )
    .expect("write show JSON declaration");

    for (query, expected) in [
        (
            "FS-authored",
            "{\"id\":\"FS-authored\",\"section\":null,\"body\":\"Lead.\\n\",\"kind_title\":\"Product contracts\",\"path\":\"docs/FS-authored.md\",\"line\":1}\n",
        ),
        (
            "FS-authored.1",
            "{\"id\":\"FS-authored\",\"section\":\"1\",\"body\":\"## 1. Detail\\n\\nDetail body.\\n\",\"kind_title\":\"Product contracts\",\"path\":\"docs/FS-authored.md\",\"line\":5}\n",
        ),
    ] {
        let output = Command::new(binaries::grund())
            .args([query, "--format=json"])
            .current_dir(&sandbox)
            .output()
            .expect("run titled show JSON query");
        assert!(
            output.status.success(),
            "{query} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
        assert!(output.stderr.is_empty());
    }

    fs::remove_dir_all(&sandbox).expect("remove show JSON fixture");
}

/// §FS-integrations.4.1.5: the sandbox install is a client `--write`, and the
/// checks below run `$HOME/.local/bin/grund-open` directly — so the resolver
/// script has to have been written alongside the client's block, at the fixed
/// resolver path, and left executable.
#[cfg(unix)]
#[test]
fn the_resolver_resolves_every_citation_form_headlessly() {
    let repo = binaries::repo_root();
    let sandbox = repo.join("target/integration-work/integrations-sandbox");
    let _ = fs::remove_dir_all(&sandbox);
    let output = Command::new("bash")
        .arg(repo.join("scripts/try-integrations.sh"))
        .arg("resolve")
        .arg("--binary")
        .arg(binaries::grund())
        .arg("--sandbox")
        .arg(&sandbox)
        .current_dir(&repo)
        .output()
        .expect("run scripts/try-integrations.sh");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "try-integrations.sh resolve exited with {}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        output.status
    );
    let failed = stdout
        .lines()
        .filter(|line| line.trim_start().starts_with("FAIL"))
        .collect::<Vec<_>>();
    assert!(
        failed.is_empty(),
        "resolver checks failed:\n{}\n\nfull output:\n{stdout}",
        failed.join("\n")
    );
    let passed = stdout
        .lines()
        .filter(|line| line.trim_start().starts_with("PASS"))
        .count();
    assert!(
        passed >= 8,
        "expected the resolver checks to run; saw {passed} PASS line(s):\n{stdout}"
    );
}
