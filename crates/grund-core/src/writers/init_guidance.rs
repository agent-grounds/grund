//! The typed post-init guidance and `--docs` scaffold (§FS-init.2.1–2.2).

use crate::config::Config;
use crate::scanner::CANONICAL_AGENT_ENTRYPOINT;
use crate::templates::{
    AS_README_TEMPLATE, DA_README_TEMPLATE, DF_README_TEMPLATE, E2E_README_TEMPLATE,
    FS_README_TEMPLATE, GITKEEP_TEMPLATE, GOALS_TEMPLATE, GRUND_DOC_TEMPLATE,
    REQUIREMENTS_TEMPLATE, canonical_template_text, render_scaffold_id_shapes,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitNext {
    pub docs: bool,
    pub entrypoint: String,
    pub fs_home: InitFsHome,
    /// Whether the effective scanner found a readable file before this guidance
    /// was rendered (§FS-init.2.2.2). Both command adapters consume this decision;
    /// neither reconstructs scanner policy from paths.
    pub scan_reads_file: bool,
}

impl InitNext {
    /// Render the shared trailing guidance for the shipped CLI and the deprecated
    /// core command adapter (§FS-init.2.2.2).
    pub fn render(&self) -> String {
        render_next_block_for_home(
            self.docs,
            Some(&self.entrypoint),
            &self.fs_home,
            self.scan_reads_file,
        )
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

/// The trailing `next:` guidance block (§FS-init.2.2.2). Suppressed by the caller
/// when every reported path was `exists ` — when the repo is already current
/// there is no next step to teach.
fn render_next_block_for_home(
    docs: bool,
    entrypoint: Option<&str>,
    fs_home: &InitFsHome,
    scan_reads_file: bool,
) -> String {
    let mut output = "\nnext:\n".to_string();
    if docs {
        output.push_str("  1. run `grund check` — a freshly scaffolded tree is clean\n");
        match fs_home {
            InitFsHome::File {
                path,
                heading_name,
                heading_marker,
            } => {
                output.push_str(&format!(
                    "  2. allocate an ID:  ID=$(grund id FS \"…\")  then add it to {path}\n"
                ));
                output.push_str(&format!(
                    "     ({heading_name}: `{heading_marker} <ID>: <one-line statement of the behavior>`)\n"
                ));
            }
            InitFsHome::Folder { path } => {
                output.push_str(&format!(
                    "  2. allocate an ID:  ID=$(grund id FS \"…\")  then add it under {path}\n"
                ));
                output.push_str("     (H1: `# <ID>: <one-line statement of the behavior>`)\n");
            }
        }
        output.push_str(
            "  3. cite it as §<ID> from the docs and e2e tests that depend on it, then `grund check` again\n",
        );
    } else {
        let fs_home_path = match fs_home {
            InitFsHome::File { path, .. } | InitFsHome::Folder { path } => path,
        };
        output.push_str(&format!(
            "  1. re-run with --docs to scaffold the FS home ({fs_home_path}), docs/, and tests/ (or create them yourself)"
        ));
        if !scan_reads_file {
            output.push_str(" — until then `grund check` has nothing to scan");
        }
        output.push('\n');
        output.push_str("  2. run `grund check` — a scaffolded tree is clean\n");
        match fs_home {
            InitFsHome::File { path, .. } => output.push_str(&format!(
                "  3. allocate an ID:  ID=$(grund id FS \"…\")  then add it to {path}\n"
            )),
            InitFsHome::Folder { path } => output.push_str(&format!(
                "  3. allocate an ID:  ID=$(grund id FS \"…\")  then add it under {path}\n"
            )),
        }
    }
    output.push_str(&format!(
        "see {} for the full workflow.\n",
        entrypoint.unwrap_or(CANONICAL_AGENT_ENTRYPOINT)
    ));
    output
}

/// Resolve the configured functional-spec home used by guidance and scaffolds.
pub(crate) fn init_fs_home(config: &Config) -> InitFsHome {
    if let Some(kind) = config.kinds.iter().find(|kind| kind.kind == "FS") {
        if let Some(file) = &kind.file {
            let (heading_name, heading_marker) = if file == "docs/grund.md" {
                ("H1", "#")
            } else {
                ("H2", "##")
            };
            return InitFsHome::File {
                path: file.clone(),
                heading_name,
                heading_marker,
            };
        }
        if let Some(folder) = &kind.folder {
            return InitFsHome::Folder {
                path: folder.clone(),
            };
        }
    }
    InitFsHome::File {
        path: "requirements.md".to_string(),
        heading_name: "H2",
        heading_marker: "##",
    }
}

/// Every `--docs` stub, paired with the path it lands at (§FS-init.2.1). Every
/// owned ID example is rendered from its kind's effective format (§FS-init.2.1.3).
pub(crate) fn docs_scaffold_for_config(
    fs_home: &InitFsHome,
    config: &Config,
) -> Vec<(String, String)> {
    let mut files = Vec::new();
    match fs_home {
        InitFsHome::File { path, .. } => files.push((
            path.clone(),
            render_scaffold_id_shapes(REQUIREMENTS_TEMPLATE, "FS", config),
        )),
        InitFsHome::Folder { path } => files.push((
            format!("{path}/README.md"),
            render_scaffold_id_shapes(FS_README_TEMPLATE, "FS", config),
        )),
    }
    files.extend(
        [
            (
                "docs/grund.md",
                render_scaffold_id_shapes(GRUND_DOC_TEMPLATE, "GRUND", config),
            ),
            (
                "docs/goals.md",
                render_scaffold_id_shapes(GOALS_TEMPLATE, "GOAL", config),
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
                render_scaffold_id_shapes(AS_README_TEMPLATE, "AR", config),
            ),
            (
                "docs/decisions/architectural/README.md",
                render_scaffold_id_shapes(DA_README_TEMPLATE, "DA", config),
            ),
            (
                "docs/decisions/functional/README.md",
                render_scaffold_id_shapes(DF_README_TEMPLATE, "DF", config),
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
    let fs_home_path = match fs_home {
        InitFsHome::File { path, .. } | InitFsHome::Folder { path } => path,
    };
    canonical_template_text(E2E_README_TEMPLATE).replace("{fs_home}", fs_home_path)
}
