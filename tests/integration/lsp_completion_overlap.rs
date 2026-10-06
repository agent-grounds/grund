//! Real protocol regressions for grammar-owned introducers (§FS-lsp.1.6.1,
//! §FS-lsp.1.6.3). Accepted edits resolve and preserve UTF-16 neighbors.

use super::completion_support::*;
use serde_json::json;

#[test]
fn overlapping_introducers_complete_longer_prefixes_and_preserve_adjacent_citations() {
    for (separator, marker, trigger) in [("-", "\u{a7}", "-"), ("_", "_", "$$")] {
        let f = Fixture::new();
        f.write("grund.toml", &format!("grund_config_version = 1\n[id]\nformat = \"{{kind}}{separator}{{slug}}\"\n[reference]\nmarker = \"{marker}\"\ntrigger = \"{trigger}\"\n[fmt.cross_refs]\nenabled = false\n"));
        for slug in ["login", "logout"] {
            f.write(
                &format!("docs/functional-spec/FS-{slug}.md"),
                &format!("# FS{separator}{slug}: {slug}\n\nLead.\n"),
            );
        }
        let mut s = Session::new(&f.0);
        let path = f.0.join("src/lib.rs");
        let id = format!("FS{separator}login");
        let other = format!("FS{separator}logout");
        for introducer in [marker, trigger] {
            for prefix in [
                "".to_string(),
                "F".into(),
                "FS".into(),
                format!("FS{separator}"),
                format!("FS{separator}lo"),
                id.clone(),
            ] {
                let before = "//! 😀 ";
                let text = format!("{before}{introducer}{prefix}");
                let items = s.completion(&path, &text, 0, text.len());
                let item = chosen(&items, &id, marker);
                assert_eq!(item["filterText"], format!("{introducer}{id}"));
                assert_eq!(
                    item["textEdit"]["range"],
                    json!({
                    "start":{"line":0,"character":before.encode_utf16().count()},
                    "end":{"line":0,"character":text.encode_utf16().count()}})
                );
                assert_eq!(
                    apply(&text, &item["textEdit"]),
                    format!("{before}{marker}{id}")
                );
            }
            for neighbor in [marker, trigger] {
                let before = format!("//! 😀 {marker}{other} ");
                let old = format!("{introducer}FS{separator}lost.1");
                let after = format!("{neighbor}{other} tail");
                let text = format!("{before}{old}{after}");
                let cursor = before.len() + format!("{introducer}FS{separator}lo").len();
                let items = s.completion(&path, &text, 0, cursor);
                let edit = &chosen(&items, &id, marker)["textEdit"];
                assert_eq!(
                    edit["range"],
                    json!({
                    "start":{"line":0,"character":before.encode_utf16().count()},
                    "end":{"line":0,"character":format!("{before}{old}").encode_utf16().count()}})
                );
                assert_eq!(apply(&text, edit), format!("{before}{marker}{id}{after}"));
                let before = format!("//! {neighbor}{other}");
                let text = format!("{before}{introducer}FS{separator}lo");
                let items = s.completion(&path, &text, 0, text.len());
                let edit = &chosen(&items, &id, marker)["textEdit"];
                assert_eq!(
                    edit["range"]["start"]["character"],
                    before.encode_utf16().count()
                );
                assert_eq!(apply(&text, edit), format!("{before}{marker}{id}"));
            }
            let text = format!("//! 😀 {introducer}FS{separator}lo");
            let items = s.completion(&path, &text, 0, text.len());
            let accepted = apply(&text, &chosen(&items, &id, marker)["textEdit"]);
            s.text(&path, &accepted);
            let response = s.request(
                "textDocument/definition",
                position(&path, &accepted, 0, accepted.len() - 1),
            );
            assert!(
                response["result"].to_string().contains("FS-login.md"),
                "{response}"
            );
            f.write("src/lib.rs", &accepted);
            let output = cli(&f.0, &[&id]);
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let output = cli(&f.0, &["fmt", "src/lib.rs", "--check"]);
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stdout)
            );
        }
    }
}
