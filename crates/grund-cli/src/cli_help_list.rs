/// The `grund list` page (§FS-cli.2.3, §FS-list.1): the catalog, its filters, and the
/// count and size rungs read off the same selection.
fn print_list_help() {
    println!("grund list — the ID catalog: every declared ID in the repo, with where it's");
    println!("declared and its one-line title. The complement of `grund refs` (which lists");
    println!("the citations of one ID) — `list` is the index of what you can read with `grund <ID>`.");
    println!();
    println!(
        "Usage:  grund list [PATH] [--kind KIND[,KIND]...] [--selector SUBJECT] [--glob PATTERN]... [--unused]"
    );
    println!(
        "                   [--summary | --total] [--size[=lines,words,bytes]] [--top N] [--format text|json]"
    );
    println!();
    print_no_path_scope_help();
    println!();
    println!("Output is one line per declared ID, `<ID>  <path>:<line>  <title>`, sorted by ID.");
    println!(
        "Stub-and-inline pairs collapse to one line; a duplicate-declared ID gets a line per home."
    );
    println!("Size mode adds every citable section and reports selected lead/full measurement pairs.");
    println!();
    println!("Options:");
    println!("  --kind KIND[,KIND]  only selected kinds; repeatable       e.g. grund list --kind FS,AR");
    println!("  --selector SUBJECT  only one declaration/chapter selector, e.g. FS.requirements");
    // §FS-coordinate-globs: `*`, `**` as a whole component, `{a,b}`, anchored per project.
    println!(
        "  --glob PATTERN      only coordinates the pattern matches; repeatable  e.g. --glob 'FS-login.**'"
    );
    // §FS-list.1.3.1: only a configured `E2E` kind's cases are left out.
    println!(
        "  --unused            only declarations nothing cites yet (skips a configured E2E kind unless selected)"
    );
    println!("  --summary           one row per kind with count and home  e.g. grund list --summary");
    // §FS-list.total: the count of the rows the same filters list.
    println!(
        "  --total             only the count of the rows             e.g. grund list --glob 'FS-*' --total"
    );
    println!("  --size[=lines,words,bytes]  coordinate lead/full sizes; bare selects all three units");
    println!("  --top N             largest N leads by the first selected unit; requires --size");
    println!(
        "  --format text|json   text (default) is the table on stdout; json emits NDJSON (adds `refs` count)."
    );
    print_run_flag_help("grund list docs/", true);
    println!();
    println!(
        "Exit:  0 scan succeeded (an empty catalog prints nothing) · 2 unreadable tree, an unknown --kind, or a refused --glob."
    );
    println!();
    println!("Examples:");
    println!("  grund list                      # the whole catalog");
    println!("  grund list --kind FS,AR docs/   # specs and architecture IDs under docs/");
    println!("  grund list --summary            # counts by kind");
    println!("  grund list --selector FS.requirements # every FS requirements chapter");
    println!("  grund list --glob 'FS-login.**' --total # how many sections FS-login holds");
    println!("  grund list --size=words --top 10 # largest coordinate leads");
    // §FS-list.1.3.1: only a configured `E2E` kind's cases are left out.
    println!(
        "  grund list --unused             # uncited declarations (specs, decisions, …) — configured E2E cases excluded"
    );
    // §FS-list.1.3.2: a taught `--unused --kind` example selects a default kind, never `E2E`.
    println!("  grund list --unused --kind FS   # uncited FS declarations only");
    println!("  grund list --kind FS --format json # one NDJSON record per ID, with its refs count");
    print_guide_links(
        &[
            "https://github.com/agent-grounds/grund/blob/main/docs/user-facing/coordinate-sizes.md",
            "https://github.com/agent-grounds/grund/blob/main/docs/user-facing/querying.md",
        ],
        &["https://github.com/agent-grounds/grund/tree/main/examples/review-checklist"],
    );
}
