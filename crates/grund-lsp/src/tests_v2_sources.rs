//! The editor reads a v2 file as the scan does (§FS-config-v2.schema.sources): the
//! on-type shorthand expansion (§FS-lsp.1.4) asks the declaration gate the scan
//! asks (§FS-config-v2.schema.sources.declarations.gate), and reads a remapped
//! suffix as the language it maps to, whole (§FS-config-v2.schema.sources.extensions).

use super::*;

const V2: &str = "grund_config_version = 2\nproject_name = \"fixture\"\n[schema]\n\
                  id_format = \"{kind}-{number}-{slug}\"\n[schema.sources]\n\
                  languages = [\"markdown\", \"rust\", \"python\"]\n[schema.sources.extensions]\n\
                  mdx = \"markdown\"\nrustdoc = \"rust\"\npyi = \"python\"\n[schema.kinds.FS]\n";

const V1: &str = "grund_config_version = 1\n[id]\nformat = \"{kind}-{number}-{slug}\"\n\
                  [[kinds]]\nkind = \"FS\"\n";

/// The text of the edits typing the last character of line `line` of `text` earns,
/// with the file at `rel` under a fresh root configured by `config`, where
/// `docs/FS-001-login.md` declares the one ID a shorthand can name.
fn edits(name: &str, config: &str, rel: &str, text: &str, line: usize) -> Vec<String> {
    let root = test_root(name);
    write(&root.join("grund.toml"), config);
    let login = root.join("docs/FS-001-login.md");
    write(&login, "# FS-001-login: Login\n\nBody.\n");
    let path = root.join(rel);
    write(&path, text);
    let typed = text.lines().nth(line).expect("the edited line");
    let position = Position {
        line: line as u32,
        character: typed.encode_utf16().count() as u32,
    };
    let declared = [DeclaredId {
        path: &login,
        id: "FS-001-login",
    }];
    on_type_replacement_for_line(&path, text, position, &declared)
        .expect("formatting check")
        .expect("formatting response")
        .into_iter()
        .map(|edit| edit.new_text)
        .collect()
}

fn expanded() -> Vec<String> {
    vec!["FS-001-login".to_string()]
}

const GATED: &str = concat!(
    "/// FS-002-api: The API\npub fn f() {\n",
    "    // FS-003-note: a note on \u{a7}FS-001 \n}\n",
);

/// §FS-config-v2.schema.sources.declarations.gate: under v2's `doc-comments` a plain
/// `//` line declares nothing, so its shorthand expands, where its v1 twin is a
/// declaration line the editor never rewrites (§FS-config.5.2).
#[test]
fn a_gated_line_expands_under_v2_and_is_kept_under_v1() {
    assert_eq!(edits("v2_gate_v2", V2, "src/lib.rs", GATED, 2), expanded());
    let v1 = edits("v2_gate_v1", V1, "src/lib.rs", GATED, 2);
    assert!(v1.is_empty(), "v1 keeps the declaration line: {v1:?}");
}

/// §FS-config-v2.schema.sources.extensions: `.rustdoc` read as Rust keeps a raw
/// string's line, as `.rs` does, and expands the comment beside it.
#[test]
fn a_remapped_rust_suffix_keeps_its_raw_string() {
    let text = "/// See \u{a7}FS-001 \npub const S: &str = r#\"\n// \u{a7}FS-001 \n\"#;\n";
    for rel in ["src/a.rs", "src/b.rustdoc"] {
        let name = format!("v2_remap_{rel}");
        assert_eq!(edits(&name, V2, rel, text, 0), expanded(), "{rel} comment");
        let kept = edits(&name, V2, rel, text, 2);
        assert!(
            kept.is_empty(),
            "{rel} rewrote inside a raw string: {kept:?}"
        );
    }
}

/// §FS-config-v2.schema.sources.extensions: `.mdx` read as Markdown keeps its fenced
/// example, as `.md` does, and expands the prose above it.
#[test]
fn a_remapped_markdown_suffix_keeps_its_fence() {
    let text = "# FS-002-b: B\n\nSee \u{a7}FS-001 \n\n```text\nexample \u{a7}FS-001 \n```\n";
    for rel in ["docs/FS-002-b.md", "docs/FS-002-b.mdx"] {
        let name = format!("v2_remap_{rel}");
        assert_eq!(edits(&name, V2, rel, text, 2), expanded(), "{rel} prose");
        let kept = edits(&name, V2, rel, text, 5);
        assert!(kept.is_empty(), "{rel} rewrote inside a fence: {kept:?}");
    }
}

/// §FS-config-v2.schema.sources.extensions: `.pyi` read as Python expands a shorthand
/// in its docstring, as `.py` does.
#[test]
fn a_remapped_python_suffix_reads_its_docstring() {
    let text = "def f():\n    \"\"\"Implements \u{a7}FS-001 \n    \"\"\"\n";
    for rel in ["src/m.py", "src/m.pyi"] {
        let name = format!("v2_remap_{rel}");
        assert_eq!(
            edits(&name, V2, rel, text, 1),
            expanded(),
            "{rel} docstring"
        );
    }
}
