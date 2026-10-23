//! §FS-show.3.5.1: rejected sections retain the configured kind's format advice
//! across query adapters without changing resolution or refusal data.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use grund_core::{RefsOpts, query_format_hint, refs, refs_outcome, refs_with_metadata_and_hint};

const GLOBAL: &str = "{kind}-{slug}";
static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    target: PathBuf,
    version: u8,
    qualified: bool,
}

fn config(version: u8, template: &str, separator: &str) -> String {
    let header = format!("grund_config_version = {version}\nproject_name = \"child\"\n");
    if version == 2 {
        format!(
            "{header}[schema]\nid_format = \"{GLOBAL}\"\nsection_separator = \"{separator}\"\n\
             [schema.kinds.ADR]\nfolders = [\"docs\"]\nindex = false\nid_format = \"{template}\"\n"
        )
    } else {
        format!(
            "{header}[id]\nformat = \"{GLOBAL}\"\nsection_separator = \"{separator}\"\n\
             [scan]\ninclude = [\"docs\"]\n\
             [[kinds]]\nkind = \"ADR\"\nfolder = \"docs\"\nindex = false\nformat = \"{template}\"\n"
        )
    }
}

impl Fixture {
    fn new(version: u8, template: &str, separator: &str, qualified: bool) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-817-section-guidance-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let target = if qualified {
            root.join("member")
        } else {
            root.clone()
        };
        fs::create_dir_all(target.join("docs")).unwrap();
        fs::write(
            target.join("grund.toml"),
            config(version, template, separator),
        )
        .unwrap();
        if qualified {
            // A different version and template make accidentally using the parent visible.
            fs::create_dir_all(root.join("docs")).unwrap();
            let parent = config(3 - version, "parent{kind}_{number}", ".")
                .replace("project_name = \"child\"", "project_name = \"parent\"");
            fs::write(
                root.join("grund.toml"),
                format!("{parent}\n[workspace]\nmembers = [\"member\"]\ninclude_root = true\n"),
            )
            .unwrap();
        }
        let fixture = Self {
            root,
            target,
            version,
            qualified,
        };
        let loaded = fixture.run(&["config", "show"]);
        assert_eq!(loaded.status.code(), Some(0), "{}", stderr(&loaded));
        // Load the target independently too: a config refusal cannot satisfy a regression.
        let loaded = Command::new(env!("CARGO_BIN_EXE_grund"))
            .current_dir(&fixture.target)
            .args(["config", "show"])
            .output()
            .unwrap();
        assert_eq!(loaded.status.code(), Some(0), "{}", stderr(&loaded));
        assert!(String::from_utf8_lossy(&loaded.stdout).contains(template));
        fixture
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .current_dir(&self.root)
            .args(args)
            .output()
            .unwrap()
    }

    fn operand(&self, raw: &str) -> String {
        if self.qualified {
            format!("child/{raw}")
        } else {
            raw.into()
        }
    }

    fn opts(&self, operand: &str) -> RefsOpts {
        RefsOpts {
            path: self.root.clone(),
            path_provided: true,
            id: operand.into(),
            ..RefsOpts::default()
        }
    }

    fn hint(&self, template: &str, overridden: bool) -> String {
        let key = match (self.version, overridden) {
            (2, true) => "[schema.kinds.ADR] id_format",
            (2, false) => "[schema] id_format",
            (_, true) => "[[kinds]] format",
            (_, false) => "[id] format",
        };
        format!(
            "hint: this repo's {key} is `{template}` (run `grund config show`); `grund list` shows the IDs that exist"
        )
    }

    fn assert_refusal(&self, raw: &str, template: &str, overridden: bool, bare_suffix: &str) {
        let operand = self.operand(raw);
        let message = format!("invalid ID `{raw}`");
        let hint = self.hint(template, overridden);
        for prefix in [vec![], vec!["show"], vec!["refs"]] {
            let suffix = if prefix.is_empty() { bare_suffix } else { "" };
            let mut args = prefix;
            args.push(&operand);
            let output = self.run(&args);
            assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
            assert!(output.stdout.is_empty());
            assert_eq!(stderr(&output), format!("{message}\n{hint}\n{suffix}"));
            args.extend(["--format", "json"]);
            let output = self.run(&args);
            assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
            assert!(output.stdout.is_empty());
            let failure: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
            assert_eq!(failure["code"], "invalid-id");
            assert_eq!(failure["message"], message);
            assert_eq!(failure.as_object().unwrap().len(), 7);
        }
        assert_eq!(query_format_hint(&self.root, &operand).unwrap(), hint);
        let error = refs(self.opts(&operand)).unwrap_err();
        assert_eq!(format!("{error:#}"), format!("{message}\n{hint}"));
        let outcome = refs_outcome(self.opts(&operand)).unwrap();
        assert!(outcome.output.hits.is_empty());
        let failure = outcome.query_failure.unwrap();
        assert_eq!(failure.kind.code(), "invalid-id");
        assert_eq!(failure.message, message);
        // The published structured hint remains the global template, without a key.
        assert_eq!(failure.format_hint.as_deref(), Some(GLOBAL));
        let (metadata, prose) = refs_with_metadata_and_hint(self.opts(&operand)).unwrap();
        assert_eq!(metadata.outcome.query_failure.as_ref(), Some(&failure));
        assert_eq!(prose.as_deref(), Some(hint.as_str()));
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn stderr(output: &Output) -> &str {
    std::str::from_utf8(&output.stderr).unwrap()
}

#[test]
fn literal_and_nonleading_kind_sections_keep_effective_guidance() {
    for version in [1, 2] {
        for qualified in [false, true] {
            for (template, id) in [
                ("x{kind}_{number}", "xADR_1"),
                ("{kind}x{number}", "ADRx1"),
                ("{number}-{kind}", "1-ADR"),
            ] {
                let fixture = Fixture::new(version, template, ".", qualified);
                fs::write(
                    fixture.target.join("docs/a.md"),
                    format!("# {id}: Choice\n\nBody.\n"),
                )
                .unwrap();
                let operand = fixture.operand(id);
                for args in [
                    vec![operand.as_str()],
                    vec!["show", &operand],
                    vec!["refs", &operand],
                ] {
                    let output = fixture.run(&args);
                    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
                    let expected = if args.first() == Some(&"refs") {
                        // §FS-refs.2.1.1: an uncited declaration, no path, points at the listing.
                        let list = if fixture.qualified {
                            "grund list --project child --unused"
                        } else {
                            "grund list --unused"
                        };
                        format!(
                            "note: {operand} is declared but nothing cites it; \
                             `{list}` lists every such declaration\n"
                        )
                    } else {
                        String::new()
                    };
                    assert_eq!(stderr(&output), expected);
                    if args.first() != Some(&"refs") {
                        assert_eq!(String::from_utf8_lossy(&output.stdout), "Body.\n");
                    }
                }
                fixture.assert_refusal(&format!("{id}.bad!"), template, true, "");
            }
        }
    }
}

#[test]
fn configured_section_separator_selects_the_same_kind() {
    for version in [1, 2] {
        let fixture = Fixture::new(version, "x{kind}_{number}", "::", false);
        // §FS-cli.1.2: only the bare adapter adds this subcommand-help hint.
        fixture.assert_refusal(
            "xADR_1::bad!",
            "x{kind}_{number}",
            true,
            "hint: run `grund --help` for the list of subcommands\n",
        );
    }
}

#[test]
fn ambiguous_or_unidentified_kinds_keep_global_advice() {
    for version in [1, 2] {
        let fixture = Fixture::new(version, "x{kind}_{number}", ".", false);
        fixture.assert_refusal("nonsense.bad!", GLOBAL, false, "");
        let path = fixture.target.join("grund.toml");
        let mut config = fs::read_to_string(&path).unwrap();
        config.push_str(if version == 2 {
            "\n[schema.kinds.xADR]\nfolders = [\"other\"]\nindex = false\nid_format = \"{kind}_{number}\"\n\
             [schema.kinds.NOTE]\nfolders = [\"notes\"]\nindex = false\n"
        } else {
            "\n[[kinds]]\nkind = \"xADR\"\nfolder = \"other\"\nindex = false\nformat = \"{kind}_{number}\"\n\
             [[kinds]]\nkind = \"NOTE\"\nfolder = \"notes\"\nindex = false\n"
        });
        fs::write(path, config).unwrap();
        fs::create_dir_all(fixture.target.join("other")).unwrap();
        fs::create_dir_all(fixture.target.join("notes")).unwrap();
        let loaded = fixture.run(&["config", "show"]);
        assert_eq!(loaded.status.code(), Some(0), "{}", stderr(&loaded));
        fixture.assert_refusal("xADR_1.bad!", GLOBAL, false, "");
        fixture.assert_refusal("NOTE-example.bad!", GLOBAL, false, "");
    }
}
