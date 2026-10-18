//! The coordinate globs of one run (§FS-coordinate-globs): read before any config is
//! discovered, then checked against the loaded projects and compiled once per project
//! they read (§FS-coordinate-globs.scope), with the note for each pattern no kind in
//! scope can start (§FS-coordinate-globs.errors.note). `list`, `list --size` and the
//! generated batch all select through it, so one pattern selects one set on each, and
//! `refs --glob` and `cover --glob` select the citation sites whose cited coordinate it
//! matches (§FS-coordinate-globs.matching.citations).

use anyhow::Result;

use crate::config::{Config, DEFAULT_MARKER, DEFAULT_SECTION_SEPARATOR};
use crate::grammar::{
    CoordinateGlob, GlobMatcher, GlobOperator, GlobQualifier, GlobRefusal, GlobStage,
    literal_after_kind_placeholder, render_id,
};
use crate::model::{Citation, OperationDiagnostic};
use crate::resolver::WorkspaceContext;

/// A refused pattern as the query failure every surface reports
/// (§FS-coordinate-globs.bindings.refusals).
fn refusal_error(refusal: GlobRefusal) -> anyhow::Error {
    let mut diagnostic =
        OperationDiagnostic::new("query", "invalid-coordinate-glob", refusal.to_string());
    diagnostic.column = Some(refusal.column);
    diagnostic.into()
}

/// The launch stage: no config is read before discovery, so what a refusal there offers
/// is read as the `.`-separated reading of a project with the built-in marker
/// (§FS-coordinate-globs.errors.shape, §FS-config.3).
const LAUNCH: GlobStage<'static> = GlobStage::new(&[DEFAULT_MARKER], &[DEFAULT_SECTION_SEPARATOR]);

/// Read every pattern, refusing the first the grammar alone refuses, in command-line
/// order (§FS-coordinate-globs.errors.shape).
pub(crate) fn read_globs(patterns: &[String]) -> Result<Vec<CoordinateGlob>> {
    patterns
        .iter()
        .map(|pattern| CoordinateGlob::parse(pattern, LAUNCH).map_err(refusal_error))
        .collect()
}

/// The refusals a frontend makes at launch, before it reads anything else
/// (§FS-coordinate-globs.errors.shape): the same reading every query makes first.
#[doc(hidden)]
pub fn check_coordinate_globs(patterns: &[String]) -> Result<()> {
    read_globs(patterns).map(drop)
}

/// Whether a scan path a frontend could not find was more likely a coordinate glob:
/// it holds an operator and is a valid pattern (§FS-coordinate-globs.errors.hint). No
/// config was read for a path that does not exist, so it is read as the launch stage
/// reads an offer, under the built-in separator and marker (§FS-config.3.2.2).
#[doc(hidden)]
pub fn reads_as_coordinate_glob(text: &str) -> bool {
    text.chars().any(|ch| GlobOperator::of(ch).is_some()) && LAUNCH.accepts(text)
}

/// Whether a scan path `refs --glob` or `refs --from` could not find reads as an ID
/// typed where the path goes: it holds no operator and is a valid pattern, read as the
/// launch stage reads one, since no config was read for it (§FS-refs.1.3).
#[doc(hidden)]
pub fn reads_as_coordinate(text: &str) -> bool {
    !text.chars().any(|ch| GlobOperator::of(ch).is_some()) && LAUNCH.accepts(text)
}

/// The pattern a `refs` refusal of `--section` or `--descendants` beside one `--glob`
/// offers (§FS-refs.1.2, §FS-errors.2.5): `pattern` with `.<section>` appended, every
/// operator and escape in the section escaped, then `{,.**}` when `descendants` is
/// asked; `None` where the launch stage would refuse the result.
#[doc(hidden)]
pub fn glob_beside_section(
    pattern: &str,
    section: Option<&str>,
    descendants: bool,
) -> Option<String> {
    let mut offer = pattern.to_string();
    if let Some(section) = section {
        offer.push('.');
        for ch in section.chars() {
            if ch == '\\' || GlobOperator::of(ch).is_some() {
                offer.push('\\');
            }
            offer.push(ch);
        }
    }
    if descendants {
        offer.push_str("{,.**}");
    }
    LAUNCH.accepts(&offer).then_some(offer)
}

/// The patterns a run was given, each compiled for every project it reads.
pub(crate) struct GlobSelection<'g> {
    /// For each loaded project, by index, the matchers of the patterns reading it.
    matchers: Vec<Vec<GlobMatcher<'g>>>,
    notes: Vec<String>,
}

