//! The coordinate globs of one run (§FS-coordinate-globs): read before any config is
//! discovered, then checked against the loaded projects and compiled once per project
//! they read (§FS-coordinate-globs.scope), with the note for each pattern no kind in
//! scope can start (§FS-coordinate-globs.errors.note). `list`, `list --size` and the
//! generated batch all select through it, so one pattern selects one set on each.

use anyhow::Result;

use crate::config::Config;
use crate::grammar::{
    CoordinateGlob, GlobMatcher, GlobOperator, GlobQualifier, GlobRefusal,
    literal_after_kind_placeholder,
};
use crate::model::OperationDiagnostic;
use crate::resolver::WorkspaceContext;

/// A refused pattern as the query failure every surface reports
/// (§FS-coordinate-globs.bindings.refusals).
fn refusal_error(refusal: GlobRefusal) -> anyhow::Error {
    let mut diagnostic =
        OperationDiagnostic::new("query", "invalid-coordinate-glob", refusal.to_string());
    diagnostic.column = Some(refusal.column);
    diagnostic.into()
}

/// Read every pattern, refusing the first the grammar alone refuses, in command-line
/// order (§FS-coordinate-globs.errors.shape).
pub(crate) fn read_globs(patterns: &[String]) -> Result<Vec<CoordinateGlob>> {
    patterns
        .iter()
        .map(|pattern| CoordinateGlob::parse(pattern).map_err(refusal_error))
        .collect()
}

/// The refusals a frontend makes at launch, before it reads anything else
/// (§FS-coordinate-globs.errors.shape): the same reading every query makes first.
#[doc(hidden)]
pub fn check_coordinate_globs(patterns: &[String]) -> Result<()> {
    read_globs(patterns).map(drop)
}

/// Whether a scan path a frontend could not find was more likely a coordinate glob:
/// it holds an operator and is a valid pattern (§FS-coordinate-globs.errors.hint).
#[doc(hidden)]
pub fn reads_as_coordinate_glob(text: &str) -> bool {
    text.chars().any(|ch| GlobOperator::of(ch).is_some()) && CoordinateGlob::parse(text).is_ok()
}

/// The patterns a run was given, each compiled for every project it reads.
pub(crate) struct GlobSelection<'g> {
    /// For each loaded project, by index, the matchers of the patterns reading it.
    matchers: Vec<Vec<GlobMatcher<'g>>>,
    notes: Vec<String>,
}

impl<'g> GlobSelection<'g> {
    /// Check each pattern in command-line order against the loaded projects — its
    /// marker, its qualifier, then the separator of every project it reads — and
    /// compile it for each (§FS-coordinate-globs.errors.shape).
    pub(crate) fn new(context: &WorkspaceContext, globs: &'g [CoordinateGlob]) -> Result<Self> {
        let mut matchers: Vec<Vec<GlobMatcher<'g>>> =
            context.projects.iter().map(|_| Vec::new()).collect();
        let mut notes = Vec::new();
        for glob in globs {
            glob.check_marker(&context.render_config().marker)
                .map_err(refusal_error)?;
            let scope = projects_read(context, glob).map_err(refusal_error)?;
            for &index in &scope {
                let config = &context.projects[index].config;
                glob.check_marker(&config.marker).map_err(refusal_error)?;
                glob.check_separator(&config.section_separator)
                    .map_err(refusal_error)?;
            }
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
        GlobQualifier::Alias(alias) => context
            .projects
            .iter()
            .position(|project| context.workspace_loaded && &project.alias == alias)
            .map(|index| vec![index])
            .ok_or_else(|| glob.unknown_alias(alias, &context.aliases())),
    }
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
            let Some(head) = kind_head(config, &kind.kind, kind.format.as_deref()) else {
                return None;
            };
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
