//! §FS-cli.3.4 / §FS-cli.3.5: `--path-base=project|invocation` is a run flag that
//! picks one report base for the whole run, outranks `[output] relative_paths`, and
//! whose bad value — like a bad `--format` — is answered before anything is loaded.
//!
//! The workspace is a root project with a member `member/`: a duplicate declaration
//! gives a multi-site finding, and each project has one dangling citation. Every case
//! runs from its own working directory, which the golden e2e harness cannot set.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ROOT_CONFIG: &str = "grund_config_version = 1\nproject_name = \"root\"\n\n\
     [workspace]\nmembers = [\"member\"]\n\n\
     [id]\nformat = \"{kind}-{slug}\"\n\n\
     [scan]\nrespect_gitignore = false\n\n\
     [[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\nindex = false\n";

const MEMBER_CONFIG: &str = "grund_config_version = 1\nproject_name = \"member\"\n\n\
     [id]\nformat = \"{kind}-{slug}\"\n\n\
     [scan]\nrespect_gitignore = false\n\n\
     [[kinds]]\nkind = \"FS\"\nfolder = \"docs/fs\"\nindex = false\n";

const PATH_BASE_ERROR: &str = "error: unsupported path base `x` (expected project or invocation)\n";

/// Builds the workspace under `target/path-base-flag/<name>`; `output` is appended to
/// both configs, so `""` leaves every `[output]` key at its default.
fn fixture(name: &str, output: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/path-base-flag")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    for dir in ["docs/fs", "src/deep", "member/docs/fs", "member/src"] {
        fs::create_dir_all(root.join(dir)).expect("create fixture dir");
    }
    let write = |path: &str, text: &str| fs::write(root.join(path), text).expect("write fixture");
    write("grund.toml", &format!("{ROOT_CONFIG}{output}"));
    write("member/grund.toml", &format!("{MEMBER_CONFIG}{output}"));
    write("docs/fs/FS-a.md", "# FS-a: A\n\nBody.\n");
    write("docs/fs/FS-dup.md", "# FS-a: A again\n\nBody.\n");
    write(
        "src/deep/lib.rs",
        "// cites \u{a7}FS-a and the dangling \u{a7}FS-missing\nfn f() {}\n",
    );
    write("member/docs/fs/FS-m.md", "# FS-m: M\n\nBody.\n");
    write(
        "member/src/m.rs",
        "// \u{a7}FS-m and dangling \u{a7}FS-nope\nfn g() {}\n",
    );
    root.canonicalize().expect("canonical fixture root")
}

