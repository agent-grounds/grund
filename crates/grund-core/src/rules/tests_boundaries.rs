//! Pending boundary drivers for §AR-rules.6.
//!
//! Each sentinel is deliberately red when forced with `--ignored`. The
//! implementation replaces its body with the named arrangement and assertion,
//! then removes `#[ignore]`; the architecture guard refuses a production
//! `rules` module while any sentinel remains.

#[test]
#[ignore = "implementation pending: parse through ParsedRule only"]
fn sentence_front_end_returns_complete_parsed_rule_without_facts_or_diagnostics() {
    panic!(
        "replace with hand-written title, origin, anchor, and vocabulary; assert every ParsedRule field"
    );
}

#[test]
#[ignore = "implementation pending: evaluate boundary values only"]
fn logic_engine_evaluates_hand_built_rule_and_facts_without_parser_or_scanner() {
    panic!("replace with hand-built ParsedRule and complete RuleFacts; assert located diagnostics");
}

#[test]
#[ignore = "implementation pending: prove producer replacement"]
fn markdown_adapter_and_second_producer_drive_the_same_engine_result() {
    panic!(
        "replace with equivalent Markdown-adapter and test-producer RuleFacts; assert equal diagnostics"
    );
}

#[test]
#[ignore = "implementation pending: gate closed-world conclusions"]
fn incomplete_fact_snapshot_suppresses_absence_and_count_conclusions() {
    panic!(
        "replace with complete and incomplete snapshots; assert only the complete one yields absence/count conclusions"
    );
}
