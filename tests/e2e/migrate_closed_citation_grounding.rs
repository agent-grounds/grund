//! §FS-config-migrate.5.2: C1 and inherited h2 grounding migrate with equal facts.
//! §FS-config-migrate.5.3's guidance normalization remains in force (#1440).
#[path = "support/closed_citation_grounding.rs"]
mod support;
use support::{Tree, config, message, row, streams, warnings};

#[test]
fn closed_citation_grounding_migrates_c1_and_inherited_h2() {
    let mut missing = Vec::new();
    for (name, text, source, unit) in [
        ("c1", row(1), "row", "file"),
        (
            "inherited-h2",
            config(
                1,
                "grounding_level = 2\n",
                "[reference]\nrequire_grounding = true\n",
            ),
            "project",
            "h2",
        ),
    ] {
        let tree = Tree::new(&format!("migrate-{name}"), &text);
        let before = tree.run(&["check"]);
        assert_eq!(before.status.code(), Some(1), "{}", streams(&before));
        if warnings(&before) != [format!("warning: {}", message(1, source, "", false))] {
            missing.push(format!("{name}: missing v1 warning: {}", streams(&before)));
        }
        let preview = tree.run(&["config", "migrate", "--to", "2"]);
        assert_eq!(preview.status.code(), Some(0), "{}", streams(&preview));
        assert!(
            String::from_utf8_lossy(&preview.stderr).contains("compared: the records are equal"),
            "{}",
            streams(&preview)
        );
        let written = tree.run(&["config", "migrate", "--to", "2", "--write", "--full"]);
        assert_eq!(written.status.code(), Some(0), "{}", streams(&written));
        let init = tree.run(&["init"]);
        assert_eq!(init.status.code(), Some(0), "{}", streams(&init));
        let after = tree.run(&["check"]);
        assert_eq!(after.status.code(), Some(1), "{}", streams(&after));
        let expected = message(2, "row", &format!("must = \"{unit}\""), false);
        if warnings(&after) != [format!("warning: {expected}")] {
            missing.push(format!(
                "{name}: missing v2 warning {expected}: {}",
                streams(&after)
            ));
        }
    }
    assert!(missing.is_empty(), "{}", missing.join("\n"));
}
