//! §FS-schema.14: `grund schema` is taught where an agent learns grund, and each
//! place is held to what the command prints — the example README shows the table
//! and the Mermaid source the command prints for `examples/schema/repo`, the
//! example is linked from both indexes, and both copies of the `grund-init` skill
//! tell an agent to run it. The two skill copies' byte identity is §FS-init.5.3's,
//! pinned by `tests/integration/test_asset_sync.py`.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(relative: &str) -> String {
    fs::read_to_string(repo_root().join(relative))
        .unwrap_or_else(|err| panic!("read {relative}: {err}"))
}

/// The stdout of `grund schema examples/schema/repo <extra>`, run from the root.
fn schema(extra: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_grund"))
        .arg("schema")
        .arg("examples/schema/repo")
        .args(extra)
        .current_dir(repo_root())
        .output()
        .expect("spawn grund");
    assert!(
        out.status.success(),
        "grund schema examples/schema/repo {extra:?} exited {:?}: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf-8 schema view")
}

/// The body of the one fenced block opened by `fence` in `text`, with its last newline.
fn fenced(text: &str, fence: &str) -> String {
    let opener = format!("\n{fence}\n");
    assert_eq!(
        text.matches(&opener).count(),
        1,
        "expected exactly one {fence} block"
    );
    let start = text.find(&opener).unwrap() + opener.len();
    let end = start + text[start..].find("\n```\n").expect("closing fence") + 1;
    text[start..end].to_string()
}

#[test]
fn example_readme_shows_the_table_the_command_prints() {
    let printed = schema(&[]);
    assert_eq!(
        read("examples/schema/expected.stdout"),
        printed,
        "examples/schema/expected.stdout is not what `grund schema` prints"
    );
    assert_eq!(
        fenced(&read("examples/schema/README.md"), "```text"),
        printed,
        "the README's table is not what `grund schema` prints"
    );
}

#[test]
fn example_readme_shows_the_mermaid_source_the_command_prints() {
    assert_eq!(
        fenced(&read("examples/schema/README.md"), "```mermaid"),
        schema(&["--format", "mermaid"]),
        "the README's mermaid block is not what `grund schema --format mermaid` prints"
    );
}

#[test]
fn example_is_linked_from_the_examples_index_and_the_readme() {
    assert!(
        read("examples/README.md").contains("](schema/)"),
        "examples/README.md does not link schema/"
    );
    assert!(
        read("README.md").contains("](examples/schema/)"),
        "README.md does not link examples/schema/"
    );
}

#[test]
fn both_skill_copies_teach_grund_schema_beside_config_show() {
    for skill in [
        "skills/grund-init/SKILL.md",
        "crates/grund-core/assets/skills/grund-init/SKILL.md",
    ] {
        let text = read(skill);
        assert!(text.contains("grund schema"), "{skill} does not name `grund schema`");
        assert!(
            text.contains("kinds and places") && text.contains("citations"),
            "{skill} does not show an example of the schema table"
        );
        assert!(
            text.contains("grund config show"),
            "{skill} does not say `grund config show` is for the effective TOML"
        );
    }
}
