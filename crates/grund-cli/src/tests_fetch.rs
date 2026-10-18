/// Test module: how `grund fetch` reports a failure core answered with
/// (§FS-fetch.7) — a query failure as its message, exit 1, and an operational
/// one, which a kind this release does not name also is, as `error: <message>`,
/// exit 2 (§AR-bindings.2.1).
#[cfg(test)]
mod tests_fetch {
    use super::*;

    #[test]
    fn a_fetch_failure_reports_cores_message_in_its_class() {
        assert_eq!(
            fetch_failure_report(FetchFailureKind::Query, "bad id"),
            ("bad id".to_owned(), 1)
        );
        assert_eq!(
            fetch_failure_report(FetchFailureKind::Operational, "no [fetch] configured"),
            ("error: no [fetch] configured".to_owned(), 2)
        );
    }
}
