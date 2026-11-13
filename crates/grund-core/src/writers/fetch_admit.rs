//! Admission (§FS-fetch.admission): fetch installs only what the selected project's own scan
//! reads back as exactly what was asked for. The output is read alone as the scan reads one file
//! at the target's path (clauses 1 and 4), the candidate is built where the scan put the
//! requested declaration (§FS-fetch.4, §FS-fetch.5), and the home as it would read after, the
//! one scan's reading with the candidate read in place of the target (§FS-fetch.6.1), is held to
//! the tree as it read before (clauses 2, 3 and 5, in `fetch_readback.rs`). Fetch keeps no
//! heading test, line loop or depth limit of its own: every answer here is the scan's.

use std::path::PathBuf;

use super::atomic_install::Holds;
use super::fetch::{FetchFailure, fetch_operational};
use super::fetch_findings::no_new_findings;
use super::fetch_readback::{Readback, keep_others, settled};
use super::fetch_splice::{Owned, insert, replace};
use super::fetch_target::{Before, Home, Request, Site, Target};
use crate::grammar::{open_block, render_catalog_id};
use crate::model::{Catalog, FileRecord, scanned_path_key};
use crate::scanner::{scan_file_again, scan_unwalked_file};

/// What admission lets fetch install: the bytes at the target, against what the scan read there.
pub(super) struct Admitted {
    /// The target as the scan names it.
    pub(super) path: PathBuf,
    pub(super) install_at: PathBuf,
    pub(super) bytes: String,
    before: Option<String>,
}

impl Admitted {
    /// §FS-fetch.6.1: the target must still hold what the scan read, or still be absent.
    pub(super) fn expected(&self) -> Holds<'_> {
        match &self.before {
            Some(text) => Holds::Bytes(text.as_bytes()),
            None => Holds::Nothing,
        }
    }
}

/// §FS-fetch.admission: the clauses in order, the first that fails refusing the fetch. Placement
/// was settled from the scan before the integration ran; its refusal speaks once the output is
/// known to be one declaration, as it always has.
pub(super) fn admit(
    request: &Request<'_>,
    before: &Before,
    placed: Result<Target, (FetchFailure, Option<anyhow::Error>)>,
    output: &str,
    diagnostic: &mut Option<anyhow::Error>,
) -> Result<Admitted, FetchFailure> {
    read_output_alone(request, output)?;
    let target = placed.map_err(|(failure, source)| {
        *diagnostic = source;
        failure
    })?;
    let original = target.before.as_deref().unwrap_or_default();
    let (candidate, old, start) = match &target.site {
        Site::Replace(owned) => (replace(original, owned, output), Some(*owned), owned.start),
        Site::Insert(at, line) => {
            let (candidate, start) = insert(original, *at, output);
            (candidate, Some(Owned::empty(*at, *line)), start)
        }
        Site::Whole => {
            let old = before.record(&target.path).and_then(|record| {
                before
                    .catalog
                    .declarations
                    .get(request.id)
                    .into_iter()
                    .flatten()
                    .find(|decl| scanned_path_key(&decl.file) == scanned_path_key(&target.path))
                    .map(|decl| Owned::of(record, decl.line, decl.body_end))
            });
            (output.to_string(), old, 0)
        }
    };
    let config = request.config;
    // §FS-fetch.6.1: the candidate in front of the target, and a file only a stub names at the
    // bytes the scan read there, so nothing the integration wrote enters the reading.
    let view = config.with_buffers(
        config.run().buffers.in_front(
            before
                .unwalked()
                .chain([(target.path.clone(), candidate.clone())]),
        ),
    );
    // §FS-fetch.admission, §FS-fetch.6.1: the tree as the scan read it, with the candidate read
    // in place of the target and no other file read again.
    let after = scan_file_again(
        &before.catalog,
        &target.path,
        view.schema(),
        view.frame(),
        &before.targets,
    )
    .map_err(|err| {
        fetch_operational(format!(
            "cannot discover declarations for {}: {err:#}",
            request.local
        ))
    })?;
    let first = candidate[..start].matches('\n').count() + 1;
    let output_lines = output.trim_end().matches('\n').count() + 1;
    let readback = Readback {
        request,
        before,
        after: &after,
        view: &view,
        path: &target.path,
        old,
        new: Owned {
            first,
            last: first + output_lines - 1,
            start,
            end: start + output.len(),
        },
        candidate: &candidate,
        whole: matches!(target.site, Site::Whole),
    };
    keep_others(&readback)?;
    no_new_findings(&readback)?;
    settled(&readback, output)?;
    Ok(Admitted {
        path: target.path,
        install_at: target.install_at,
        bytes: candidate,
        before: target.before,
    })
}

