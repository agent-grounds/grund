//! The published `cover --lines` contract (§FS-cover.6): which declaration and
//! section own each line of a requested range of one file, read from the scan's
//! own ownership lookup and rendered with no output format chosen and no exit
//! code mapped (§AR-bindings.2).

use anyhow::{Result, bail};
use std::path::PathBuf;

use crate::config::display_path;
use crate::grammar::render_id;
use crate::model::{Finding, RangeOwnership, paths_same_location};
use crate::resolver::load_narrowable_workspace_context;
use crate::scanner::ApiScanError;

use super::cover::cover_scan_errors;
use super::report::context_run_warnings;

/// A `cover --lines` request: one file and the ranges asked of it, each as the
/// caller wrote it — `<N>` or `<N>-<M>` — so a refusal can echo it
/// (§FS-cover.6.1, §FS-cover.6.4).
#[derive(Clone, Debug, Default)]
pub struct CoverLinesOpts {
    pub path: PathBuf,
    pub path_provided: bool,
    pub lines: Vec<String>,
}

/// One requested range's answer (§FS-cover.6.3, §FS-output-shapes.5.3).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverLines {
    /// The alias of the project the file belongs to, exactly when a
    /// §FS-cover.3.2 record of the same run would carry one; `None` otherwise.
    pub project: Option<String>,
    pub path: String,
    pub start: usize,
    pub end: usize,
    /// The owner runs, in line order; empty when no line of the range is owned.
    pub owners: Vec<CoverLineOwner>,
}

/// A maximal run of lines owned by one declaration, clipped to the range
/// (§FS-cover.6.3).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverLineOwner {
    /// Bare, rendered as `enclosing_declaration` is (§FS-cover.6.3).
    pub declaration: String,
    pub start: usize,
    pub end: usize,
    /// The section runs that partition this owner run, in line order.
    pub sections: Vec<CoverLineSection>,
}

/// A maximal run under one innermost section; `None` is the lead or lines no
/// accepted section contains (§FS-cover.6.3).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverLineSection {
    pub section: Option<String>,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CoverLinesOutput {
    pub output_format: String,
    /// One record per requested range, in the order asked; none at all when the
    /// scan does not read the file (§FS-cover.6.3).
    pub records: Vec<CoverLines>,
    pub scan_errors: Vec<ApiScanError>,
    /// The run's warning channel, as on [`super::CoverOutput::warnings`]
    /// (§FS-distribution.3.1).
    pub warnings: Vec<Finding>,
}

