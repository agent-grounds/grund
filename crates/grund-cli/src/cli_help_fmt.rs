/// The `grund fmt --help` page (§FS-cli.2.3, §FS-fmt): the rewrite modes, their report,
/// the exit codes, and examples of each mode.
fn print_fmt_help() {
    println!(
        "grund fmt — normalize citation syntax: rewrite the `$$` trigger to the `§` marker,"
    );
    println!(
        "optionally upgrade bare ID tokens, and optionally emit Markdown cross-reference links."
    );
    println!();
    println!("Usage:  grund fmt [PATH] [--check | --write] [--marker] [--cross-refs]");
    println!();
    println!("Options:");
    println!(
        "  --check        report pending rewrites, exit 1 if any exist         e.g. grund fmt --check"
    );
    println!(
        "  --write        apply the changes in place                           e.g. grund fmt --write"
    );
    println!(
        "  --marker       also prefix bare `<ID>` tokens with the marker        e.g. grund fmt --write --marker"
    );
    println!(
        "  --cross-refs   wrap citations as Markdown links to targets          e.g. grund fmt --write --cross-refs"
    );
    println!(
        "                 runs by default in both modes for Markdown scopes; set [fmt.cross_refs].enabled = false to opt out"
    );
    println!();
    println!(
        "With neither --check nor --write, fmt previews every change --write would apply and exits 1 if any."
    );
    println!(
        "--write prints `rewrote N lines:` then one `  <path> (count)` line per file touched."
    );
    println!(
        "The report goes to stdout (like `grund check`); CLI-level `error:` lines go to stderr."
    );
    println!();
    println!(
        "Exit:  0 nothing to do, or --write succeeded · 1 changes pending (dry run / --check) · 2 unreadable tree or CLI error."
    );
    println!();
    println!("Examples:");
    println!("  grund fmt                       # preview every rewrite --write would apply");
    println!("  grund fmt --check               # the same preview as a gate: exit 1 if pending");
    println!("  grund fmt --write               # rewrite `$$` to `§` in place");
    println!("  grund fmt --write --marker      # also mark bare ID tokens");
}
