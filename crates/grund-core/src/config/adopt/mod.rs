//! `grund config adopt-builtins` (§FS-config-adopt-builtins, §AR-config.10): the plan, built from
//! the file as read and version 2's epoch alone (§AR-config.10.1), then a text edit of the three
//! language keys (§AR-config.10.2), then the gate (§AR-config.10.3). What sits above config, the
//! two `check` runs and the write, is the api layer's. It renders nothing; `grund-cli` prints the
//! record.

mod edit;
mod gate;
mod plan;
mod record;
#[cfg(test)]
mod tests_adopt;

use std::path::Path;

use anyhow::Context;

use super::discovery::config_with_spans;
use super::migrate::Located;
use super::run::{Invocation, Selection};
use super::v2;
pub use record::{
    AddedSuffixes, AdoptComment, AdoptFailure, AdoptOutcome, AdoptReport, AdoptedSuffix, Compared,
    KeptSuffix, KeptWhy,
};

/// What config found at the located file, before `check` runs.
pub(crate) enum Prepared {
    /// §FS-config-adopt-builtins.9: no suffix is adopted.
    Nothing(AdoptOutcome),
    /// §FS-config-adopt-builtins.8.1: a symbolic link, refused before the plan.
    Symlink(Box<AdoptReport>),
    /// The plan, its candidate and what the gate said, with the text the plan was built from.
    Planned {
        report: Box<AdoptReport>,
        text: String,
    },
}

/// §FS-config-adopt-builtins: adopt in the config at `located`, everything but the `check`
/// comparison and the write.
pub(crate) fn prepare(
    located: &Located,
    write: bool,
    drop_comments: bool,
) -> Result<Prepared, AdoptFailure> {
    let mut report = AdoptReport {
        config: located.config.clone(),
        write,
        drop_comments,
        ..AdoptReport::default()
    };
    // §FS-config-adopt-builtins.8.1, §FS-config-migrate.6.4: renaming over a link would replace it.
    if std::fs::symlink_metadata(&located.file).is_ok_and(|meta| meta.file_type().is_symlink()) {
        report.symlink = true;
        return Ok(Prepared::Symlink(Box::new(report)));
    }
    let text = super::input_read_to_string(&located.file)
        .with_context(|| format!("read {}", located.config))
        .map_err(AdoptFailure::Load)?;
    // §FS-config-adopt-builtins.10: version 1 is migration's first.
    if !v2::selects(&text) && matches!(v2::version(&text), None | Some("1")) {
        return Err(AdoptFailure::Version1 {
            config: located.config.clone(),
        });
    }
    let invocation = Invocation::new(&located.root, false, Selection::Project);
    let shown = Path::new(&located.config);
    let (config, spans) =
        config_with_spans(&text, shown, &located.root, &invocation).map_err(AdoptFailure::Load)?;
    let project = config.project();
    let Some(model) = project.schema.sources.model.as_ref() else {
        return Err(AdoptFailure::Version1 {
            config: located.config.clone(),
        });
    };
    let plan = plan::plan(model);
    if !plan.adopts() {
        return Ok(Prepared::Nothing(AdoptOutcome::NothingToAdopt {
            config: located.config.clone(),
            kept: plan.kept(),
        }));
    }
    let edited = edit::edit(&text, &spans, model, &plan);
    report.adopted = plan.adopted(model);
    report.kept = plan.kept();
    // §FS-config-adopt-builtins.5.2 item 3: in the candidate's `languages` order.
    report.added = plan.added.clone();
    report.added.sort_by_key(|added| {
        edited
            .languages
            .iter()
            .position(|name| *name == added.row)
            .unwrap_or(usize::MAX)
    });
    report.comments = edited.comments;
    report.languages = edited.languages;
    report.candidate = edited.candidate;
    gate::gate(&mut report, project, &plan, &located.root);
    Ok(Prepared::Planned {
        report: Box::new(report),
        text,
    })
}

/// §FS-config-adopt-builtins.6.4: whether the config still holds `text`, the bytes the plan was
/// built from, read again after both `check` runs.
pub(crate) fn still_holds(located: &Located, text: &str) -> bool {
    super::input_read_to_string(&located.file).is_ok_and(|now| now == text)
}