impl<'g> GlobSelection<'g> {
    /// Check each pattern in command-line order against the loaded projects, leftmost
    /// refusal first — its qualifier at column 1, then the stage of the projects it
    /// reads, their markers and separators, which an offer passes too — and compile it
    /// for each (§FS-coordinate-globs.errors.shape).
    pub(crate) fn new(context: &WorkspaceContext, globs: &'g [CoordinateGlob]) -> Result<Self> {
        let mut matchers: Vec<Vec<GlobMatcher<'g>>> =
            context.projects.iter().map(|_| Vec::new()).collect();
        let mut notes = Vec::new();
        for glob in globs {
            let scope = projects_read(context, glob).map_err(refusal_error)?;
            let configs: Vec<_> = scope
                .iter()
                .map(|&index| &context.projects[index].config)
                .collect();
            let markers: Vec<&str> = std::iter::once(&context.render_config().marker)
                .chain(configs.iter().map(|config| &config.marker))
                .map(String::as_str)
                .collect();
            let separators: Vec<&str> = configs
                .iter()
                .map(|config| config.section_separator.as_str())
                .collect();
            glob.check(GlobStage::new(&markers, &separators))
                .map_err(refusal_error)?;
            for &index in &scope {
                let separator = &context.projects[index].config.section_separator;
                matchers[index].push(glob.matcher(separator));
            }
            notes.extend(note(context, glob, &scope));
        }
        Ok(Self { matchers, notes })
    }

    /// Whether any pattern reading the project at `project` selects the coordinate.
    pub(crate) fn selects(&self, project: usize, id: &str, section: Option<&str>) -> bool {
        self.matchers[project]
            .iter()
            .any(|matcher| matcher.matches(id, section))
    }

    /// Whether any pattern selects what one citation in the project at `citing` cites:
    /// its coordinate in the project it targets, the loaded project its alias names or
    /// else the one it sits in, spelled under that project's grammar, declared or not
    /// (§FS-coordinate-globs.matching.citations, §FS-coordinate-globs.scope.targets). An
    /// alias that names no loaded project leaves the citation matched by no pattern.
    pub(crate) fn selects_citation(
        &self,
        context: &WorkspaceContext,
        citing: usize,
        citation: &Citation,
    ) -> bool {
        let target = match citation.namespace.as_deref() {
            None => Some(citing),
            Some(alias) => loaded_alias(context, alias),
        };
        target.is_some_and(|target| {
            let id = render_id(&context.projects[target].config.grammar, &citation.id);
            self.selects(target, &id, citation.section.as_deref())
        })
    }

    /// Each note's text after `note: `, one per line (§FS-coordinate-globs.bindings.note).
    pub(crate) fn note(&self) -> Option<String> {
        (!self.notes.is_empty()).then(|| self.notes.join("\n"))
    }
}

/// The loaded projects a pattern reads (§FS-coordinate-globs.scope): the current one,
/// every one, or the one with its alias. An excluded root leaves an unqualified pattern
/// none.
fn projects_read(
    context: &WorkspaceContext,
    glob: &CoordinateGlob,
) -> std::result::Result<Vec<usize>, GlobRefusal> {
    match glob.qualifier() {
        GlobQualifier::Current => Ok(context.current.into_iter().collect()),
        GlobQualifier::Every => Ok((0..context.projects.len()).collect()),
        GlobQualifier::Alias(alias) => loaded_alias(context, alias)
            .map(|index| vec![index])
            .ok_or_else(|| glob.unknown_alias(alias, &context.aliases())),
    }
}

/// The index of the loaded project an alias names; outside a workspace, none does.
fn loaded_alias(context: &WorkspaceContext, alias: &str) -> Option<usize> {
    context
        .projects
        .iter()
        .position(|project| context.workspace_loaded && project.alias == alias)
}

/// The ID head a kind's IDs begin with — the kind and the literal its format writes
/// after `{kind}` — or `None` for a format that does not begin with `{kind}`, whose IDs
/// can begin with anything.
fn kind_head(config: &Config, kind: &str, format: Option<&str>) -> Option<String> {
    let format = format.unwrap_or(&config.id_format);
    format.starts_with("{kind}").then(|| {
        format!(
            "{kind}{}",
            literal_after_kind_placeholder(format).unwrap_or("")
        )
    })
}

/// The note for a valid pattern no configured citable kind in scope can start, with the
/// one kind it names with ASCII case ignored as its suggestion (§FS-coordinate-globs.errors.note).
fn note(context: &WorkspaceContext, glob: &CoordinateGlob, scope: &[usize]) -> Option<String> {
    let separator = |index: usize| context.projects[index].config.section_separator.as_str();
    let prefix = glob.literal_prefix(scope.first().map_or(".", |&index| separator(index)));
    let overlaps = |head: &str, fold: bool| {
        let (head, prefix) = if fold {
            (head.to_ascii_lowercase(), prefix.to_ascii_lowercase())
        } else {
            (head.to_string(), prefix.clone())
        };
        head.starts_with(&prefix) || prefix.starts_with(&head)
    };
    let mut folded: Vec<String> = Vec::new();
    for &index in scope {
        let config = &context.projects[index].config;
        for kind in config.kinds.iter().filter(|kind| kind.citable) {
            let head = kind_head(config, &kind.kind, kind.format.as_deref())?;
            if overlaps(&head, false) {
                return None;
            }
            if overlaps(&head, true) && !folded.contains(&head) {
                folded.push(head);
            }
        }
    }
    let mut text = format!("no configured kind in scope starts with \"{prefix}\"");
    if let [head] = folded.as_slice() {
        let spelling = head.get(..head.len().min(prefix.len()));
        if let Some(suggestion) = spelling.and_then(|spelling| glob.respelled(spelling)) {
            text.push_str(&format!("; did you mean \"{suggestion}\"?"));
        }
    }
    Some(text)
}
