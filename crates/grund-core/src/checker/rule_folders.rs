//! §FS-config-v2.checks.rule-folder-file-not-markdown: a file the walk read from a rule kind's
//! folder by a `source` language, which only the walk can see, so `check` says it after the walk
//! rather than the configuration at load (§FS-config-v2.schema.sources.rules).

use crate::config::{Extent, Form, Frame, LanguageClass, Owners, Schema, can_hold_markdown};
use crate::model::{Catalog, CheckReport, Code, Diagnostic, remedies};

/// §FS-config-v2.checks.rule-folder-file-not-markdown: one warning at line 1 of each scanned
/// file a `source` language reads in a `form = "rule"` row's folder. Never for version 1, and
/// never in a folder `config-rule-folder-not-markdown` already names.
pub(super) fn check_rule_folder_files(
    schema: &Schema,
    catalog: &Catalog,
    frame: Frame<'_>,
    report: &mut CheckReport,
) {
    let Some(model) = schema.sources.model.as_ref() else {
        return;
    };
    let rule_rows = schema.rows.iter().any(|row| {
        row.kind
            .as_ref()
            .is_some_and(|kind| kind.form == Form::Rule)
            && row.folders().next().is_some()
    });
    if !rule_rows || !can_hold_markdown(model) {
        return;
    }
    let owners = Owners::new(schema, frame);
    for file in &catalog.scanned_files {
        let Some(owner) = owners.of(file) else {
            continue;
        };
        let Extent::Folder(folder) = &owner.place.extent else {
            continue;
        };
        if !owner
            .row
            .kind
            .as_ref()
            .is_some_and(|kind| kind.form == Form::Rule)
        {
            continue;
        }
        let Some(language) = frame.reading(schema, file).language else {
            continue;
        };
        if language.class != LanguageClass::Source {
            continue;
        }
        let (name, kind) = (&language.name, &owner.row.name);
        report.warnings.push(Diagnostic {
            code: Code::RULE_FOLDER_FILE_NOT_MARKDOWN,
            path: Some(file.clone()),
            line: Some(1),
            column: None,
            message: format!(
                "read as `{name}`, which does not read Markdown, in `{folder}`, a folder home of \
                 rule kind `{kind}`, so a rule written as a Markdown heading here declares \
                 nothing; {}",
                remedies::rule_folder_file_not_markdown(format!("`{folder}`"))
            ),
            sites: Vec::new(),
            authority: Vec::new(),
        });
    }
}
