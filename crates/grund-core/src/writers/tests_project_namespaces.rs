//! Project-boundary guidance follows workspace discovery (§FS-init.2.3.4.16).

use super::render_agents_md;
use crate::config::{Config, load_config};
use crate::testing::{test_root, write};
use std::path::Path;

// §FS-init.2.3.4.16: frozen before extraction at 4115eeab9a; independent of assets.
const BEFORE: &str = include_str!("fixtures/project-namespaces-v17.txt");

fn render(target: &Path) -> String {
    let config = load_config(target).expect("load fixture config");
    render_agents_md("demo", &config, target, true)
}

fn assert_workspace_section(block: &str, marker: &str) {
    let start = block
        .find("### Project namespaces")
        .expect("workspace guidance");
    let end = block[start..].find("\n\n### ").expect("next section") + start;
    assert_eq!(
        &block[start..end],
        BEFORE.trim_end_matches('\n').replace("{MARKER}", marker),
        "workspace Project namespaces bytes changed"
    );
    assert!(block[..start].ends_with("\n\n"), "section separator");
}

/// §FS-init.2.3.4.16: standalone discovery omits both workspace sections.
#[test]
fn project_namespaces_omitted_without_workspace() {
    let root = test_root("project_namespaces_omitted_without_workspace");
    let config = Config::default_for(root.clone());
    let block = render_agents_md("demo", &config, &root, true);
    assert!(!block.contains("### Workspace members"));
    assert!(
        !block.contains("### Project namespaces"),
        "standalone target still emits Project namespaces"
    );
}

/// §FS-init.2.3.4.16: independent preservation control, including marker substitution.
#[test]
fn project_namespaces_workspace_bytes_preserved_for_configured_markers() {
    let root = test_root("project_namespaces_workspace_bytes_preserved_for_configured_markers");
    for marker in ["§", "@"] {
        write(
            &root.join("grund.toml"),
            &format!(
                "project_name = \"root\"\n[reference]\nmarker = \"{marker}\"\n[workspace]\nmembers = [\"api\"]\n"
            ),
        );
        write(
            &root.join("api/grund.toml"),
            &format!("project_name = \"api\"\n[reference]\nmarker = \"{marker}\"\n"),
        );
        for target in [&root, &root.join("api")] {
            let block = render(target);
            assert_workspace_section(&block, marker);
            assert!(block.contains("### Workspace members"));
            assert!(block.find("### Project namespaces") < block.find("### Workspace members"));
        }
    }
}

/// §FS-init.2.3.4.16: expansion failure does not erase discovered workspace guidance.
#[test]
fn project_namespaces_retained_when_workspace_members_are_suppressed() {
    let root = test_root("project_namespaces_retained_when_workspace_members_are_suppressed");
    for members in ["[\"missing\"]", "[\"apps/api\", \"services/api\"]"] {
        write(
            &root.join("grund.toml"),
            &format!("project_name = \"root\"\n[workspace]\nmembers = {members}\n"),
        );
        std::fs::create_dir_all(root.join("apps/api")).unwrap();
        std::fs::create_dir_all(root.join("services/api")).unwrap();
        let block = render(&root);
        assert!(!block.contains("### Workspace members"));
        assert_workspace_section(&block, "§");
    }
}

/// §FS-init.2.3.4.16, §FS-workspace.6.1: claimed chains and unrelated ancestors.
#[test]
fn project_namespaces_nested_claims_and_unclaimed_ancestors() {
    let root = test_root("project_namespaces_nested_claims_and_unclaimed_ancestors");
    write(
        &root.join("grund.toml"),
        "project_name = \"root\"\n[workspace]\nmembers = [\"group\"]\n",
    );
    write(
        &root.join("group/grund.toml"),
        "project_name = \"group\"\n[workspace]\nmembers = [\"leaf\"]\n",
    );
    write(
        &root.join("group/leaf/grund.toml"),
        "project_name = \"leaf\"\n",
    );
    let leaf = render(&root.join("group/leaf"));
    assert_workspace_section(&leaf, "§");
    assert!(leaf.contains("### Workspace members"));

    write(&root.join("stray/grund.toml"), "project_name = \"stray\"\n");
    let stray = render(&root.join("stray"));
    assert!(!stray.contains("### Workspace members"));
    assert!(
        !stray.contains("### Project namespaces"),
        "unclaimed ancestor leaked guidance"
    );
}

/// §FS-init.2.3.4.16, §FS-workspace.6.1.8: an unclaimed group keeps its own tree.
#[test]
fn project_namespaces_unclaimed_group_keeps_its_own_workspace() {
    let root = test_root("project_namespaces_unclaimed_group_keeps_its_own_workspace");
    write(
        &root.join("grund.toml"),
        "project_name = \"outer\"\n[workspace]\nmembers = [\"listed\"]\n",
    );
    std::fs::create_dir_all(root.join("listed")).unwrap();
    write(
        &root.join("stray/grund.toml"),
        "project_name = \"stray\"\n[workspace]\nmembers = [\"leaf\"]\n",
    );
    write(
        &root.join("stray/leaf/grund.toml"),
        "project_name = \"leaf\"\n",
    );
    let block = render(&root.join("stray"));
    assert_workspace_section(&block, "§");
    assert!(block.contains("[`leaf`](leaf/)"));
    assert!(!block.contains("`outer`") && !block.contains("`listed`"));
}

/// §FS-init.2.3.7.1: support and checker agree on a new base-plus-one version pair.
#[test]
fn project_namespaces_version_selectors_advance_together() {
    let root = test_root("project_namespaces_version_selectors_advance_together");
    let base = super::super::block_version(&Config::default_for(root.clone()));
    write(
        &root.join("grund.toml"),
        "[[kinds]]\nkind = \"RULE\"\nfolder = \"docs/rules\"\nrules = true\n",
    );
    let rules = super::super::block_version(&load_config(&root).unwrap());
    assert_eq!(rules, base + 1);
    assert_eq!(crate::grammar::AGENTS_BLOCK_VERSION, rules);
    assert!(
        base > 18,
        "workspace-only guidance must advance beyond v17/v18"
    );
}
