//! The two surfaces an agent reads before it runs `grund <ID>` say that a section read opens
//! with its own heading line, as every format prints it (§FS-show.2.2): the `show` page's
//! `--format` row (§FS-cli.2.3.4) and the managed block's first grounding rung
//! (§FS-init.2.3.4.3).

use std::fs;

#[path = "support/init_fixture.rs"]
mod init_fixture;

use init_fixture::{manifest_dir, run_grund, workdir};

const FORMAT_ROW: &str =
    "  --format text|md|json  text (default) is the body, opening with a selected section's
                         own heading line; md adds the declaration heading above it;
                         json wraps the text body
";
const LEAD_RUNG: &str = "- `grund <ID>` — the lead, cut at the first child section: heading-less for a \
whole declaration, while `grund <ID>.<section>` opens with that section's heading line. The cheap \
first read for a bare `§<ID>` citation.";

/// The `--format` row reads exactly as §FS-cli.2.3.4 fixes it and no longer calls text heading-less.
#[test]
#[ignore = "red until #1026 rewrites the show page's --format row; implement removes this"]
fn show_help_format_row_names_the_section_heading() {
    let output = run_grund(&["show", "--help"], manifest_dir());
    assert!(output.status.success(), "show --help failed");
    let help = String::from_utf8(output.stdout).expect("utf-8 help");
    assert!(
        !help.contains("md keeps the heading"),
        "the --format row still says only md keeps the heading:\n{help}"
    );
    assert!(
        help.contains(FORMAT_ROW),
        "the --format row is not §FS-cli.2.3.4's:\n{help}"
    );
}

/// The block's first rung reads exactly as §FS-init.2.3.4.3 fixes it, once.
#[test]
#[ignore = "red until #1026 rewrites the block's `grund <ID>` rung; implement removes this"]
fn init_block_lead_rung_names_the_section_heading() {
    let target = workdir("init_block_lead_rung_names_the_section_heading");
    let output = run_grund(
        &["init", target.to_str().unwrap(), "--agents-md"],
        manifest_dir(),
    );
    assert!(
        output.status.success(),
        "init failed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let agents = fs::read_to_string(target.join("AGENTS.md")).expect("read AGENTS.md");
    assert!(
        !agents.contains("the lead (heading-less, cut at the first child section)"),
        "the block still calls every default read heading-less:\n{agents}"
    );
    let rungs: Vec<&str> = agents.lines().filter(|line| *line == LEAD_RUNG).collect();
    assert_eq!(
        rungs.len(),
        1,
        "the block does not carry §FS-init.2.3.4.3's rung once:\n{agents}"
    );
}
