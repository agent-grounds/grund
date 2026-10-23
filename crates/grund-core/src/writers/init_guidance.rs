//! The typed post-init guidance and the `--scaffold` starter files
//! (§FS-init.2.1–2.2).

use std::path::Path;

use crate::config::{Config, KindConfig, default_kind_folder, guided_kind};
use crate::scanner::CANONICAL_AGENT_ENTRYPOINT;
use crate::sources::path_exists;
use crate::templates::{
    AS_README_TEMPLATE, DA_README_TEMPLATE, DF_README_TEMPLATE, E2E_README_TEMPLATE,
    FS_FILE_TEMPLATE, FS_README_TEMPLATE, GITKEEP_TEMPLATE, GOALS_TEMPLATE, GRUND_DOC_TEMPLATE,
    REQ_README_TEMPLATE, canonical_template_text, render_scaffold_id_shapes,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitNext {
    /// Whether the run laid the starter files (`--scaffold`, §FS-init.1). The
    /// name is published API and stays (§DF-init-templates.2.3).
    pub docs: bool,
    /// Whether the effective config declares any citable kind, the answer
    /// `allocation.is_some()` gives. Where it does not, the guidance teaches the
    /// first kind instead (§FS-init.2.2.2.2).
    pub declares_kinds: bool,
    pub entrypoint: String,
    /// The home `--scaffold` lays the FS starter in, the built-in FS folder where
    /// no FS row has one (§FS-init.2.2.2.3). The guidance names `allocation`.
    pub fs_home: InitFsHome,
    /// The kind and place the guidance names, `None` exactly where the config has
    /// no citable row (§FS-init.2.2.2.3).
    pub allocation: Option<InitAllocation>,
    /// Whether the effective scanner found a readable file before this guidance
    /// was rendered (§FS-init.2.2.2). Both command adapters consume this decision;
    /// neither reconstructs scanner policy from paths.
    pub scan_reads_file: bool,
}

impl InitNext {
    /// Render the shared trailing guidance for the shipped CLI and the deprecated
    /// core command adapter (§FS-init.2.2.2): the block of the kind `allocation`
    /// selected, or, with none, the block that teaches the first kind
    /// (§FS-init.2.2.2.3).
    pub fn render(&self) -> String {
        match &self.allocation {
            Some(allocation) => render_next_block(
                self.docs,
                &self.entrypoint,
                allocation,
                self.scan_reads_file,
            ),
            None => render_first_kind_block(Some(self.entrypoint.as_str())),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InitFsHome {
    File {
        path: String,
        heading_name: &'static str,
        heading_marker: &'static str,
    },
    Folder {
        path: String,
    },
}

impl InitFsHome {
    /// The home's path relative to the target, whether file or folder.
    pub(crate) fn path(&self) -> &str {
        match self {
            InitFsHome::File { path, .. } | InitFsHome::Folder { path } => path,
        }
    }
}

/// The kind and place the guidance names (§FS-init.2.2.2.3): one selection, read
/// by the allocation step and by the refresh test of §FS-init.2.2.2.1.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitAllocation {
    /// The selected kind, the `<KIND>` of `grund id <KIND>`.
    pub kind: String,
    /// The selected row's first file, else its first folder; `None` for a row
    /// with no home.
    pub home: Option<InitAllocationHome>,
    /// Whether `home` exists on disk, resolved against the target; `false` for a
    /// row with no home.
    pub home_exists: bool,
}

/// The selected place, in the shapes [`InitFsHome`] gives one (§FS-init.2.2.2.3).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InitAllocationHome {
    File {
        path: String,
        heading_name: &'static str,
        heading_marker: &'static str,
    },
    Folder {
        path: String,
    },
}

impl InitAllocationHome {
    /// The place's path relative to the target, whether file or folder.
    pub(crate) fn path(&self) -> &str {
        match self {
            InitAllocationHome::File { path, .. } | InitAllocationHome::Folder { path } => path,
        }
    }
}

impl InitAllocation {
    /// Whether the selection still names a home the disk lacks. A refresh that
    /// leaves none to create is complete (§FS-init.2.2.2.1), and only a missing
    /// home earns the configured-kind form a create step (§FS-init.2.2.2.3).
    pub(crate) fn home_missing(&self) -> bool {
        self.home.is_some() && !self.home_exists
    }

    /// Whether the selection gets the FS form: FS with a home (§FS-init.2.2.2.3).
    fn is_fs_with_home(&self) -> bool {
        self.kind == "FS" && self.home.is_some()
    }
}

/// Select the kind and place the guidance names (§FS-init.2.2.2.3): the `FS` row
/// when it is citable and has a home, else the first citable row with a home,
/// else the first citable row, in configuration order; `None` where no row is
/// citable (§FS-init.2.2.2.2). Whether the place exists is read from `target`
/// once, for the create step and the refresh test alike (§FS-init.2.2.2.1).
pub(crate) fn init_allocation(config: &Config, target: &Path) -> Option<InitAllocation> {
    let row = guided_kind(&config.kinds)?;
    let home = row_home(row);
    let home_exists = home
        .as_ref()
        .is_some_and(|home| path_exists(target.join(home.path())));
    Some(InitAllocation {
        kind: row.kind.clone(),
        home,
        home_exists,
    })
}

/// A row's one place: its first `file`, else its first `folder`, each in written
/// order, and never another because that one exists (§FS-init.2.2.2.3). A
/// scaffold seeds one starter at the same place (§FS-init.2.1).
fn row_home(row: &KindConfig) -> Option<InitAllocationHome> {
    if let Some(file) = &row.file {
        let (heading_name, heading_marker) = if file == "docs/grund.md" {
            ("H1", "#")
        } else {
            ("H2", "##")
        };
        return Some(InitAllocationHome::File {
            path: file.clone(),
            heading_name,
            heading_marker,
        });
    }
    row.folder
        .as_ref()
        .map(|folder| InitAllocationHome::Folder {
            path: folder.clone(),
        })
}

/// The `next:` block of a project that declares no kinds (§FS-init.2.2.2.2):
/// there is no FS home to scaffold or allocate in, so it teaches the first kind.
fn render_first_kind_block(entrypoint: Option<&str>) -> String {
    let entrypoint = entrypoint.unwrap_or(CANONICAL_AGENT_ENTRYPOINT);
    format!(
        "\nnext:\n  \
         1. declare a first kind in grund.toml, then run `grund init` again so {entrypoint} lists it\n  \
         2. run `grund check`\n  \
         3. allocate an ID:  ID=$(grund id <KIND> \"…\")  then add it to that kind's home\n\
         see {entrypoint} for the full workflow.\n"
    )
}

/// The check step of the FS form and of every `--scaffold` form: a fresh scaffold
/// checks with the expected "nothing recognized" warning, not a clean verdict
/// (§FS-init.2.2.2).
const CHECK_STEP: &str = "run `grund check` — on a fresh scaffold it warns \"nothing recognized\" \
     until the first declaration; this is expected (exit 0), no repair needed";

/// The configured-kind form's check step without `--scaffold`, which neither laid
/// nor offered a scaffold for "on a fresh scaffold" to refer to (§FS-init.2.2.2.3).
const PLAIN_CHECK_STEP: &str = "run `grund check`";

/// The trailing `next:` guidance block (§FS-init.2.2.2) for the selected kind and
/// place: the FS form for FS with a home, the configured-kind form for every other
/// selection (§FS-init.2.2.2.3). Suppressed by the caller on a refresh of a complete
/// setup (§FS-init.2.2.2.1), or on a `--scaffold` run whose every reported path is
/// `exists ` (§FS-init.2.2.2) — then there is no next step to teach.
fn render_next_block(
    docs: bool,
    entrypoint: &str,
    allocation: &InitAllocation,
    scan_reads_file: bool,
) -> String {
    let InitAllocation { kind, home, .. } = allocation;
    let statement = "<one-line statement of the behavior>";
    let (destination, heading) = match home {
        Some(InitAllocationHome::File {
            path,
            heading_name,
            heading_marker,
        }) => (
            format!("add it to {path}"),
            format!("{heading_name}: `{heading_marker} <ID>: {statement}`"),
        ),
        Some(InitAllocationHome::Folder { path }) => (
            format!("add it under {path}"),
            format!("H1: `# <ID>: {statement}`"),
        ),
        None => (
            "declare it in a source doc-comment".to_string(),
            format!("doc-comment: `<ID>: {statement}`"),
        ),
    };
    let allocate = format!("allocate an ID:  ID=$(grund id {kind} \"…\")  then {destination}");
    let mut steps = Vec::new();
    if docs {
        // §FS-init.2.2.2: the heading line and the citation step print only here.
        steps.push(CHECK_STEP.to_string());
        steps.push(format!("{allocate}\n     ({heading})"));
        steps.push(
            "cite it as §<ID> from the docs and e2e tests that depend on it, then `grund check` again"
                .to_string(),
        );
    } else {
        // §FS-init.2.2.2: the clause rides on the step that makes a home.
        let until_then = if scan_reads_file {
            ""
        } else {
            " — until then `grund check` has nothing to scan"
        };
        let fs_form = allocation.is_fs_with_home();
        match home {
            Some(home) if fs_form => steps.push(format!(
                "re-run with --scaffold to scaffold the FS home ({}), docs/, and tests/ (or create them yourself){until_then}",
                home.path()
            )),
            // §FS-init.2.2.2.3: never `--scaffold`, and only while the home is missing.
            Some(home) if allocation.home_missing() => {
                steps.push(format!("create the {kind} home ({}){until_then}", home.path()));
            }
            _ => {}
        }
        steps.push(
            if fs_form {
                CHECK_STEP
            } else {
                PLAIN_CHECK_STEP
            }
            .to_string(),
        );
        steps.push(allocate);
    }
    let mut output = "\nnext:\n".to_string();
    for (index, step) in steps.iter().enumerate() {
        output.push_str(&format!("  {}. {step}\n", index + 1));
    }
    output.push_str(&format!("see {entrypoint} for the full workflow.\n"));
    output
}

/// Resolve the home `--scaffold` lays the FS starter in (§FS-init.2.1): the FS
/// row's one place, read the way the guidance reads a row's (§FS-init.2.2.2.3).
pub(crate) fn init_fs_home(config: &Config) -> InitFsHome {
    let fs_row = config.kinds.iter().find(|kind| kind.kind == "FS");
    match fs_row.and_then(row_home) {
        Some(InitAllocationHome::File {
            path,
            heading_name,
            heading_marker,
        }) => InitFsHome::File {
            path,
            heading_name,
            heading_marker,
        },
        Some(InitAllocationHome::Folder { path }) => InitFsHome::Folder { path },
        // §FS-init.2.2.2.3: no FS home falls back to the built-in FS folder, which
        // stays the scaffold's, read from the one table that defines it
        // (§FS-config.3.4.4).
        None => InitFsHome::Folder {
            path: default_kind_folder("FS")
                .expect("FS has a built-in folder")
                .to_string(),
        },
    }
}

/// Every starter file of the `software-development` template, paired with the
/// path it lands at (§FS-init.2.1). Every
/// owned ID example is rendered from its kind's effective format (§FS-init.2.1.3).
pub(crate) fn docs_scaffold_for_config(
    fs_home: &InitFsHome,
    config: &Config,
) -> Vec<(String, String)> {
    let project = config.project();
    let mut files = Vec::new();
    match fs_home {
        InitFsHome::File { path, .. } => files.push((
            path.clone(),
            render_scaffold_id_shapes(FS_FILE_TEMPLATE, "FS", project),
        )),
        InitFsHome::Folder { path } => files.push((
            format!("{path}/README.md"),
            render_scaffold_id_shapes(FS_README_TEMPLATE, "FS", project),
        )),
    }
    // §FS-init.2.1.3: the `REQ` index lands right after the FS home, in the
    // folder the config's `REQ` row names (its first, one starter), and only where it declares one.
    if let Some(folder) = config
        .kinds
        .iter()
        .find(|kind| kind.kind == "REQ")
        .and_then(|kind| kind.folder.as_ref())
    {
        files.push((
            format!("{folder}/README.md"),
            render_scaffold_id_shapes(REQ_README_TEMPLATE, "REQ", project),
        ));
    }
    files.extend(
        [
            (
                "docs/grund.md",
                render_scaffold_id_shapes(GRUND_DOC_TEMPLATE, "GRUND", project),
            ),
            (
                "docs/goals.md",
                render_scaffold_id_shapes(GOALS_TEMPLATE, "GOAL", project),
            ),
            (
                "docs/roadmap.md",
                "# Roadmap\n\n<!-- placeholder - replace with real content -->\n".to_string(),
            ),
            (
                "docs/changelog.md",
                "# Changelog\n\n<!-- placeholder - replace with real content -->\n".to_string(),
            ),
            (
                "docs/architecture/README.md",
                render_scaffold_id_shapes(AS_README_TEMPLATE, "AR", project),
            ),
            (
                "docs/decisions/architectural/README.md",
                render_scaffold_id_shapes(DA_README_TEMPLATE, "DA", project),
            ),
            (
                "docs/decisions/functional/README.md",
                render_scaffold_id_shapes(DF_README_TEMPLATE, "DF", project),
            ),
            ("tests/e2e/README.md", render_e2e_readme(fs_home)),
            (
                "tests/integration/.gitkeep",
                canonical_template_text(GITKEEP_TEMPLATE),
            ),
        ]
        .into_iter()
        .map(|(path, contents)| (path.to_string(), contents)),
    );
    files
}

/// The scaffold under the default config, for tests that read its shape.
#[cfg(test)]
pub(crate) fn docs_scaffold(fs_home: &InitFsHome) -> Vec<(String, String)> {
    docs_scaffold_for_config(fs_home, &Config::default_for(std::path::PathBuf::from(".")))
}

fn render_e2e_readme(fs_home: &InitFsHome) -> String {
    canonical_template_text(E2E_README_TEMPLATE).replace("{fs_home}", fs_home.path())
}
