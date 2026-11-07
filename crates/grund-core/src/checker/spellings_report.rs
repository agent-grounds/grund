//! What a report says of a file reached by two names (§FS-scan.identity.findings). The place rules
//! judge each spelling at its own path through [`PlaceNames`], and the one pass where the report is
//! finished, [`name_every_spelling`], adds the `sites` and the `(also at …)` clause rather than
//! each check doing so (§AR-scanner.1.8). All of it reads the run's `Spellings`, and a tree with
//! no second name skips it at the first question.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::finding_sides::Side;
use super::reference_scope::tag_out_of_scope_for_version;
use crate::config::{Frame, Schema, WalkBounds};
use crate::model::remedies;
use crate::model::{
    Catalog, CheckReport, Citation, Code, Diagnostic, ReadingConflict, ReadingMove, Site,
};
use crate::scanner::file_home_kind;
use crate::sources::Spellings;

/// The names of a file a report may speak of: those the plain run reaches, so that `--full` stays
/// additive (§FS-check.1.3.2, §FS-scan.identity.findings.content).
pub(super) struct PlaceNames<'a> {
    spellings: &'a Spellings,
    active: bool,
    plain: Option<WalkBounds>,
}

impl<'a> PlaceNames<'a> {
    pub(super) fn new(schema: &Schema, frame: Frame<'a>) -> Self {
        let spellings = &frame.run.spellings;
        let active = spellings.has_second_names();
        // §FS-check.1.3.2: the configured roots, whichever path this run was handed.
        let plain = active.then(|| WalkBounds::plain(schema, frame.root()));
        Self {
            spellings,
            active,
            plain,
        }
    }

    /// Every name of the file at `file` the plain run reaches, in listing order. Empty where the
    /// table knows no second name, or where the plain run reaches none of them.
    fn plain_names(&self, file: &Path) -> Vec<PathBuf> {
        if !self.active {
            return Vec::new();
        }
        let mut names = self.spellings.names_of(file);
        if let Some(plain) = &self.plain {
            names.retain(|name| {
                plain.contains(name) && self.spellings.plain_name_of(name).is_none()
            });
        }
        names
    }

    /// Those names, where there are two to speak of: empty otherwise
    /// (§FS-scan.identity.findings).
    fn reached(&self, file: &Path) -> Vec<PathBuf> {
        let mut names = self.plain_names(file);
        if names.len() < 2 {
            names.clear();
        }
        names
    }

    /// The name a whole run's report gives the file at `file`: the first the plain run reaches, or
    /// `file` where it reaches none (§FS-scan.identity.findings.content). A run naming a path holds
    /// the name it was handed, so a rule that writes a path into a message asks this first.
    pub(super) fn first(&self, file: &Path) -> PathBuf {
        let mut names = self.plain_names(file);
        if names.is_empty() {
            file.to_path_buf()
        } else {
            names.swap_remove(0)
        }
    }

    /// `sites` at the names of [`Self::first`], so a rule renders the whole run's text from them
    /// and sorts them as it does. A value source's declaration is copied per spelling
    /// (§FS-values.2.3), so a site whose first name is already a site keeps its own.
    pub(super) fn settle(&self, sites: &mut [Site]) {
        let held: BTreeSet<(PathBuf, usize)> = sites
            .iter()
            .map(|site| (site.path.clone(), site.line))
            .collect();
        for site in sites {
            let first = self.first(&site.path);
            if !held.contains(&(first.clone(), site.line)) {
                site.path = first;
            }
        }
    }

    /// The spellings a place rule judges `file` at (§FS-scan.identity.findings.place): every name
    /// the plain run reaches, or the file alone where it has no other.
    pub(super) fn of(&self, file: &Path) -> Vec<PathBuf> {
        let names = self.reached(file);
        if names.is_empty() {
            vec![file.to_path_buf()]
        } else {
            names
        }
    }

