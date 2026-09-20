//! Black-box contract for grounded chapter rules (§FS-rules). These tests use
//! only the shipped executable and repository fixtures, so the contract compiles
//! before the parser, fact producer, or evaluator exists.

#[path = "rules_contract/documentation.rs"]
mod documentation;
#[path = "rules_contract/refusals.rs"]
mod refusals;
#[path = "rules_contract/regressions.rs"]
mod regressions;
#[path = "rules_contract/support.rs"]
mod support;
#[path = "rules_contract/surfaces.rs"]
mod surfaces;
