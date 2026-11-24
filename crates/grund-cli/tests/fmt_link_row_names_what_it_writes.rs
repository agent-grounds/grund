//! Process-level coverage of §FS-fmt.3.6, kept with the end-to-end tests: each item's left side
//! is the span the write replaces (§FS-fmt.3.6), an item holding a title's `, ` is read by its
//! brackets (§FS-fmt.3.6), and the issue's rows are §FS-fmt.3.6's first case.

#[path = "../../../tests/e2e/fmt_link_row_names_what_it_writes.rs"]
mod contract;
