// §FS-declarations.checks.parse-moves.2: shape 4, a relative heading in a doc-comment body,
// run alone; the full corpus discovers these same cases.
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

// §FS-declarations.checks.parse-moves.1: the coordinate message in each doc-comment language.
case!(
    warns_in_a_rust_doc_comment,
    "check-parse-moves-relative-heading-rust"
);
case!(
    warns_in_a_python_docstring,
    "check-parse-moves-relative-heading-python"
);
case!(
    warns_in_a_java_block_comment,
    "check-parse-moves-relative-heading-java"
);
case!(
    warns_in_a_go_comment,
    "check-parse-moves-relative-heading-go"
);
case!(
    warns_as_one_json_object,
    "check-parse-moves-relative-heading-json"
);
case!(
    names_the_missing_parent,
    "check-parse-moves-relative-heading-no-parent"
);
case!(
    warns_once_on_a_bare_block_comment_line,
    "check-parse-moves-relative-heading-bare-block-line"
);

// §FS-declarations.checks.parse-moves.3: the shape is selected with its siblings.
case!(only_keeps_it, "check-parse-moves-relative-heading-only");
case!(ignore_drops_it, "check-parse-moves-relative-heading-ignore");

// §FS-declarations.section.relative.where: what keeps its meaning in both releases.
case!(
    a_code_span_stays_prose,
    "check-parse-moves-relative-heading-code-span"
);
case!(
    a_bold_label_stays_prose,
    "check-parse-moves-relative-heading-bold-label"
);
case!(
    a_fenced_line_stays_content,
    "check-parse-moves-relative-heading-fence"
);
case!(
    outside_any_body_stays_prose,
    "check-parse-moves-relative-heading-outside-body"
);
case!(
    a_complete_path_is_unchanged,
    "check-parse-moves-relative-heading-complete-path"
);
case!(
    a_refused_form_stays_prose,
    "check-parse-moves-relative-heading-refused-form"
);