fn grund(args: &[&str], cwd: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_grund"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("spawn grund")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// Exit code, stdout and stderr, so one `assert_eq!` compares a whole run.
fn run(args: &[&str], cwd: &Path) -> (Option<i32>, String, String) {
    let out = grund(args, cwd);
    (out.status.code(), text(&out.stdout), text(&out.stderr))
}

/// The cases of the acceptance list: (working directory relative to the root, or `/`
/// for the filesystem root; arguments). `{abs}` is the fixture's absolute file path.
const CASES: &[(&str, &[&str])] = &[
    ("", &["check"]),
    ("src/deep", &["check"]),
    ("src/deep", &["check", "../.."]),
    ("member", &["check"]),
    ("member/src", &["check"]),
    ("", &["check", "src/deep/lib.rs"]),
    ("/", &["check", "{abs}"]),
    ("src/deep", &["list"]),
    ("src/deep", &["refs", "member/FS-m"]),
    ("src/deep", &["refs", "FS-a"]),
    ("src/deep", &["cover"]),
    ("src/deep", &["show", "member/FS-m"]),
    ("src/deep", &["member/FS-m"]),
];

/// The case's arguments, `{abs}` filled in, run flags inserted after the subcommand
/// (§FS-cli.4.1).
fn case_args(root: &Path, args: &[&str], flags: &[&str]) -> Vec<String> {
    let abs = root.join("src/deep/lib.rs").to_string_lossy().into_owned();
    let mut all: Vec<String> = args.iter().map(|a| a.replace("{abs}", &abs)).collect();
    all.splice(1..1, flags.iter().map(|f| f.to_string()));
    all
}

fn cwd_of(root: &Path, rel: &str) -> PathBuf {
    if rel == "/" {
        root.ancestors()
            .last()
            .expect("filesystem root")
            .to_path_buf()
    } else {
        root.join(rel)
    }
}

fn run_case(root: &Path, rel: &str, args: &[String]) -> (Option<i32>, String, String) {
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run(&refs, &cwd_of(root, rel))
}

/// §FS-cli.3.4: under `--path-base=invocation` a tree with no `[output]` table prints
/// exactly what the same tree prints today under `relative_paths = false` — text and
/// JSON, every case of the acceptance list, multi-site `sites` included.
#[test]
fn invocation_flag_prints_the_bytes_of_relative_paths_false() {
    let flagged = fixture("invocation-flag", "");
    let keyed = fixture("invocation-key", "\n[output]\nrelative_paths = false\n");
    for (rel, args) in CASES {
        for format in [&[][..], &["--format", "json"][..]] {
            let keyed_args = case_args(&keyed, args, format);
            let flags = [&["--path-base=invocation"][..], format].concat();
            let flagged_args = case_args(&flagged, args, &flags);
            let expected = run_case(&keyed, rel, &keyed_args);
            let actual = run_case(&flagged, rel, &flagged_args);
            // The two fixtures differ only in their directory name, which no
            // report path may carry (§FS-errors.4).
            assert_eq!(
                actual, expected,
                "(cd {rel}) grund {flagged_args:?} must equal relative_paths = false"
            );
        }
    }
}

/// §FS-cli.3.4: the proposal's worked example, pinned byte for byte so the equivalence
/// above cannot pass by both sides drifting together.
#[test]
fn invocation_flag_from_a_subdirectory_spells_paths_from_there() {
    let root = fixture("invocation-literal", "");
    let deep = root.join("src/deep");

    assert_eq!(
        run(&["check", "--path-base=invocation"], &deep),
        (
            Some(1),
            concat!(
                "../../docs/fs/FS-a.md:1: error: duplicate declaration of FS-a ",
                "(also declared at ../../docs/fs/FS-dup.md:1)\n",
                "../../member/src/m.rs:1: error: unknown reference FS-nope\n",
                "lib.rs:1: error: unknown reference FS-missing\n",
            )
            .to_string(),
            String::new(),
        )
    );
    assert_eq!(
        run(
            &["check", "--path-base", "invocation", "--format", "json"],
            &deep
        ),
        (
            Some(1),
            concat!(
                "{\"severity\":\"error\",\"path\":\"../../docs/fs/FS-a.md\",\"line\":1,",
                "\"code\":\"duplicate\",\"message\":\"duplicate declaration of FS-a ",
                "(also declared at ../../docs/fs/FS-dup.md:1)\",\"sites\":[",
                "{\"path\":\"../../docs/fs/FS-a.md\",\"line\":1},",
                "{\"path\":\"../../docs/fs/FS-dup.md\",\"line\":1}],\"authority\":null}\n",
                "{\"severity\":\"error\",\"path\":\"../../member/src/m.rs\",\"line\":1,",
                "\"code\":\"dangling\",\"message\":\"unknown reference FS-nope\",",
                "\"sites\":null,\"authority\":null}\n",
                "{\"severity\":\"error\",\"path\":\"lib.rs\",\"line\":1,",
                "\"code\":\"dangling\",\"message\":\"unknown reference FS-missing\",",
                "\"sites\":null,\"authority\":null}\n",
            )
            .to_string(),
            String::new(),
        )
    );
    assert_eq!(
        run(
            &["check", "--path-base=invocation"],
            &root.join("member/src")
        ),
        (
            Some(1),
            "m.rs:1: error: unknown reference FS-nope\n".to_string(),
            String::new(),
        )
    );
    let filesystem_root = cwd_of(&root, "/");
    let abs = root.join("src/deep/lib.rs");
    assert_eq!(
        run(
            &["check", "--path-base=invocation", &abs.to_string_lossy()],
            &filesystem_root
        ),
        (
            Some(1),
            "lib.rs:1: error: unknown reference FS-missing\n".to_string(),
            String::new(),
        )
    );
}

/// §FS-cli.3.4: with no flag nothing moves, and `--path-base=project` overrides a
/// committed `relative_paths = false` for the whole run, members included.
#[test]
fn project_flag_overrides_committed_relative_paths_false() {
    let default = fixture("project-default", "");
    let keyed = fixture("project-key", "\n[output]\nrelative_paths = false\n");
    for (rel, args) in CASES {
        let plain = case_args(&default, args, &[]);
        let flagged = case_args(&keyed, args, &["--path-base=project"]);
        assert_eq!(
            run_case(&keyed, rel, &flagged),
            run_case(&default, rel, &plain),
            "(cd {rel}) grund {flagged:?} must equal the default project base"
        );
    }
}

/// §FS-cli.3.4: the config file a load error names follows the flag, the one path a
/// config key cannot move.
#[test]
fn broken_member_config_is_named_from_the_invocation() {
    let root = fixture("broken-member", "");
    fs::write(
        root.join("member/grund.toml"),
        "grund_config_version = 1\n[output]\nformat = \"yaml\"\n",
    )
    .expect("break member config");
    let src = root.join("src/deep");

    assert_eq!(
        run(&["check"], &src),
        (
            Some(2),
            String::new(),
            "error: member/grund.toml:3: unsupported output format\n".to_string(),
        )
    );
    assert_eq!(
        run(&["check", "--path-base=invocation"], &src),
        (
            Some(2),
            String::new(),
            "error: ../../member/grund.toml:3: unsupported output format\n".to_string(),
        )
    );
}

/// §FS-cli.3.5: a bad value of either run flag is a usage error answered before the
/// load, on every command that takes the flag — even against a broken `grund.toml`.
#[test]
fn bad_run_flag_is_answered_before_a_broken_config() {
    let root = fixture("bad-flag", "");
    fs::write(
        root.join("grund.toml"),
        "grund_config_version = 1\n[output]\nformat = \"yaml\"\n",
    )
    .expect("break root config");

    for (args, message) in [
        (
            &["list", "--format", "bogus"][..],
            "error: unsupported list format `bogus`\n",
        ),
        (
            &["refs", "FS-a", "--format", "bogus"][..],
            "error: unsupported refs format `bogus`\n",
        ),
    ] {
        assert_eq!(
            run(args, &root),
            (Some(2), String::new(), message.to_string()),
            "grund {args:?}"
        );
    }

    let commands: &[&[&str]] = &[
        &["check"],
        &["list"],
        &["refs", "FS-a"],
        &["cover"],
        &["show", "FS-a"],
        &["FS-a"],
        &["id", "FS", "x"],
        &["integrations"],
        &["fmt"],
        &["config", "validate"],
        &["config", "show"],
    ];
    for command in commands {
        for flag in [&["--path-base=x"][..], &["--path-base", "x"][..]] {
            let mut args: Vec<&str> = command.to_vec();
            if args[0] == "config" {
                args.splice(2..2, flag.iter().copied());
            } else {
                args.splice(1..1, flag.iter().copied());
            }
            assert_eq!(
                run(&args, &root),
                (Some(2), String::new(), PATH_BASE_ERROR.to_string()),
                "grund {args:?}"
            );
        }
    }
}

/// §FS-cli.3: the commands that render no report path keep rejecting the flag, and
/// §FS-cli.4.1: a leading `--path-base` is a placement error naming the fixed command.
#[test]
fn path_base_placement_and_rejection() {
    let root = fixture("placement", "");

    assert_eq!(
        run(&["--path-base", "invocation", "list"], &root),
        (
            Some(2),
            String::new(),
            "error: --path-base follows the subcommand: grund list --path-base invocation\n"
                .to_string(),
        )
    );
    assert_eq!(
        run(&["--path-base=invocation", "check", "src"], &root),
        (
            Some(2),
            String::new(),
            "error: --path-base follows the subcommand: grund check --path-base=invocation src\n"
                .to_string(),
        )
    );
    // A leading run flag before an ID is still the default `show` query (§FS-cli.1).
    assert_eq!(
        run(&["--path-base=invocation", "member/FS-m"], &root),
        (Some(0), "Body.\n".to_string(), String::new())
    );

    for args in [
        &["init", "--path-base=invocation"][..],
        &["fetch", "--path-base=invocation", "FS-a"][..],
        &["completions", "bash", "--path-base=invocation"][..],
    ] {
        assert_eq!(
            run(args, &root),
            (
                Some(2),
                String::new(),
                "error: unknown flag `--path-base=invocation`\n".to_string(),
            ),
            "grund {args:?}"
        );
    }
    assert_eq!(
        run(
            &["agent-setup-instructions", "--path-base=invocation"],
            &root
        ),
        (
            Some(2),
            String::new(),
            "error: agent-setup-instructions takes no arguments\n".to_string(),
        )
    );
}

/// §FS-cli.3.4: every command that takes the flag documents it, defines
/// `invocation`, and names it as the replacement for `relative_paths = false`; the
/// commands that honour `[output] format` name `--format json` as its replacement.
#[test]
fn help_pages_show_the_run_flags() {
    let root = fixture("help", "");
    let honours_format: &[&[&str]] = &[&["check"], &["list"], &["refs"], &["cover"]];
    let path_only: &[&[&str]] = &[&["show"], &["id"], &["integrations"], &["fmt"], &["config"]];
    for command in honours_format.iter().chain(path_only) {
        let mut args = command.to_vec();
        args.push("--help");
        let (code, stdout, stderr) = run(&args, &root);
        assert_eq!((code, stderr.as_str()), (Some(0), ""), "grund {args:?}");
        for needle in [
            "--path-base project|invocation",
            "relative to the path you passed, or the current directory if you passed none",
            "relative_paths = false",
            "--path-base=invocation",
        ] {
            assert!(
                stdout.contains(needle),
                "grund {args:?} lacks `{needle}`:\n{stdout}"
            );
        }
    }
    for command in honours_format {
        let mut args = command.to_vec();
        args.push("--help");
        let (_, stdout, _) = run(&args, &root);
        assert!(
            stdout.contains("[output] format"),
            "grund {args:?} does not name the key `--format json` replaces:\n{stdout}"
        );
    }
}
