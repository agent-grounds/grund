use anyhow::{Result, anyhow};
use std::collections::BTreeSet;

/// The exact public code vocabulary accepted by `grund check --only` and
/// `--ignore`, kept sorted for deterministic help output (§FS-errors.5.5).
#[doc(hidden)]
pub const CHECK_FINDING_CODES: &[&str] = &[
    "agents-init",
    "broken-stub",
    "chapter-cardinality",
    "citation-cardinality",
    "dangling",
    "declaration-near-miss",
    "deprecated-config-location",
    "discouraged-citation",
    "duplicate",
    "duplicate-section",
    "empty-citation-obligation",
    "empty-scan",
    "escaped-citation-resolves",
    "forbidden-citation",
    "full-scope-ignored",
    "glob-citation",
    "inline-citation-style",
    "invalid-rule",
    "invalid-value-binding",
    "invalid-value-declaration",
    "io",
    "local-section-citation",
    "misplaced-declaration",
    "missing-citation",
    "missing-index-entry",
    "missing-section",
    "missing-snapshot",
    "nothing-recognized",
    "optional-member-absent",
    "orphan-section",
    "out-of-scope-dangling",
    "out-of-scope-local-section-citation",
    "out-of-scope-missing-section",
    "out-of-scope-shorthand-citation",
    "out-of-scope-unknown-project",
    "oversized-lead",
    "redundant-config",
    "section-heading-level",
    "section-outside-declaration",
    "shorthand-citation",
    "shorthand-numeric-run",
    "suggested-citation",
    "uncited-unit",
    "ungrounded",
    "unknown-project",
    "unlinked-index-entry",
    "unlisted-workspace-block",
    "unmarked-heading",
    "unreached-declaration",
    "unused",
    "value-mismatch",
];

/// The eight rule-produced codes: a selection naming any of them also selects
/// `invalid-rule`, because a skipped rule is a check the run did not evaluate
/// (§FS-rules.7.6).
const RULE_PRODUCED_CODES: &[&str] = &[
    "chapter-cardinality",
    "citation-cardinality",
    "uncited-unit",
    "unreached-declaration",
    "missing-citation",
    "forbidden-citation",
    "discouraged-citation",
    "suggested-citation",
];

/// The origin a `check --rule` trial sentence carries, and so the authority
/// `--only-rule` asks for (§FS-rules.8, §AR-rules.2).
const TRIAL_RULE_ORIGIN: &str = "--rule";

/// Repeatable exact-code selection for the two CLI check adapters, plus the
/// rule-authority axis `--only-rule` adds beside it (§FS-check.1.4). The
/// structured Rust API deliberately does not carry this presentation query, so
/// API and LSP callers continue to receive the complete report (§FS-check.1.4).
#[doc(hidden)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CheckFindingSelection {
    only: BTreeSet<String>,
    ignore: BTreeSet<String>,
    only_rule: bool,
}

impl CheckFindingSelection {
    /// Add one selector value after applying the pre-scan CLI validation and
    /// exact error vocabulary from §FS-cli.4.
    pub fn add_only(&mut self, value: &str) -> Result<()> {
        validate_check_finding_code("--only", value)?;
        self.only.insert(value.to_string());
        Ok(())
    }

    /// Add one ignored code; repetitions collapse into the specified set
    /// semantics (§FS-check.1.4).
    pub fn add_ignore(&mut self, value: &str) -> Result<()> {
        validate_check_finding_code("--ignore", value)?;
        self.ignore.insert(value.to_string());
        Ok(())
    }

    /// Narrow the report to what the `--rule` trial sentence authored
    /// (§FS-rules.8). A boolean with no value to validate, and repeating it is
    /// the same request twice (§FS-check.1.4).
    pub fn scope_to_trial_rule(&mut self) {
        self.only_rule = true;
    }

    /// Whether `--only-rule` was asked for, which the CLI reads to refuse the
    /// flag without a `--rule` sentence before any config discovery
    /// (§FS-rules.8, §FS-check.1.4).
    pub fn scopes_to_trial_rule(&self) -> bool {
        self.only_rule
    }

    /// Decide whether a completed check's diagnostic enters the selected
    /// report. The two axes intersect — a finding passes when its code passes
    /// *and* its authority does — while `--ignore` wins over both and `io`
    /// cannot be hidden at all, because it marks a read the run could not make
    /// (§FS-check.1.4, §FS-check.2.4). Unhideable is not the same as exiting
    /// `2`: the §FS-check.1.3.6.3 unread-source caution wears this code and is
    /// unselectable with it, and is a warning (§FS-check.2.1.2).
    pub fn retains(&self, code: &str, authority: &[String]) -> bool {
        code == "io"
            || self.code_axis_retains(code)
                && self.authority_axis_retains(authority)
                && !self.ignore.contains(code)
    }

    /// §FS-check.1.4: the code axis passes what `--only` names, and passes
    /// every `invalid-rule` row as well once it names a rule-produced code, so
    /// a narrowed run never reads a skipped rule as a pass (§FS-rules.7.6).
    fn code_axis_retains(&self, code: &str) -> bool {
        self.only.is_empty()
            || self.only.contains(code)
            || code == "invalid-rule"
                && RULE_PRODUCED_CODES
                    .iter()
                    .any(|produced| self.only.contains(*produced))
    }

    /// §FS-rules.8: the authority axis keeps a finding the trial sentence
    /// authored, whether it authored it alone or jointly with a declared rule
    /// that reached the same meaning (§FS-rules.6).
    fn authority_axis_retains(&self, authority: &[String]) -> bool {
        !self.only_rule || authority.iter().any(|origin| origin == TRIAL_RULE_ORIGIN)
    }
}

fn validate_check_finding_code(flag: &str, value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(anyhow!("{flag} requires a finding code"));
    }
    let valid_shape = value.split('-').all(|part| {
        !part.is_empty()
            && part
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    });
    if !valid_shape {
        return Err(anyhow!(
            "invalid finding code \"{value}\" (expected lowercase kebab-case)"
        ));
    }
    if CHECK_FINDING_CODES.binary_search(&value).is_err() {
        return Err(anyhow!(
            "unknown check finding code \"{value}\"; run `grund check --help` for supported codes"
        ));
    }
    Ok(())
}
