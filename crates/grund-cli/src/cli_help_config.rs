/// The `grund config --help` page (§FS-cli.2.3, §FS-cli.6): `show` in the file's
/// own version (§FS-config.4.2), `validate`, `explain` (§FS-config-explain.1) and
/// `migrate` (§FS-config-migrate.1), how the config is discovered, and examples of each.
fn print_config_help() {
    println!("grund config — inspect the effective `grund.toml` discovered from a path.");
    println!();
    println!("Usage:  grund config <show | validate> [PATH]");
    println!("        grund config explain [TARGET] [PATH] [--path PATH] [--codes | --all]");
    println!("        grund config migrate --to 2 [PATH] [--write] [--drop-comments] [--full]");
    println!();
    println!(
        "  show       print the effective config as TOML in the file's own version (defaults filled in for keys you didn't set)."
    );
    println!(
        "  validate   parse the discovered config and report the first error; exit 0 if it's well-formed."
    );
    println!(
        "  explain    say what a table, a key or a check code is, in the version the file is read as; with no target, list"
    );
    println!(
        "             the tables. It reads the file's version and nothing else, so a config other commands refuse is answered."
    );
    // §FS-config-explain.1.1: a path with one reading is answered.
    println!(
        "             A dotted path with one reading, such as schema.sources, is answered as '[schema.sources]' is."
    );
    println!(
        "  migrate    preview the version-2 file that means what this version-1 config means, and report where each key went;"
    );
    // §FS-config-migrate.6.5, §FS-config-migrate.2.2: compact success and refusal.
    println!("             writes nothing unless --write is given; successful writes and refused previews use a compact report.");
    println!();
    println!("Options:");
    println!("  --codes              explain: every check code the version can report, one line each.");
    println!("                       e.g. grund config explain --codes");
    println!("  --all                explain: every table with its keys, then every check code.");
    println!("                       e.g. grund config explain --all");
    println!(
        "  --path PATH          explain: the path to discover the config from, so the first positional stays the target."
    );
    println!("                       e.g. grund config explain oversized-lead --path docs/");
    println!(
        "  --to 2               migrate: the version to migrate to; required, and 2 is the only one."
    );
    println!("                       e.g. grund config migrate --to 2");
    println!(
        "  --write              migrate: replace the config with the candidate, once both comparisons pass."
    );
    println!("                       e.g. grund config migrate --to 2 --write");
    // §FS-config-migrate.1.3: expansion belongs only to migration presentation.
    println!(
        "  --full               migrate: restore the candidate and complete report after a successful write, or a preview only comments refuse."
    );
    println!("                       e.g. grund config migrate --to 2 --write --full");
    println!(
        "  --drop-comments      migrate: let --write drop the comments the preview lists, and name them again."
    );
    println!("                       e.g. grund config migrate --to 2 --write --drop-comments");
    print_run_flag_help("grund config show docs/", false);
    println!();
    println!(
        "PATH defaults to `.`; config is discovered by walking up from it — the root `grund.toml` is the home, `.agents/grund.toml` a deprecated fallback."
    );
    // §FS-cli.6 — the rule the line states; a printed ID would name
    // nothing in the reader's tree (§REQ-shipped-surfaces.1).
    println!("There is no `--config <file>` override: it is discovered, not pointed at.");
    println!();
    println!(
        "Exit:  0 well-formed / printed / answered / migrated · 1 `validate` found an error, `explain` knows no such target, or `migrate` refused · 2 no subcommand, or the config couldn't be read."
    );
    println!();
    println!("Examples:");
    println!("  grund config show               # the effective config, defaults filled in");
    println!("  grund config validate           # exit 1 if the discovered config has an error");
    println!("  grund config show docs/         # the config that governs docs/");
    println!(
        "  grund config explain oversized-lead   # what a check code means, its severity and what turns it on"
    );
    println!(
        "  grund config explain '[schema.sources]'           # a version-2 table: its keys, bracketed as in the file"
    );
    println!(
        "  grund config explain '[schema.sources] exclude'   # one key of a version-2 table, after its bracketed header"
    );
    println!("  grund config migrate --to 2     # the version-2 file, previewed; nothing written");
    // §FS-config-migrate.6.5: the ordinary apply step reports only its outcome and follow-up.
    println!("  grund config migrate --to 2 --write   # compact report on stderr; stdout is empty");
    println!(
        "  grund config migrate --to 2 --write --full   # apply with candidate and complete report"
    );
    print_guide_links(
        &[
            "https://github.com/agent-grounds/grund/blob/main/docs/user-facing/explain.md",
            "https://github.com/agent-grounds/grund/blob/main/docs/user-facing/migrating.md",
        ],
        &[
            "https://github.com/agent-grounds/grund/tree/main/examples/config-explain",
            "https://github.com/agent-grounds/grund/tree/main/examples/migrate",
        ],
    );
}
