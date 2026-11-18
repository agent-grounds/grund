//! The config API selects the same member as its dependent views
//! (§AR-config.1.1, §FS-config.4.2, §FS-schema.2).

use crate::fixture::{FORMAT, Tree};
use grund_core::{effective_config, effective_project, schema_view};

#[test]
fn configless_member_effective_config_root_file_and_defaults() {
    let tree = Tree::workspace();
    let member = tree.0.join("apps/nocfg");
    let mut failures = Vec::new();
    for path in tree
        .scopes()
        .into_iter()
        .chain([member.join("docs/functional-spec/login.md")])
    {
        let config = effective_config(&path).unwrap();
        if config.root != member || config.config_file.is_some() || config.id_format != FORMAT {
            failures.push(format!(
                "effective_config({}): expected root={}, file=None, format={FORMAT}; actual root={}, file={:?}, format={}",
                path.display(), member.display(), config.root.display(), config.config_file, config.id_format
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn configless_member_effective_project_and_schema_identity() {
    let tree = Tree::workspace();
    let mut failures = Vec::new();
    for path in tree.scopes() {
        let project = effective_project(&path).unwrap();
        if project.version != 1 || project.name.is_some() || project.workspace.declared {
            failures.push(format!(
                "effective_project({}): expected v1, name=None, workspace=false; actual v{}, name={:?}, workspace={}",
                path.display(), project.version, project.name, project.workspace.declared
            ));
        }
        let view = schema_view(&path).unwrap();
        if view.project != "nocfg"
            || view.config_file != "defaults, no grund.toml"
            || view.version != 1
            || !view.members.is_empty()
        {
            failures.push(format!(
                "schema_view({}): expected nocfg, defaults, no grund.toml, v1, no members; actual {}, {}, v{}, members={:?}",
                path.display(), view.project, view.config_file, view.version, view.members
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
