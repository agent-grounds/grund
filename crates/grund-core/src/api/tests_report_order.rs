//! Completed reports sort every channel stably, including suggestions (§FS-errors.4.1).

use super::report::complete_report;
use crate::model::{CheckReport, Code, Diagnostic};

/// Keep columns as non-key identities, including ties between absent and zero lines
/// (§FS-errors.4.1). Each channel receives its own unsorted copy.
#[test]
fn completed_report_sorts_all_channels_stably() {
    let entries = [
        (Some("b.md"), Some(1), "A", 1),
        (Some("a.md"), Some(2), "A", 2),
        (None, Some(1), "A", 3),
        (Some("a.md"), Some(1), "a", 4),
        (Some("a.md"), Some(1), "Z", 5),
        (None, None, "a", 6),
        (Some("a.md"), Some(0), "same", 9),
        (None, Some(0), "Z", 10),
        (Some("a.md"), None, "same", 7),
        (None, None, "Z", 8),
        (Some("A.md"), Some(9), "z", 11),
    ];
    let unsorted = entries
        .into_iter()
        .map(|(path, line, message, column)| Diagnostic {
            code: Code::UNUSED,
            path: path.map(Into::into),
            line,
            column: Some(column),
            message: message.into(),
            sites: Vec::new(),
            authority: Vec::new(),
        })
        .collect::<Vec<_>>();
    let mut report = CheckReport {
        errors: unsorted.clone(),
        warnings: unsorted.clone(),
        suggestions: unsorted,
        ..CheckReport::default()
    };

    complete_report(&mut report);

    for channel in [&report.errors, &report.warnings, &report.suggestions] {
        let order = channel
            .iter()
            .map(|d| d.column.unwrap())
            .collect::<Vec<_>>();
        assert_eq!(order, [10, 8, 6, 3, 11, 9, 7, 5, 4, 2, 1]);
    }
}
