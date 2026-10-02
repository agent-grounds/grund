// §FS-refs.4: ambiguity refusals and their exact-coordinate boundaries use
// the ordinary e2e fixture runner; each case also runs in the full e2e suite.
#[path = "support/case_runner.rs"]
#[allow(dead_code)]
mod case_runner;

use case_runner::{CaseKind::E2e, assert_every_case_passed, run_case};

macro_rules! case {
    ($test:ident, $name:literal) => {
        #[test]
        fn $test() {
            let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
            let outcome = run_case(&root, &root.join("tests/e2e/cases").join($name), E2e);
            assert_every_case_passed($name, &[outcome]);
        }
    };
}

case!(refs_ambiguous_section, "refs-ambiguous-section");
case!(refs_ambiguous_section_flag, "refs-ambiguous-section-flag");
case!(refs_ambiguous_section_json, "refs-ambiguous-section-json");
case!(
    refs_ambiguous_section_summary,
    "refs-ambiguous-section-summary"
);
case!(refs_ambiguous_id, "refs-ambiguous-id");
case!(refs_ambiguous_id_json, "refs-ambiguous-id-json");
case!(refs_ambiguous_id_total, "refs-ambiguous-id-total");
case!(
    refs_ambiguous_section_descendants,
    "refs-ambiguous-section-descendants"
);
case!(
    refs_unrelated_duplicate_section,
    "refs-unrelated-duplicate-section"
);
case!(
    refs_bare_id_duplicate_section,
    "refs-bare-id-duplicate-section"
);
case!(
    refs_absent_target_with_duplicate_section,
    "refs-absent-target-with-duplicate-section"
);
case!(
    refs_absent_section_with_duplicate_section,
    "refs-absent-section-with-duplicate-section"
);
case!(refs_stub_inline_one_home, "refs-stub-inline-one-home");
case!(
    workspace_refs_ambiguous_section_json,
    "workspace-refs-ambiguous-section-json"
);
case!(workspace_refs_undeclared, "workspace-refs-undeclared");
case!(
    workspace_refs_json_target_grammar,
    "workspace-refs-json-target-grammar"
);
case!(refs_ambiguous_shorthand, "refs-ambiguous-shorthand");
case!(
    refs_ambiguous_shorthand_json,
    "refs-ambiguous-shorthand-json"
);
