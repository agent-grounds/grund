//! The format a refused `refs --from` source prints in (§FS-refs.from) is `refs`' own
//! (§FS-refs.3): `--format`, else the configured `[output] format` (§FS-config.3.6) of
//! the run, the one its listing prints in. A scope inside a workspace member with no
//! `grund.toml` runs on the canonical defaults (§FS-scan.projects.discovery), so under a
//! root that configures `json` its listing prints `text`, and so must its refusal.
//!
//! The controls pin what holds today: the root's key at the root, the member's listing in
//! `text`, and `--format` over both. The member's refusal printed the root's `json` until
//! #1423, so that test is red until then. These are not goldens because a golden has no
//! skip. The tree is built under Cargo's scratch directory and only read.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

const ROOT_CONFIG: &str = "grund_config_version = 1\nproject_name = \"root\"\n\n\
                           [reference]\nstrict = true\n\n\
                           [id]\nformat = \"{kind}-{slug}\"\nnamed_sections = true\n\n\
                           [[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\nindex = false\n\n\
                           [scan]\ninclude = [\"docs\"]\n\n\
                           [output]\nformat = \"json\"\n\n\
                           [workspace]\nmembers = [\"api\"]\n";

const ROOT_DOC: &str = "# FS-root: Root\n\nRoot lead, see \u{a7}FS-root.\n";
const MEMBER_DOC: &str = "# FS-api: Api\n\nApi lead, see \u{a7}FS-api.\n";

/// The not-found refusal's hint (§FS-show.3.5).
const NOT_FOUND_HINT: &str = "hint: run `grund list` to see every declared ID, or \
                              `grund id <KIND> \"<title>\"` to propose a new one\n";

/// The invalid-ID hint naming the canonical defaults' template, the member's (§FS-show.3.5.1).
const MEMBER_FORMAT_HINT: &str = "hint: this repo's [id] format is `{kind}-{number}-{slug}` \
                                  (run `grund config show`); `grund list` shows the IDs that exist\n";

/// The member's one site, as its listing prints it in `text` (§FS-refs.3.1).
const MEMBER_LISTING: &str = "docs/FS-api.md:3: \u{a7}FS-api\n";

/// A workspace root configuring `[output] format = "json"`, declaring `FS-root`, and a
/// member `api` with no config of its own, declaring `FS-api`; removed on drop.
struct Workspace(PathBuf);

impl Workspace {
    fn new(name: &str) -> Self {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join("refs_from_refusal_format")
            .join(format!("{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for (path, text) in [
            ("grund.toml", ROOT_CONFIG),
            ("docs/FS-root.md", ROOT_DOC),
            ("api/docs/FS-api.md", MEMBER_DOC),
        ] {
            let path = root.join(path);
            fs::create_dir_all(path.parent().expect("a fixture file has a parent"))
                .expect("create the fixture's directories");
            fs::write(&path, text).expect("write a fixture file");
        }
        Self(root)
    }

    /// `grund <args>` run in `dir` below the root, as `(exit code, stdout, stderr)`.
    fn run(&self, dir: &str, args: &[&str]) -> (i32, String, String) {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(self.0.join(dir))
            .env("NO_COLOR", "1")
            .output()
            .expect("run grund");
        (
            output.status.code().expect("exited with a status"),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        )
    }

    /// Each `(dir, args, exit code, stdout, stderr)` printed exactly that, every stream
    /// compared whole; the failures are listed together, so one run cannot hide another.
    fn assert_runs(&self, runs: &[(&str, &[&str], i32, &str, &str)]) {
        let failures: Vec<String> = runs
            .iter()
            .filter_map(|&(dir, args, code, stdout, stderr)| {
                let actual = self.run(dir, args);
                let expected = (code, stdout.to_owned(), stderr.to_owned());
                (actual != expected).then(|| {
                    format!(
                        "(cd {dir}) grund {}\n  expected: {expected:?}\n  actual:   {actual:?}",
                        args.join(" ")
                    )
                })
            })
            .collect();
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A JSON refusal on stderr in the bare query-failure shape (§FS-errors.5.2).
fn json_refusal(code: &str, message: &str) -> String {
    format!(
        "{{\"severity\":\"error\",\"path\":null,\"line\":null,\"code\":\"{code}\",\
         \"message\":\"{message}\",\"sites\":null,\"authority\":null}}\n"
    )
}

/// What holds today and must keep holding: at the root the configured `json` answers the
/// listing and the refusal alike, the configless member lists in `text` from the root and
/// from inside it, and `--format` wins over either (§FS-refs.3, §FS-config.3.6).
#[test]
fn refs_from_refusal_format_controls() {
    let workspace = Workspace::new("controls");
    let root_listing = "{\"project\":\"root\",\"path\":\"docs/FS-root.md\",\"line\":3,\
                        \"column\":16,\"id\":\"FS-root\",\"section\":null,\"marker\":true,\
                        \"text\":\"\u{a7}FS-root\",\"enclosing_declaration\":\"FS-root\",\
                        \"enclosing_section\":null}\n";
    let root_refusal = json_refusal("not-found", "ID not found: FS-nope");
    let member_json = json_refusal("not-found", "ID not found: FS-001-nope");
    let member_text = format!("ID not found: FS-001-nope\n{NOT_FOUND_HINT}");
    let member_listed: &[&str] = &["refs", "--from", "FS-api", "api"];
    let flag_json: &[&str] = &["refs", "--from", "FS-001-nope", "api", "--format", "json"];
    let flag_text: &[&str] = &["refs", "--from", "FS-001-nope", "api", "--format", "text"];
    workspace.assert_runs(&[
        (".", &["refs", "--from", "FS-root"], 0, root_listing, ""),
        (".", &["refs", "--from", "FS-nope"], 1, "", &root_refusal),
        (".", member_listed, 0, MEMBER_LISTING, ""),
        ("api", &["refs", "--from", "FS-api"], 0, MEMBER_LISTING, ""),
        (".", flag_json, 1, "", &member_json),
        (".", flag_text, 1, "", &member_text),
    ]);
}

/// In the configless member a refused source prints in `text`, the format the same run
/// lists in, whether the member is named by the path or is the working directory, and
/// whether the source is not found or invalid (§FS-refs.from, §FS-refs.3,
/// §FS-scan.projects.discovery). Until #1423 each printed the root's `json`.
#[test]
fn refs_from_refusal_in_a_configless_member_prints_in_the_format_the_run_lists_in() {
    let workspace = Workspace::new("configless-member");
    let not_found = format!("ID not found: FS-001-nope\n{NOT_FOUND_HINT}");
    let invalid = format!("invalid ID `FS-`\n{MEMBER_FORMAT_HINT}");
    let member_named: &[&str] = &["refs", "--from", "FS-001-nope", "api"];
    workspace.assert_runs(&[
        (".", member_named, 1, "", &not_found),
        (".", &["refs", "--from", "FS-", "api"], 1, "", &invalid),
        ("api", &["refs", "--from", "FS-001-nope"], 1, "", &not_found),
    ]);
}
