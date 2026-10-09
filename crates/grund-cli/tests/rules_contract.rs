//! Black-box contract for grounded chapter rules (§FS-rules). These tests use
//! only the shipped executable and repository fixtures, so the contract compiles
//! before the parser, fact producer, or evaluator exists.

#[path = "rules_contract/authority.rs"]
mod authority;
#[path = "rules_contract/chapter_path_reasons.rs"]
mod chapter_path_reasons;
#[path = "rules_contract/documentation.rs"]
mod documentation;
#[path = "rules_contract/paste_back.rs"]
mod paste_back;
#[path = "rules_contract/refusal_forms.rs"]
mod refusal_forms;
#[path = "rules_contract/refusals.rs"]
mod refusals;
#[path = "rules_contract/regressions.rs"]
mod regressions;
#[path = "rules_contract/selector_refusals.rs"]
mod selector_refusals;
#[path = "rules_contract/support.rs"]
mod support;
#[path = "rules_contract/surfaces.rs"]
mod surfaces;
#[path = "rules_contract/unreached_declaration.rs"]
mod unreached_declaration;
#[path = "rules_contract/workspace_scope.rs"]
mod workspace_scope;