/// Parse one `--lines` value, `<N>` or `<N>-<M>` in decimal digits, into an
/// inclusive 1-based range, or the §FS-cover.6.4 message that refuses it
/// (without the `error: ` a frontend adds). A number too large to hold is kept
/// as the largest one, so the range is refused as ending past the file rather
/// than called malformed.
pub fn parse_cover_line_range(text: &str) -> std::result::Result<(usize, usize), String> {
    let number = |part: &str| {
        (!part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
            .then(|| part.parse::<usize>().unwrap_or(usize::MAX))
    };
    let parsed = match text.split_once('-') {
        Some((start, end)) => number(start).zip(number(end)),
        None => number(text).map(|line| (line, line)),
    };
    let Some((start, end)) = parsed else {
        return Err(format!("--lines takes <N> or <N>-<M>, got `{text}`"));
    };
    if start == 0 || end == 0 {
        return Err(format!("--lines `{text}`: lines are numbered from 1"));
    }
    if end < start {
        return Err(format!("--lines `{text}` ends before it starts"));
    }
    Ok((start, end))
}

/// Programmatic `cover --lines`: the owners of each requested range of one
/// file, by the scan's own rules (§FS-cover.6, §AR-scanner.2.4.3,
/// §AR-scanner.2.4.4).
pub fn cover_lines(opts: CoverLinesOpts) -> Result<CoverLinesOutput> {
    cover_lines_with_run_warnings(opts).1
}

/// Additive cautions survive a later refusal (§FS-distribution.3.1).
pub fn cover_lines_with_run_warnings(
    opts: CoverLinesOpts,
) -> (Vec<Finding>, Result<CoverLinesOutput>) {
    let mut cautions = Vec::new();
    let result = cover_lines_run(opts, &mut cautions);
    (cautions, result)
}

fn cover_lines_run(opts: CoverLinesOpts, cautions: &mut Vec<Finding>) -> Result<CoverLinesOutput> {
    // §FS-cover.6.4: every refusal of the request itself comes before the load.
    let ranges = opts
        .lines
        .iter()
        .map(|text| parse_cover_line_range(text))
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(anyhow::Error::msg)?;
    if !opts.path_provided {
        bail!("--lines needs a file path");
    }
    if opts.path.is_dir() {
        bail!(
            "--lines needs a file path, and `{}` is a directory",
            opts.path.display()
        );
    }
    // §FS-cover.6.1: one narrowed scan of the enclosing project, the same load
    // a `cover <file>` run takes, with the ranges riding into the file's pass.
    let context = load_narrowable_workspace_context(&opts.path, opts.path_provided, &ranges)?;
    *cautions = context_run_warnings(&context);
    let mut records = Vec::new();
    for project in &context.projects {
        let owned = project
            .findings
            .line_ownership
            .iter()
            .filter(|owned| paths_same_location(&owned.file, &opts.path));
        for owned in owned {
            // §FS-cover.6.4: the commonest mistake, a line number from the old side
            // of a diff, is refused with the count the scan saw.
            let past_end = opts
                .lines
                .iter()
                .zip(&ranges)
                .find(|(_, (_, end))| *end > owned.total_lines);
            if let Some((text, _)) = past_end {
                bail!(
                    "--lines `{text}` ends past line {}, the last line of `{}`",
                    owned.total_lines,
                    opts.path.display()
                );
            }
            // §FS-cover.6.3: `path` and `project` render as on the §FS-cover.3.2
            // record of the same run.
            let path = if context.workspace_loaded {
                display_path(context.render_config(), &owned.file)
            } else {
                display_path(&project.config, &owned.file)
            };
            let alias = context.workspace_loaded.then(|| project.alias.clone());
            let render = |range: &RangeOwnership| CoverLines {
                project: alias.clone(),
                path: path.clone(),
                start: range.start,
                end: range.end,
                owners: range
                    .owners
                    .iter()
                    .map(|run| CoverLineOwner {
                        declaration: render_id(&project.config.grammar, &run.declaration),
                        start: run.start,
                        end: run.end,
                        sections: run
                            .sections
                            .iter()
                            .map(|section| CoverLineSection {
                                section: section.section.clone(),
                                start: section.start,
                                end: section.end,
                            })
                            .collect(),
                    })
                    .collect(),
            };
            records.extend(owned.ranges.iter().map(render));
        }
    }
    Ok(CoverLinesOutput {
        output_format: context.render_config().output_format.clone(),
        records,
        scan_errors: cover_scan_errors(&context),
        warnings: context_run_warnings(&context),
    })
}

#[cfg(test)]
mod tests {
    use super::parse_cover_line_range;

    /// §FS-cover.6.1: `<N>` is `<N>-<N>`, and the range is inclusive.
    #[test]
    fn a_line_or_a_range_parses() {
        assert_eq!(parse_cover_line_range("17"), Ok((17, 17)));
        assert_eq!(parse_cover_line_range("1-14"), Ok((1, 14)));
        assert_eq!(parse_cover_line_range("7-7"), Ok((7, 7)));
    }

    /// §FS-cover.6.4: each refusal, with the range echoed as written.
    #[test]
    fn a_bad_range_is_refused_with_its_message() {
        for text in ["", "-", "3-", "-3", "3-x", "+3", "3 - 4", "1-2-3", "x"] {
            assert_eq!(
                parse_cover_line_range(text),
                Err(format!("--lines takes <N> or <N>-<M>, got `{text}`"))
            );
        }
        assert_eq!(
            parse_cover_line_range("0-3"),
            Err("--lines `0-3`: lines are numbered from 1".to_string())
        );
        assert_eq!(
            parse_cover_line_range("0"),
            Err("--lines `0`: lines are numbered from 1".to_string())
        );
        assert_eq!(
            parse_cover_line_range("9-7"),
            Err("--lines `9-7` ends before it starts".to_string())
        );
    }

    /// §REQ-never-crashes: a number too large to hold is still a range, refused
    /// later as ending past the file rather than parsed into a panic.
    #[test]
    fn an_overflowing_number_saturates() {
        assert_eq!(
            parse_cover_line_range("5-99999999999999999999999999"),
            Ok((5, usize::MAX))
        );
    }
}
