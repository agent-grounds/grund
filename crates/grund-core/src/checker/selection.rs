use anyhow::{Result, anyhow};
use std::collections::BTreeSet;

use super::armed::ArmedChecks;
use crate::model::{Code, Finding, Report};

/// The nine rule-produced codes: a selection naming any of them also selects
/// `invalid-rule`, because a skipped rule is a check the run did not evaluate
/// (§FS-rules.7.6), and `empty-citation-obligation`, because an obligation with
/// no unit is one that judged nothing (§FS-check.1.4). The ninth is that line
/// itself, which a `must` or `should` rule writes (§FS-check.2.2.1.4).
const RULE_PRODUCED_CODES: &[&str] = &[
    "chapter-cardinality",
    "citation-cardinality",
    "uncited-unit",
    "unreached-declaration",
    "missing-citation",
    "forbidden-citation",
    "discouraged-citation",
    "suggested-citation",
    "empty-citation-obligation",
];

/// The origin a `check --rule` trial sentence carries, and so the authority
/// `--only-rule` asks for (§FS-rules.8, §AR-rules.2).
const TRIAL_RULE_ORIGIN: &str = "--rule";

/// The code of the line a selected check the run left off earns (§FS-check.1.4.1).
const SELECTED_CHECK_OFF: &str = Code::SELECTED_CHECK_OFF.as_str();

/// What a selection carries beside a rule-produced code (§FS-check.1.4).
const CARRIED_WITH_RULE_CODES: &[&str] = &["invalid-rule", "empty-citation-obligation"];

/// Repeatable exact-code selection for the two CLI check adapters, plus the
/// rule-authority axis `--only-rule` adds beside it (§FS-check.1.4). The
/// structured Rust API deliberately does not carry this presentation query, so
/// API and LSP callers continue to receive the complete report (§FS-check.1.4).
/// [`Self::select`] is the one narrowing the CLI and both bindings read
/// (§AR-bindings.2), and the place a selected check the run left off is said.
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
            // §FS-imports.checks: what a run read, or could not, about its imports.
            || code == "import-live"
            || code == "import-missing"
            // §FS-check.1.4.1: about the selection itself, so both axes pass it.
            || code == SELECTED_CHECK_OFF && !self.ignore.contains(code)
            || self.code_axis_retains(code)
                && self.authority_axis_retains(authority)
                && !self.ignore.contains(code)
    }

    /// The selected report of a complete check, for the CLI and the bindings
    /// alike: every finding both axes retain, in its order, then one
    /// `selected-check-off` warning per selected code the run left off, in code
    /// order (§FS-check.2.1.2, §FS-check.1.4.1).
    pub fn select(&self, report: &Report, armed: &ArmedChecks) -> Report {
        let retained = |findings: &[Finding]| {
            findings
                .iter()
                .filter(|finding| self.retains(finding.code, &finding.authority))
                .cloned()
                .collect::<Vec<_>>()
        };
        let mut selected = Report {
            errors: retained(&report.errors),
            warnings: retained(&report.warnings),
            suggestions: retained(&report.suggestions),
        };
        selected.warnings.extend(self.off_lines(armed));
        selected
    }

    /// §FS-check.1.4.1: `--only <code> judged nothing — <advice>` for each code
    /// `--only` names and `--ignore` does not that no project of the run armed.
    /// The set is ordered, so the lines come in code order.
    fn off_lines<'a>(&'a self, armed: &'a ArmedChecks) -> impl Iterator<Item = Finding> + 'a {
        self.only
            .iter()
            .filter(|_| !self.ignore.contains(SELECTED_CHECK_OFF))
            .filter(|code| !self.ignore.contains(*code))
            .filter_map(|code| {
                let advice = armed.off_advice(code)?;
                Some(Finding {
                    severity: "warning",
                    code: SELECTED_CHECK_OFF,
                    path: None,
                    line: None,
                    column: None,
                    message: format!("--only {code} judged nothing — {advice}"),
                    sites: Vec::new(),
                    authority: Vec::new(),
                })
            })
    }

    /// §FS-check.1.4: the code axis passes what `--only` names, and passes
    /// every `invalid-rule` and `empty-citation-obligation` row as well once it
    /// names a rule-produced code, so a narrowed run never reads a skipped rule
    /// or an obligation with nothing to judge as a pass (§FS-rules.7.6).
    fn code_axis_retains(&self, code: &str) -> bool {
        self.only.is_empty()
            || self.only.contains(code)
            || CARRIED_WITH_RULE_CODES.contains(&code)
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
    // §FS-errors.5.5: the vocabulary is the catalog's handles, sorted.
    if Code::named(value).is_none() {
        return Err(anyhow!(
            "unknown check finding code \"{value}\"; run `grund check --help` for supported codes"
        ));
    }
    Ok(())
}
