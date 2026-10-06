// §FS-declarations.line.configured-literals: run the reported list/show fixture
// through the ordinary e2e harness; the full corpus discovers these same cases.
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

case!(
    colon_format_list_discovers_the_complete_id,
    "list-colon-format"
);
case!(
    colon_format_bare_show_reads_the_lead,
    "show-colon-format-bare"
);
case!(
    colon_format_explicit_show_reads_the_lead,
    "show-colon-format-explicit"
);
case!(
    colon_format_show_reads_a_section,
    "show-colon-format-section"
);
case!(
    colon_format_underscore_control,
    "show-colon-format-underscore-control"
);

// §FS-config.3.2.5: canonical parse failures retain discovery, reading and diagnostics.
case!(
    numeric_overflow_list_retains_the_exact_declaration,
    "list-numeric-overflow"
);
case!(
    numeric_overflow_show_reads_the_compatibility_body,
    "show-numeric-overflow"
);
case!(
    numeric_overflow_check_reports_the_near_miss,
    "check-numeric-overflow"
);
