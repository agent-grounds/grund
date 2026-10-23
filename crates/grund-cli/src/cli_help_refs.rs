/// The `grund refs` page (§FS-cli.2.3, §FS-refs.1): the citation sites of one coordinate,
/// and the two modes that select them by a pattern (§FS-refs.globs) or a source's own body
/// (§FS-refs.from).
fn print_refs_help() {
    // §FS-terms.terms.2: a coordinate's citation sites.
    println!("grund refs — list every citation site of a coordinate, as `path:line`, so you can");
    println!("see who depends on a declaration before you change it.");
    println!();
    println!("Usage:  grund refs <ID>[.<section>] [PATH] [--section S] [--descendants]");
    println!("                   [--summary] [--total] [--format text|json]");
    println!(
        "        grund refs --glob PATTERN [--glob …] [PATH] [--summary] [--total] [--format text|json]"
    );
    println!(
        "        grund refs --from COORDINATE [PATH] [--summary] [--total] [--format text|json]"
    );
    println!();
    print_no_path_scope_help();
    println!();
    println!("With a `.<section>` (or --section), only citations of that exact section are");
    println!("listed; --descendants widens that to the section and every section beneath it.");
    println!("An ID with no citations prints nothing and exits 0, or the line `not cited`");
    println!("under --total.");
    println!();
    // §FS-refs.1.1: the modes, and every positional the path under either.
    println!("--glob lists every site whose cited coordinate a pattern matches, dangling ones");
    println!("included; repeated, the patterns union and each site is listed once. A bare");
    println!("`--glob 'FS-login'` selects only the bare citations of FS-login, where");
    println!("`grund refs FS-login` is `--glob 'FS-login{{,.**}}'`. --from asks the other way,");
    println!("what one coordinate cites: the sites written in its lead, its ancestors and");
    println!("itself, never in a descendant, or the line `cites nothing` under --total.");
    println!("Under either, every positional is PATH.");
    println!();
    println!("Options:");
    println!(
        "  --section S          list only citations of that section path   e.g. grund refs FS-login --section 3"
    );
    println!(
        "  --descendants        also list citations of sections beneath it e.g. grund refs FS-login.3 --descendants"
    );
    // §FS-refs.1.2: the flags each mode refuses beside it.
    println!(
        "  --glob PATTERN       sites citing what it matches; repeatable   e.g. grund refs --glob 'FS-login.**'"
    );
    println!("                       not with --from, --section or --descendants.");
    println!(
        "  --from COORDINATE    what that coordinate's own body cites      e.g. grund refs --from FS-login.3"
    );
    println!("                       given once; not with --glob, --section or --descendants.");
    println!(
        "  --summary            group citations by citing file             e.g. grund refs FS-login --summary"
    );
    println!(
        "  --total              the set's size instead of its members      e.g. grund refs FS-login --total"
    );
    println!(
        "  --format text|json   text (default) prints `path:line: <citation>`; json emits NDJSON."
    );
    print_run_flag_help("grund refs FS-login src/", true);
    println!();
    println!("The citation list is the result, so it goes to stdout (text and json alike) —");
    println!("`grund refs <ID> | …` works like `grund list`. Only a `note:` goes to stderr:");
    // §FS-refs.2.1: the typo note, or the §FS-refs.2.1.1 pointer to the bulk query.
    println!("one for an ID declared nowhere, or, with no PATH and no section, one naming");
    println!("`grund list --unused` for a declared ID nothing cites.");
    println!();
    println!(
        "Exit:  0 scan succeeded (with or without hits) · 1 ID or --from source rejected after context"
    );
    println!("       · 2 run/scan error, or a refused --glob.");
    println!();
    println!("Examples:");
    println!("  grund refs FS-login             # every citation of FS-login");
    println!("  grund refs FS-login --summary   # one row per citing file");
    println!("  grund refs FS-login --total     # how many sites, in how many files");
    println!("  grund refs FS-login.3           # only citations of section 3");
    println!("  grund refs FS-login --format json # one NDJSON record per citation site");
    println!("  grund refs --glob 'FS-login.{{lockout,reset}}' # who cites either section");
    println!("  grund refs --from FS-login.3    # what FS-login.3 cites, and where");
}

/// The `grund cover` page (§FS-cli.2.3, §FS-cover.1): the citation graph by scanned
/// file, filtered by a pattern (§FS-cover.globs), or one file's line owners (§FS-cover.6).
fn print_cover_help() {
    println!("grund cover — group the citation graph by scanned file.");
    println!();
    println!("Usage:  grund cover [PATH] [--glob PATTERN]... [--format text|json]");
    println!("        grund cover FILE --lines N|N-M [--lines …] [--format text|json]");
    println!();
    print_no_path_scope_help();
    println!();
    println!("The command runs the same scan as `check` and `refs`, then prints one file");
    println!("record with the citations found in that file.");
    println!("With --glob a record keeps only the citations whose cited coordinate a pattern");
    println!("matches, and every scanned file keeps its record: `(no citations)` in text.");
    println!("With --lines it prints instead, for each range of FILE, the declaration");
    println!("and section that own its lines.");
    println!();
    println!("Options:");
    println!(
        "  --format text|json   text (default) groups citations by file; json emits one record per file."
    );
    // §FS-cover.globs: repeatable and unioned; §FS-cover.6.4: no grouping beside --lines.
    println!(
        "  --glob PATTERN       keep citations of what it matches; repeatable; not with --lines."
    );
    println!("                       e.g. grund cover src/ --glob 'FS-login.**'");
    println!(
        "  --lines N|N-M        report the owners of these lines of FILE; repeatable, one record per range."
    );
    print_run_flag_help("grund cover src/", true);
    println!();
    println!(
        "Exit:  0 scan succeeded · 2 unreadable tree, incomplete scan, a refused --glob, or CLI error."
    );
    println!();
    println!("Examples:");
    println!("  grund cover src/                # source files and their spec citations");
    println!("  grund cover --format json       # machine-readable coverage index");
    println!("  grund cover src/ --glob 'FS-login{{,.**}}' # where each file cites FS-login");
    println!(
        "  grund cover docs/FS-x.md --lines 105-121   # which spec points a diff hunk edits"
    );
}
