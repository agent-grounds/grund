//! §FS-check.checks.closed-citation-grounding: shared config diagnostics, not CLI-only text.
use super::config_findings::config_diagnostics;
use crate::config::load_config;
use crate::testing::{test_root, write};

#[test]
fn closed_citation_grounding_two_rows_are_two_unsited_diagnostics() {
    let root = test_root("grund-1460-two-rows");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 2\n\
        [schema.kinds.ce]\nfolders = [\"ce\"]\ncitable = false\n\
        [schema.kinds.other]\nfiles = [\"other.md\"]\ncitable = false\n\
        [rules.citations]\ndefault = \"must-not\"\n\
        [rules.citations.grounding]\nwarn = \"h2\"\n",
    );
    let config = load_config(&root).expect("valid config");
    let found: Vec<_> = config_diagnostics(&config, config.display())
        .filter(|d| d.code == "closed-citation-grounding")
        .collect();
    let expected: Vec<_> = ["ce", "other"]
        .iter()
        .map(|kind| {
            format!(
                "grund.toml: row `{kind}`: [rules.citations.grounding] warn = \"h2\" \
         asks for citations, but [rules.citations] default = \"must-not\" permits no target; \
         write an empty [rules.citations.{kind}.grounding] to disable grounding for this row, \
         or permit a target in [rules.citations.{kind}]"
            )
        })
        .collect();
    assert_eq!(
        found.iter().map(|d| d.message.clone()).collect::<Vec<_>>(),
        expected,
        "missing shared closed-citation-grounding diagnostics"
    );
    for diagnostic in found {
        assert!(diagnostic.path.is_none() && diagnostic.line.is_none());
        assert!(diagnostic.sites.is_empty());
    }
}
