//! Complete setup advice on both skill surfaces (§FS-init.5.4).

use crate::templates::AGENT_SETUP_INSTRUCTIONS;

/// §FS-init.5.4: the recipe is together in the setup guide, rather than scattered tokens.
#[test]
fn project_namespaces_setup_instructions_teach_complete_member_split() {
    let text = AGENT_SETUP_INSTRUCTIONS;
    let heading = text
        .lines()
        .find(|line| line.starts_with('#') && line.ends_with("Splitting off a member"))
        .expect("setup instructions lack the complete 'Splitting off a member' recipe");
    let start = text.find(heading).unwrap();
    let recipe = &text[start..];
    let end = recipe.find("\n## ").unwrap_or(recipe.len());
    let recipe = &recipe[..end];
    for phrase in [
        "independently checked",
        "app",
        "package",
        "service",
        "subproject",
        "module",
        "component",
        "grund.toml",
        "project_name",
        "stable",
        "[workspace]",
        "members",
        "grund init",
        "member",
        "alias/<ID>",
        "one alias segment per",
        "grund check",
        "root",
    ] {
        assert!(
            recipe.contains(phrase),
            "member-splitting recipe must teach {phrase:?}"
        );
    }
    assert!(
        recipe.contains("not") || recipe.contains("never"),
        "exclude ordinary modules"
    );
    assert!(
        text[..start].contains("[Splitting off a member]")
            || text[end + start..].contains("[Splitting off a member]"),
        "validation advice must point back to the member-splitting recipe"
    );
}
