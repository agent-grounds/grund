/// The single and batch show usage page (§FS-cli.2.3, §FS-show.1).
fn print_show_help() {
    // §FS-terms.terms.1: the default read is a coordinate's lead.
    println!("grund show — print a coordinate's lead, so an agent pulls a single fact into");
    println!("context without loading the whole document. `show` is the default command.");
    println!();
    println!(
        "Usage:  grund [show] <ID>[.<section>] [PATH] [--section S] [--brief|--toc|--full] [--format text|md|json] [--path PATH]"
    );
    println!(
        "        grund show --batch [--all | --glob PATTERN...] [PATH] --format=json [--brief|--toc|--full] [--path PATH]"
    );
    println!();
    println!("Modes form an ordered ladder (each adds to the previous):");
    println!("  --brief                heading + first paragraph    e.g. grund --brief FS-login");
    println!("  (default)              + the rest of the lead, cut at the first child section");
    println!("  --toc                  + the nested section map     e.g. grund --toc FS-login");
    println!("  --full                 + every subsection body      e.g. grund --full FS-login");
    println!();
    println!("Other options:");
    println!("  --section S            show only that section path  e.g. grund FS-login --section 3.1");
    // §FS-cli.2.3.4: a section read opens with its own heading line in text too (§FS-show.2.2).
    println!("  --format text|md|json  text (default) is the body, opening with a selected section's");
    println!("                         own heading line; md adds the declaration heading above it;");
    println!("                         json wraps the text body");
    // §FS-cli.2.3.2: a default per form, as PATH narrows a generated batch but not one ID.
    println!("  --path PATH            repo or subtree to resolve the ID in (no PATH answers as `.`);");
    println!("                         narrows --batch --all and --glob, whose queries with no PATH");
    println!("                         cover the whole project or workspace discovered from the");
    println!("                         current directory");
    println!("                         e.g. grund FS-login --path docs/");
    println!(
        "  --batch                 read ordered {{\"id\":…,\"section\":…}} NDJSON queries from stdin"
    );
    println!("  --all                   with --batch, query every declaration and section");
    // §FS-show.1.9: the generated batch, kept where a pattern matches; no stdin.
    println!(
        "  --glob PATTERN          with --batch, query the coordinates a pattern matches; repeatable"
    );
    print_run_flag_help("grund FS-login docs/", false);
    println!();
    println!(
        "Exit:  0 printed · 1 ID not found / ambiguous / broken stub / unknown section · 2 unknown project alias, or CLI error."
    );
    println!(
        "Batch: 0 all queries succeeded · 1 any query failed · 2 invocation, input, or scan error."
    );
    println!();
    println!("Examples:");
    println!("  grund FS-login                   # the lead — the cheap default");
    println!("  grund FS-login --toc             # lead + section map");
    println!("  grund FS-login.3.1               # the lead of that nested section");
    println!("  grund FS-login --full            # the whole declaration body");
    println!(
        "  printf '%s\\n' '{{\"id\":\"FS-login\"}}' | grund show --batch --format=json"
    );
    println!("  grund show --batch --all --format=json # every coordinate, one scan");
    println!("  grund show --batch --glob 'FS-login{{,.**}}' --toc --format json # one ID and its sections");
    println!();
    println!(
        "ID not found? `grund list` shows every declared ID; `grund id <KIND> \"…\"` proposes a new one."
    );
    print_guide_links(
        &["https://github.com/agent-grounds/grund/blob/main/docs/user-facing/querying.md"],
        &["https://github.com/agent-grounds/grund/tree/main/examples/review-checklist"],
    );
}
