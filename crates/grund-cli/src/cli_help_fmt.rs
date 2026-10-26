/// The `grund fmt --help` page (§FS-cli.2.3.3, §FS-fmt): the rewrite modes, their report
/// (§FS-fmt.3.5.1, §FS-fmt.3.7, §FS-section-lock.write.report),
/// the exit codes, and examples of each mode.
fn print_fmt_help() {
    println!(
        "grund fmt — normalize citation syntax: rewrite the `$$` trigger to the `§` marker,"
    );
    println!(
        "optionally upgrade bare ID tokens, and optionally emit Markdown cross-reference links."
    );
    println!();
    println!("After a declaration rename or file move, first edit the declaration and affected");
    println!("authored citation IDs. When cross-reference emission applies, fmt re-derives generated");
    println!("Markdown link destinations and anchors; it does not repair arbitrary URLs or rename IDs.");
    println!();
    println!("Run --write from the config/project root or workspace root to update eligible section locks.");
    println!("Locks are keyed by declaration ID: a file move alone does not rename a lock file.");
    println!("fmt never silently approves a protected section move; fix citations or leave a pointer first.");
    println!("Then run grund check and grund fmt --check, and commit the generated lock changes.");
    println!();
    println!("Usage:  grund fmt [PATH] [--check | --write] [--marker] [--cross-refs]");
    println!();
    print_no_path_scope_help();
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
        "                 runs by default in both modes for Markdown scopes; in version 1, set [fmt.cross_refs].enabled = false to opt out"
    );
    print_run_flag_help("grund fmt docs/", false);
    println!();
    println!(
        "With neither --check nor --write, fmt previews every change --write would apply and exits 1 if any."
    );
    println!(
        "Each preview row names every kind of rewrite its line gets: `bare → marker, markdown link`."
    );
    println!(
        "--write prints `rewrote N lines:` and its totals, e.g. `rewrote 7 lines: 6 citations marked, 6 links written`,"
    );
    println!("then one `  <path> (count)` line per file touched.");
    println!(
        "After a rename, a lock file fmt deletes or creates says so: its first preview row ends"
    );
    println!(
        "`; deletes the file` or `; creates the file`, and its --write line ends ` deleted` or ` created`."
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
    println!("  grund fmt --write               # after an authored rename/move, maintain links and locks from the root");
    println!("  grund fmt --write --marker      # also mark bare ID tokens");
}