    /// What makes a citation-direction finding about a declaration a content finding
    /// (§FS-scan.identity.findings.content): its one site, at the name a whole run gives the file,
    /// which the finishing pass grows into every name ([`Side::of_finding`]). Nothing where no other
    /// name reaches the report, so that file keeps the bytes it had and goes by the code's side.
    pub(super) fn subject_sites(&self, file: &Path, line: usize) -> Vec<Site> {
        let names = self.plain_names(file);
        if names.len() < 2 && names.first().is_none_or(|first| first == file) {
            return Vec::new();
        }
        vec![Site {
            path: self.first(file),
            line,
        }]
    }

    /// [`Self::other_sites`] of every citation of the project.
    pub(super) fn all_other_sites(
        &self,
        findings: &Catalog,
        schema: &Schema,
        frame: Frame<'_>,
    ) -> Vec<Citation> {
        findings
            .citations
            .iter()
            .flat_map(|cite| self.other_sites(cite, schema, frame))
            .collect()
    }

    /// The other sites of a citation no declaration owns: its kind is the place's, so it is judged
    /// at each spelling (§FS-scan.identity.findings.place, §FS-config.3.9.2). A citation a
    /// declaration owns has that declaration's kind at every name, and is one finding.
    pub(super) fn other_sites(
        &self,
        cite: &Citation,
        schema: &Schema,
        frame: Frame<'_>,
    ) -> Vec<Citation> {
        if !self.active || cite.enclosing_declaration.is_some() {
            return Vec::new();
        }
        let names = self.reached(&cite.file);
        names
            .into_iter()
            .filter(|name| *name != cite.file)
            .map(|name| {
                let mut site = cite.clone();
                site.source_kind = file_home_kind(&name, schema, frame)
                    .unwrap_or_else(|| schema.complement_name().to_string());
                site.file = name;
                site
            })
            .collect()
    }
}

/// The findings of the two codes the walk's verdicts earn (§FS-scan.checks.reading-conflict,
/// §FS-scan.checks.reading-moved).
pub(super) fn check_reading_names(
    catalog: &Catalog,
    schema: &Schema,
    frame: Frame<'_>,
    report: &mut CheckReport,
) {
    if catalog.reading_conflicts.is_empty() && catalog.reading_moves.is_empty() {
        return;
    }
    // §FS-scan.identity.reading.plain: where the plain run reaches no name, the line is the tier's.
    let plain = WalkBounds::plain(schema, frame.root());
    let tiered = |finding: Diagnostic, names: &[PathBuf]| {
        if names.iter().any(|name| plain.contains(name)) {
            finding
        } else {
            tag_out_of_scope_for_version(finding, frame.version)
        }
    };
    for conflict in &catalog.reading_conflicts {
        let finding = conflict_finding(conflict, frame);
        report.errors.push(tiered(finding, &conflict.names));
    }
    for moved in &catalog.reading_moves {
        let finding = moved_finding(moved);
        report
            .warnings
            .push(tiered(finding, std::slice::from_ref(&moved.file)));
    }
}

/// The refusal, located at the name listed first and naming every name once
/// (§FS-scan.checks.reading-conflict).
fn conflict_finding(conflict: &ReadingConflict, frame: Frame<'_>) -> Diagnostic {
    let names: Vec<String> = conflict
        .names
        .iter()
        .map(|name| frame.display_path(name))
        .collect();
    let readings: Vec<String> = conflict
        .readings
        .iter()
        .map(|reading| format!("as {reading}"))
        .collect();
    let ways = match readings.len() {
        2 => "two".to_string(),
        3 => "three".to_string(),
        count => count.to_string(),
    };
    Diagnostic {
        code: Code::READING_CONFLICT,
        path: conflict.names.first().cloned(),
        line: Some(1),
        column: None,
        message: format!(
            "{} are one file that reads {ways} ways, {}; {}",
            ReadingConflict::joined(&names),
            ReadingConflict::joined(&readings),
            remedies::reading_conflict(),
        ),
        sites: conflict
            .names
            .iter()
            .map(|name| Site {
                path: name.clone(),
                line: 1,
            })
            .collect(),
        authority: Vec::new(),
    }
}

