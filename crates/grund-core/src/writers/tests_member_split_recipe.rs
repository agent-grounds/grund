//! Complete setup advice on both skill surfaces (§FS-init.5.4).

use crate::templates::{AGENT_SETUP_INSTRUCTIONS, SetupPart, setup_instructions};

/// §FS-init.5.4: the recipe is together in the setup guide's `members` section, rather
/// than scattered tokens, and the core's validation advice points to it.
#[test]
fn project_namespaces_setup_instructions_teach_complete_member_split() {
    let recipe = setup_instructions(SetupPart::Section("members"))
        .expect("setup instructions lack the `members` section");
    assert!(
        recipe.starts_with("## Splitting off a member\n"),
        "the `members` section is the complete 'Splitting off a member' recipe"
    );
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
        AGENT_SETUP_INSTRUCTIONS.contains("`grund agent-setup-instructions --section members`"),
        "validation advice must point to the member-splitting recipe by its invocation"
    );
}
