//! §AR-config.8: `grund config migrate --to 2`'s translation (§FS-config-migrate).
//! It reads the v1 file once, through the v1 reader's tolerant line pass
//! (§AR-config.8.1), converts its `Project` into v2-shaped records that
//! `Project::v2_toml` spells (§AR-config.7.4), loads that text back through
//! `pending_config` and holds the result to the records gate
//! (§FS-config-migrate.5.1). It returns one typed record and renders nothing.
//!
//! The report gate and the write sit above config, in `api/config_migrate.rs`:
//! one runs `check`, the other is a writer, and config reads neither
//! (§AR-system.4).

mod candidate;
mod comment_guidance;
mod comments;
mod keymap;
mod keys;
mod languages;
mod places;
mod record;
mod records_gate;
mod refusals;
#[cfg(test)]
mod tests_comments;
#[cfg(test)]
mod tests_gates;
#[cfg(test)]
mod tests_mapping;

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use super::compiled::compile;
use super::discovery::{config_file_in, discovery_start_dir, pending_config};
use super::kind::KindResolution;
use super::project::Project;
use super::run::{Invocation, Selection};
use super::v1::{LinePass, LineRecord};
use super::{v1, v2, validate};
use crate::model::{format_path, relative_from_base};
pub use record::{
    Gate, GateResult, ListedComment, MappedKey, MigrateFailure, MigrateNotice, MigrateOutcome,
    MigrateRefusal, MigrateReport, RefusalAnchor,
};

/// The config a run migrates (§FS-config-migrate.1.1): where it is, the root it
/// governs, and its path from the report base.
pub(crate) struct Located {
    pub(crate) root: PathBuf,
    pub(crate) file: PathBuf,
    pub(crate) config: String,
}

/// §FS-config-migrate.1.1: discovery walks up from `path`, as `config validate`
/// does, and the governing config of the first directory that has one is it.
pub(crate) fn locate(path: &Path) -> Option<Located> {
    let start = discovery_start_dir(path).to_path_buf();
    let walk_start = fs::canonicalize(&start).unwrap_or(start);
    let mut cursor = Some(walk_start.as_path());
    while let Some(dir) = cursor {
        if let Some(file) = config_file_in(dir) {
            let from_cli = super::call_scope::report_path_base()
                == Some(super::call_scope::PathBase::Invocation);
            let shown = if from_cli {
                relative_from_base(&walk_start, &file)
            } else {
                file.strip_prefix(dir)
                    .map_or_else(|_| file.clone(), Path::to_path_buf)
            };
            return Some(Located {
                root: dir.to_path_buf(),
                file,
                config: format_path(&shown),
            });
        }
        cursor = dir.parent();
    }
    None
}

/// §FS-config-migrate.8: the config of the member at `root` when it is still on
/// version 1. Config reads the file, so the surface above it never does
/// (§AR-system.2.13).
pub(crate) fn member_on_v1(root: &Path) -> Option<PathBuf> {
    let file = config_file_in(root)?;
    let text = super::input_read_to_string(&file).ok()?;
    (!v2::selects(&text)).then_some(file)
}

/// What the translation found, before the report gate and the write.
pub(crate) enum Prepared {
    AlreadyV2,
    /// The report so far, the v1 text and project it was built from, and each
    /// candidate line that carries a key with its v1 line.
    Translated {
        report: Box<MigrateReport>,
        v1_text: String,
        v1: Box<Project>,
        line_map: Vec<(usize, usize)>,
        /// Each value row with the line of its `values` key, which the
        /// guidance comparison names (§FS-config-migrate.5.3).
        value_lines: Vec<(String, usize)>,
    },
}

/// The v1 file `text` as its project and its line pass, judged as `config
/// validate` judges it but read tolerantly (§AR-config.8.1).
fn read_v1(text: &str, report: &Path, root: &Path) -> Result<(Project, Vec<LineRecord>)> {
    let mut pass = LinePass::tolerant();
    let read = v1::read_sections_with(text, report, &mut pass)?;
    validate::reference(report, &read.project)?;
    let project = read.lower_kinds(report, root)?;
    validate::kinds(report, &project)?;
    compile(&project).with_context(|| format!("{}: invalid [id] grammar", format_path(report)))?;
    validate::lists(report, &project)?;
    Ok((project, pass.lines))
}