/// The one warning a link whose reading moved earns (§FS-scan.checks.reading-moved).
fn moved_finding(moved: &ReadingMove) -> Diagnostic {
    Diagnostic {
        code: Code::READING_MOVED,
        path: Some(moved.file.clone()),
        line: Some(1),
        column: None,
        message: format!(
            "read as {}, as its target's own name reads, not as {} (since 0.22.0); {}",
            moved.now,
            moved.was,
            remedies::reading_moved(),
        ),
        sites: Vec::new(),
        authority: Vec::new(),
    }
}

/// The one pass where the report is finished (§FS-scan.identity.findings.content, §AR-scanner.1.8),
/// after every rule and before the path narrows, so a rule's finding is named like any other and
/// a site it gains here keeps it in scope (§FS-check.1.3.6.2): a finding about what the file holds
/// becomes one line at the name listed first, the others in a closing clause and all of them in
/// `sites`. A finding about where the file sits, and one judged from its directory, gets neither
/// (§FS-scan.identity.findings.place, §FS-scan.identity.findings.judged).
pub(crate) fn name_every_spelling(report: &mut CheckReport, schema: &Schema, frame: Frame<'_>) {
    let names = PlaceNames::new(schema, frame);
    if !names.active {
        return;
    }
    for channel in [
        &mut report.errors,
        &mut report.warnings,
        &mut report.suggestions,
    ] {
        finish_channel(channel, &names, frame);
    }
}

fn finish_channel(channel: &mut Vec<Diagnostic>, names: &PlaceNames<'_>, frame: Frame<'_>) {
    // A value source's declaration is copied per spelling (§FS-values.2.3), so one content finding
    // can arrive once per copy: one file, one line, one line of report.
    let mut seen = BTreeSet::new();
    channel.retain_mut(|finding| {
        if Side::of_finding(finding) != Some(Side::Content) {
            return true;
        }
        let (Some(path), Some(line)) = (finding.path.clone(), finding.line) else {
            return true;
        };
        let first = names.first(&path);
        let reached = names.reached(&path);
        if !reached.is_empty() {
            let key = (finding.code, first.clone(), line, finding.message.clone());
            if !seen.insert(key) {
                return false;
            }
        }
        finding.path = Some(first);
        name_every_site(finding, &reached, line, names, frame);
        true
    });
}

/// The clause names the other spellings of the line the finding stands at, except one its rule
/// already has as a site of that line, and `sites` holds every spelling of every line it speaks of,
/// once. A finding's own sites are kept: they are other lines of the same text, or other files
/// (§FS-check.1.3.6.2). The rule wrote its message from first names ([`PlaceNames::settle`]), so
/// this pass reads no message and edits none of it but the clause it adds.
fn name_every_site(
    finding: &mut Diagnostic,
    reached: &[PathBuf],
    line: usize,
    names: &PlaceNames<'_>,
    frame: Frame<'_>,
) {
    let own = std::mem::take(&mut finding.sites);
    let others: Vec<String> = reached
        .iter()
        .skip(1)
        .filter(|name| {
            !own.iter()
                .any(|site| site.line == line && site.path == **name)
        })
        .map(|name| format!("{}:{line}", frame.display_path(name)))
        .collect();
    if !others.is_empty() {
        finding.message = format!("{} (also at {})", finding.message, others.join(", "));
    }
    let mut sites: Vec<Site> = reached
        .iter()
        .map(|path| Site {
            path: path.clone(),
            line,
        })
        .collect();
    for site in own {
        let line = site.line;
        let spellings = names.reached(&site.path);
        for path in spellings.into_iter().chain([names.first(&site.path)]) {
            if !sites.iter().any(|s| s.path == path && s.line == line) {
                sites.push(Site { path, line });
            }
        }
    }
    finding.sites = sites;
}