/// §FS-fetch.admission.1, §FS-fetch.admission.4: the output alone, as the scan reads one file at
/// the target's path.
fn read_output_alone(request: &Request<'_>, output: &str) -> Result<(), FetchFailure> {
    let config = request.config;
    let at = match &request.home {
        Home::File(file) => file.clone(),
        Home::Folder(folder) => folder.join(format!("{}.md", request.local)),
    };
    // §DF-one-reading-answer.2.6: a name no admitted language claims is read as 0.22.0 read it.
    let view = config
        .with_buffers(
            config
                .run()
                .buffers
                .in_front([(at.clone(), output.to_string())]),
        )
        .with_unclaimed_read_by_name();
    let alone = scan_unwalked_file(&at, view.schema(), view.frame(), &[]).map_err(|err| {
        fetch_operational(format!(
            "cannot discover declarations for {}: {err:#}",
            request.local
        ))
    })?;
    let Some(record) = alone.files.values().next() else {
        return Err(fetch_operational(
            "fetch output must contain exactly one Markdown declaration",
        ));
    };
    one_declaration(request, &alone, record)?;
    // §FS-fetch.admission.4: a block left open swallows every line after it in the home.
    if let Some(open) = open_block(output) {
        return Err(fetch_operational(format!(
            "fetch output line {} opens a {} it does not close; close it, and fetch again",
            open.line,
            if open.raw {
                "raw-text block"
            } else {
                "fenced block"
            }
        )));
    }
    Ok(())
}

/// One heading the scan read in the output: a declaration or a near miss, which it records as
/// declaring, or any other heading.
struct Read<'a> {
    line: usize,
    level: usize,
    /// The ID token as written, where the scan reads the line as declaring one.
    declares: Option<String>,
    /// The scan's ID, where it parsed one.
    id: Option<&'a crate::model::Id>,
    atx: bool,
}

/// §FS-fetch.admission.1: the requested declaration is there once, on the first line at the
/// home's level, and its body takes in all of the output.
fn one_declaration(
    request: &Request<'_>,
    alone: &Catalog,
    record: &FileRecord,
) -> Result<(), FetchFailure> {
    let native = request.home.level();
    let first = (1..=record.total_lines).find(|line| {
        record
            .line(*line)
            .is_some_and(|text| !text.trim().is_empty())
    });
    let Some(first) = first else {
        return Err(fetch_operational(
            "fetch output must contain exactly one Markdown declaration",
        ));
    };
    let reads = headings_read(&request.config.grammar, alone, record);
    let declaration = reads
        .iter()
        .find(|read| read.line == first && read.id.is_some());
    let Some(declaration) = declaration else {
        return Err(fetch_operational(
            "fetch output must begin with one Markdown declaration",
        ));
    };
    if first > 1 {
        return Err(exactly_one());
    }
    if declaration.level != native {
        return Err(fetch_operational(format!(
            "fetch output heading depth {} is invalid for a depth-{native} declaration",
            declaration.level
        )));
    }
    if declaration.id != Some(request.id) {
        return Err(fetch_operational(format!(
            "fetch output declares `{}` instead of requested `{}`",
            declaration.declares.as_deref().unwrap_or_default(),
            request.local
        )));
    }
    for read in reads.iter().filter(|read| read.line > first) {
        match &read.declares {
            // §FS-fetch.3: a second declaration at the home's level, or above it.
            Some(_) if read.level > 0 && read.level <= native => return Err(exactly_one()),
            // §FS-fetch.3.1: anything deeper the scan reads as declaring an ID, a comment line too.
            Some(nested) => {
                return Err(fetch_operational(format!(
                    "fetch output nests declaration `{nested}` inside requested `{}`; fetch \
                     output must hold only the requested declaration",
                    request.local
                )));
            }
            // A heading that ends the body before the output does.
            None if read.atx && read.level <= native => {
                return Err(fetch_operational(format!(
                    "fetch output line {}: heading `{}` ends {} before the output does; fence \
                     it, make it a bold label, or remove it",
                    read.line,
                    record.line(read.line).unwrap_or_default().trim_end(),
                    request.local
                )));
            }
            None => {}
        }
    }
    Ok(())
}

fn exactly_one() -> FetchFailure {
    fetch_operational("fetch output must contain exactly one declaration and no leading content")
}

/// Every heading the scan read in the output, in line order, with what it declares.
fn headings_read<'a>(
    grammar: &crate::grammar::Grammar,
    alone: &'a Catalog,
    record: &FileRecord,
) -> Vec<Read<'a>> {
    let level_at = |line: usize| {
        record.heading_at(line).map_or_else(
            || {
                let text = record.line(line).unwrap_or_default().trim_start();
                text.bytes().take_while(|byte| *byte == b'#').count()
            },
            |heading| heading.level,
        )
    };
    let mut reads: Vec<Read<'a>> = alone
        .declarations
        .values()
        .flatten()
        .map(|decl| Read {
            line: decl.line,
            // §FS-fetch.3.1: a declaration the scan read on a comment line sits at no level.
            level: if record
                .line(decl.line)
                .is_some_and(|text| text.trim_start().starts_with('#'))
            {
                decl.heading_level
            } else {
                0
            },
            declares: Some(
                decl.authored_id
                    .clone()
                    .unwrap_or_else(|| render_catalog_id(grammar, &alone.declarations, &decl.id)),
            ),
            id: Some(&decl.id),
            atx: true,
        })
        .chain(alone.near_miss_headings.iter().map(|near| Read {
            line: near.line,
            level: level_at(near.line),
            declares: Some(near.text.clone()),
            id: None,
            atx: true,
        }))
        .collect();
    for heading in &record.headings {
        if !reads.iter().any(|read| read.line == heading.line) {
            reads.push(Read {
                line: heading.line,
                level: heading.level,
                declares: None,
                id: None,
                atx: heading.atx,
            });
        }
    }
    reads.sort_by_key(|read| read.line);
    reads
}
