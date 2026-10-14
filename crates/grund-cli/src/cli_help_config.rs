/// The `grund config --help` page (§FS-cli.2.3, §FS-cli.6): `show`, `validate` and
/// `migrate` (§FS-config-migrate.1), how the config is discovered, and examples of each.
fn print_config_help() {
    println!("grund config — inspect the effective `grund.toml` discovered from a path.");
    println!();
    println!("Usage:  grund config <show | validate> [PATH]");
    println!("        grund config migrate --to 2 [PATH] [--write] [--drop-comments]");
    println!();
    println!(
        "  show       print the effective config as TOML (defaults filled in for keys you didn't set)."
    );
    println!(
        "  validate   parse the discovered config and report the first error; exit 0 if it's well-formed."
    );
    println!(
        "  migrate    print the version-2 file that means what this version-1 config means, and report where each key went;"
    );
    println!("             writes nothing unless --write is given.");
    println!();
    println!("Options:");
    println!(
        "  --to 2               migrate: the version to migrate to; required, and 2 is the only one."
    );
    println!("                       e.g. grund config migrate --to 2");
    println!(
        "  --write              migrate: replace the config with the candidate, once both comparisons pass."
    );
    println!("                       e.g. grund config migrate --to 2 --write");
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
        "Exit:  0 well-formed / printed / migrated · 1 `validate` found an error, or `migrate` refused · 2 no subcommand, or the config couldn't be read."
    );
    println!();
    println!("Examples:");
    println!("  grund config show               # the effective config, defaults filled in");
    println!("  grund config validate           # exit 1 if the discovered config has an error");
    println!("  grund config show docs/         # the config that governs docs/");
    println!("  grund config migrate --to 2     # the version-2 file, previewed; nothing written");
}
