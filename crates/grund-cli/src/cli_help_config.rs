/// The `grund config --help` page (§FS-cli.2.3, §FS-cli.6): `show` and `validate`, how the
/// config is discovered, and examples of each.
fn print_config_help() {
    println!("grund config — inspect the effective `grund.toml` discovered from a path.");
    println!();
    println!("Usage:  grund config <show | validate> [PATH]");
    println!();
    println!(
        "  show       print the effective config as TOML (defaults filled in for keys you didn't set)."
    );
    println!(
        "  validate   parse the discovered config and report the first error; exit 0 if it's well-formed."
    );
    println!();
    println!(
        "PATH defaults to `.`; config is discovered by walking up from it — the root `grund.toml` is the home, `.agents/grund.toml` a deprecated fallback."
    );
    // §FS-cli.6 — the rule the line states; a printed ID would name
    // nothing in the reader's tree (§REQ-shipped-surfaces.1).
    println!("There is no `--config <file>` override: it is discovered, not pointed at.");
    println!();
    println!(
        "Exit:  0 well-formed / printed · 1 `validate` found an error · 2 no subcommand, or `show` couldn't read the config."
    );
    println!();
    println!("Examples:");
    println!("  grund config show               # the effective config, defaults filled in");
    println!("  grund config validate           # exit 1 if the discovered config has an error");
    println!("  grund config show docs/         # the config that governs docs/");
}
