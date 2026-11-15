//! The remedies of the six version-2 load refusals grund 1.0.0 made of grund 0.23.0's warnings
//! (§DF-markdown-homes-refused-in-1-0.2.1), worded in `config/` as `fetch_key_repair` is,
//! because their codes are spent and the §FS-errors.5.5 catalog no longer carries a *Fix* to
//! fill (§FS-config-v2.checks.config-md-not-markdown).

/// The suffix every one of the six refusals ends with
/// (§FS-config-v2.checks.config-md-not-markdown, §REQ-backwards-compatibility.2).
pub(in crate::config) const BECAME_1_0: &str = "; this became an error in grund 1.0.0";

/// §FS-config-v2.checks.config-md-not-markdown
pub(in crate::config) const MD_NOT_MARKDOWN: &str =
    "drop the entry, or map `md` to a `document` language";

/// §FS-config-v2.checks.config-index-not-markdown
pub(in crate::config) const INDEX_NOT_MARKDOWN: &str =
    "name a file a `document` language reads, or set `index = false`";

/// §FS-config-v2.checks.config-rule-file-not-markdown
pub(in crate::config) const RULE_FILE_NOT_MARKDOWN: &str =
    "move the rules into a file a `document` language reads";

/// §FS-config-v2.checks.config-rule-folder-not-markdown
pub(in crate::config) const RULE_FOLDER_NOT_MARKDOWN: &str =
    "admit one in [schema.sources] languages, such as `markdown`";

/// §FS-config-v2.checks.config-value-home-absent: a later home of `shape`, `file` or `folder`.
pub(in crate::config) fn value_home_absent(shape: &str) -> String {
    format!("create the {shape}, or drop the entry")
}

/// §FS-config-v2.checks.config-value-home-invalid: a later home that is not an existing `shape`.
pub(in crate::config) fn value_home_wrong_shape(shape: &str) -> String {
    format!("make it an existing {shape}, or drop the entry")
}

/// §FS-config-v2.checks.config-value-home-invalid: a later home that resolves outside the root.
pub(in crate::config) const VALUE_HOME_OUTSIDE: &str =
    "name a path inside the project root, or drop the entry";

/// §FS-config-v2.checks.config-value-home-invalid, §FS-fetch.4.3: a home that cannot be looked
/// up, first or later; the first entry's refusal gives it as its `hint:`.
pub(in crate::config) const VALUE_HOME_LOOKUP: &str =
    "make the home and every directory above it accessible";