/// §FS-config-migrate: translate the config at `located`, everything but the
/// report gate, the members and the write.
pub(crate) fn translate(located: &Located, write: bool, drop_comments: bool) -> Result<Prepared> {
    let mut report = MigrateReport {
        config: located.config.clone(),
        write,
        drop_comments,
        ..MigrateReport::default()
    };
    // §FS-config-migrate.6.4: renaming over a link would replace it.
    if fs::symlink_metadata(&located.file).is_ok_and(|meta| meta.file_type().is_symlink()) {
        report.refusals.push(MigrateRefusal {
            anchor: RefusalAnchor::File,
            what: "is a symbolic link, and replacing it would replace the link with a regular file".into(),
            details: Vec::new(),
            repair: "replace the link with the file it points to, or migrate that file where it is the config".into(),
        });
        return Ok(translated(report, String::new(), v1::default_project()));
    }
    let shown = Path::new(&located.config);
    let text = super::input_read_to_string(&located.file)
        .with_context(|| format!("read {}", located.config))?;
    if v2::selects(&text) {
        return Ok(Prepared::AlreadyV2);
    }
    let (project, lines) = read_v1(&text, shown, &located.root)?;
    report.v1_lines = text.lines().count();
    let keys = keys::Keys::of(&lines);
    let root = &located.root;
    let include = places::include(&project, &keys, root);
    let values_home = project.schema.value_rows().next().is_some();
    let languages = languages::translate(
        &project,
        keys.line("scan", "extensions"),
        keys.line("scan", "comment_prefixes"),
        values_home,
    );
    let mut refusals = refusals::settings(&project, &keys, root);
    refusals.extend(places::nested_homes(&project, &keys, root));
    refusals.extend(include.refusals.iter().cloned());
    refusals.extend(languages.refusals.iter().cloned());
    refusals.extend(refusals::strings_closing_early(&keys, shown));
    refusals.sort_by_key(|refusal| match refusal.anchor {
        RefusalAnchor::Line(line) => line,
        RefusalAnchor::Unset | RefusalAnchor::File => usize::MAX,
    });
    report.refusals = refusals;
    let records = candidate::records(&project, languages.model, include.places());
    let body = records
        .v2_toml()
        .expect("migration renders a version 2 project");
    let (_, notices) = keymap::map(&keys, &records, &include);
    report.notices = notices;
    // §FS-config-migrate.2.2: no promised destinations before a candidate loads.
    let unavailable = comment_guidance::Guidance {
        lines: &lines,
        mapped: &[],
        available: false,
    };
    if !report.refusals.is_empty() {
        comments::carry(&body, &lines, &keys, &unavailable, &mut report);
        return Ok(translated(report, text, project));
    }
    let invocation = Invocation::new(root, false, Selection::Project);
    let loaded = match pending_config(&body, root, &invocation) {
        Ok(config) => config.project().clone(),
        Err(error) => {
            comments::carry(&body, &lines, &keys, &unavailable, &mut report);
            report.gates = Some(GateResult::Differs {
                gate: Gate::Records,
                lines: vec![format!("+ the candidate does not load: {error:#}")],
            });
            return Ok(translated(report, text, project));
        }
    };
    // *fetch resolves must*: the reload puts v2's default for a fetched kind
    // back, and the candidate leaves it unwritten, as the mapping row says.
    let mut shown = loaded.clone();
    shown
        .rules
        .resolution
        .retain(|_, resolution| *resolution != KindResolution::Must);
    let body = shown.v2_toml().expect("a version 2 project renders");
    // §FS-config-migrate.4.1.1: the rendered destinations decide which comments need review.
    let comment_map = keymap::comment_map(&keys, &shown, &include);
    let guidance = comment_guidance::Guidance {
        lines: &lines,
        mapped: &comment_map,
        available: true,
    };
    let carried = comments::carry(&body, &lines, &keys, &guidance, &mut report);
    report.mapped = keymap::map(&keys, &shown, &include).0;
    report.written = keymap::written(&keys, &project, &body);
    report.code_row_note = !include.places().is_empty();
    let differences = records_gate::compare(&project, &loaded, root);
    report.gates = Some(if differences.is_empty() {
        GateResult::Equal
    } else {
        GateResult::Differs {
            gate: Gate::Records,
            lines: differences,
        }
    });
    let line_map = comments::line_map(&carried, &lines, &keys);
    report.candidate = Some(carried);
    let value_lines = project
        .schema
        .value_rows()
        .filter_map(|row| Some((row.name.clone(), keys.row_line(&row.name, "values")?)))
        .collect();
    let mut prepared = translated(report, text, project);
    if let Prepared::Translated {
        line_map: map,
        value_lines: values,
        ..
    } = &mut prepared
    {
        *map = line_map;
        *values = value_lines;
    }
    Ok(prepared)
}

fn translated(report: MigrateReport, v1_text: String, v1: Project) -> Prepared {
    Prepared::Translated {
        report: Box::new(report),
        v1_text,
        v1: Box::new(v1),
        line_map: Vec::new(),
        value_lines: Vec::new(),
    }
}
